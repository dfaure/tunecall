// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::rc::Rc;

use slint::{Image, VecModel};

mod annotations;
mod db;
#[cfg(target_os = "android")]
mod immersive;
mod pdf;
mod setlist;
mod settings;
mod storage;
mod sync;

use db::Song;

// Include the slint-generated code
slint::include_modules!();

/// Width, in pixels, that PDF pages are rasterized to. The UI scales the result
/// to fit, so this is really a quality/sharpness knob.
const RENDER_WIDTH: i32 = 1200;

/// Cap on the rasterization upscale for pinch-zoom re-renders. Beyond ~2x the
/// bitmap of a tall page would exceed common GPU texture size limits (4096px)
/// and the pixel buffer gets very large.
const MAX_RENDER_ZOOM: f32 = 2.0;

/// Max search results shown at once.
const SEARCH_LIMIT: usize = 300;

/// Tab indices (must match the `Tab` order in `app-window.slint`).
const TAB_SEARCH: i32 = 0;
const TAB_BOOKS: i32 = 2;

/// What's currently shown in the viewer.
struct ViewerState {
    path: String,
    page: u16,
    count: u16,
    /// Index into the current nav list, so Prev/Next result can find the
    /// neighboring song.
    result_idx: usize,
    /// Width the page is currently rasterized at (bumped by pinch-zoom
    /// re-renders, reset to `RENDER_WIDTH` on every page change).
    render_width: i32,
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: slint::android::AndroidApp) -> Result<(), Box<dyn Error>> {
    // Resolve the app-specific storage directory before anything touches it.
    // There is no usable fallback on modern Android (scoped storage blocks
    // arbitrary shared paths), so a missing path is fatal.
    let data_dir = app
        .external_data_path()
        .or_else(|| app.internal_data_path())
        .expect("Android provided no app data path");
    storage::set_data_dir(data_dir);

    // File logging is opt-in (Books tab toggle): off by default so we don't
    // write a debug log for every user. The logger can't be reconfigured after
    // start(), so the choice is read once here and a toggle change only takes
    // effect on the next launch. When disabled we install no logger, so the
    // log:: calls below simply no-op.
    if settings::load().file_logging {
        // Log to a file inside the app data dir (shared storage like Download/
        // is not writable on modern Android without extra permissions).
        flexi_logger::Logger::try_with_env_or_str(
            "debug,android_activity::activity_impl::glue=off",
        )?
        .log_to_file(
            flexi_logger::FileSpec::default()
                .directory(storage::data_dir())
                .basename("tunecall"),
        )
        .format(flexi_logger::detailed_format)
        .start()?;
    }

    log::info!("tunecall started");
    // Stash the app handle for the immersive-fullscreen JNI calls (the viewer
    // hides the system bars). Clone: slint::android::init consumes `app`.
    immersive::init(app.clone());
    slint::android::init(app).unwrap();
    log::debug!("slint::android initialized");
    let ret = tunecall_main();
    if let Err(ref e) = ret {
        log::error!("{:?}", e);
    }
    // When we get here, exit process so Android restarts fresh next time
    std::process::exit(0);
}

/// (Re)load all per-PDF indexes into `library` and update the idle status.
fn reload_library(ui: &AppWindow, library: &Rc<RefCell<Vec<Song>>>) {
    match db::load_library() {
        Ok(songs) => {
            log::info!("loaded {} songs from per-PDF indexes", songs.len());
            *library.borrow_mut() = songs;
        }
        Err(e) => {
            log::warn!("loading library failed: {e}");
            library.borrow_mut().clear();
        }
    }
    set_idle_status(ui, library);
}

/// Status line shown when no search is active.
fn set_idle_status(ui: &AppWindow, library: &Rc<RefCell<Vec<Song>>>) {
    ui.set_status_error(false);
    let n = library.borrow().len();
    if n > 0 {
        ui.set_status(format!("{n} songs indexed. Type to search.").into());
    } else if db::list_books().is_empty() {
        ui.set_status(
            format!(
                "No song indexes found in\n{}\nTap Reload to download them.",
                storage::pdf_dir().display()
            )
            .into(),
        );
    } else {
        // Indexes are present but no matching PDF is installed yet, so nothing is
        // searchable. Tell the user where to put the PDFs.
        ui.set_status(
            format!(
                "No PDFs installed yet. Copy them to\n{}\nvia USB.",
                storage::pdf_dir().display()
            )
            .into(),
        );
    }
}

fn book_stem_from_path(path: &str) -> String {
    std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string()
}

fn load_and_set_annotations(ui: &AppWindow, state: &ViewerState) {
    let stem = book_stem_from_path(&state.path);
    let items: Vec<AnnotationItem> = annotations::load(&stem, state.page)
        .into_iter()
        .map(|a| AnnotationItem {
            x: a.x,
            y: a.y,
            text: a.text.into(),
        })
        .collect();
    ui.set_page_annotations(Rc::new(VecModel::from(items)).into());
}

/// Render the page described by `state` into the viewer, at the width it was
/// last rasterized at — so flipping pages while zoomed in (e.g. to crop the
/// margins) keeps both the zoom and its sharpness.
fn show_page(ui: &AppWindow, state: &ViewerState) {
    ui.set_viewer_error("".into());
    ui.set_page_number(state.page as i32 + 1);
    ui.set_page_count(state.count as i32);
    load_and_set_annotations(ui, state);
    match pdf::render_page(&state.path, state.page, state.render_width) {
        Ok(img) => ui.set_page_image(img),
        Err(e) => {
            log::warn!("render_page failed: {e}");
            ui.set_page_image(Image::default());
            ui.set_viewer_error(format!("Cannot render PDF: {e}").into());
        }
    }
}

fn empty_results() -> slint::ModelRc<SongResult> {
    Rc::new(VecModel::<SongResult>::default()).into()
}

/// Populate the Books overlay list (every indexed book + whether its PDF is
/// installed). Re-run after Reload so its effect shows there.
fn refresh_books(ui: &AppWindow) {
    let items: Vec<BookEntry> = db::list_books()
        .into_iter()
        .map(|b| BookEntry {
            name: b.name.into(),
            title: b.title.unwrap_or_default().into(),
            installed: b.has_pdf,
        })
        .collect();
    ui.set_book_list(Rc::new(VecModel::from(items)).into());
}

/// (Re)build the book stem → friendly title map from the indexed books, so the
/// UI can show "557 Standards" instead of the cryptic "557standrd" stem.
fn refresh_book_titles(titles: &Rc<RefCell<HashMap<String, String>>>) {
    let mut map = HashMap::new();
    for b in db::list_books() {
        if let Some(t) = b.title {
            map.insert(b.name.to_lowercase(), t);
        }
    }
    *titles.borrow_mut() = map;
}

/// Friendly book name for `stem`, falling back to the file stem when the index
/// recorded no title.
fn book_label(titles: &HashMap<String, String>, stem: &str) -> String {
    titles
        .get(&stem.to_lowercase())
        .cloned()
        .unwrap_or_else(|| stem.to_string())
}

/// "Book Title · p.N" for a song row, using the friendly title where known (the
/// file stem otherwise). N is the 1-based render page the viewer shows.
fn book_subtitle(titles: &HashMap<String, String>, stem: &str, page: i32) -> String {
    format!("{} · p.{}", book_label(titles, stem), page + 1)
}

/// Push the setlist list (name + song count) to the Setlists tab.
fn refresh_setlists(ui: &AppWindow, setlists: &Rc<RefCell<Vec<setlist::Setlist>>>) {
    let items: Vec<SetlistEntry> = setlists
        .borrow()
        .iter()
        .map(|s| SetlistEntry {
            name: s.name.clone().into(),
            song_count: s.songs.len() as i32,
        })
        .collect();
    ui.set_setlists(Rc::new(VecModel::from(items)).into());
}

/// Reflect the currently edited setlist (if any) into the editor properties.
/// `editing == None` collapses the editor back to the setlist list view
/// (`editing-index = -1`). Each song's `available` flag is whether its PDF is
/// installed (else it shows greyed and opens to a render error).
fn refresh_editor(
    ui: &AppWindow,
    setlists: &Rc<RefCell<Vec<setlist::Setlist>>>,
    editing: &Rc<RefCell<Option<usize>>>,
    titles: &Rc<RefCell<HashMap<String, String>>>,
) {
    let lists = setlists.borrow();
    let current = (*editing.borrow()).and_then(|i| lists.get(i).map(|sl| (i, sl)));
    if let Some((i, sl)) = current {
        let titles = titles.borrow();
        ui.set_editing_index(i as i32);
        ui.set_editing_name(sl.name.clone().into());
        let songs: Vec<SetlistSongEntry> = sl
            .songs
            .iter()
            .map(|s| SetlistSongEntry {
                title: s.title.clone().into(),
                subtitle: book_subtitle(&titles, &s.book, s.page).into(),
                available: db::resolve_pdf(&s.book).is_some(),
            })
            .collect();
        ui.set_editing_songs(Rc::new(VecModel::from(songs)).into());
    } else {
        ui.set_editing_index(-1);
        ui.set_editing_name("".into());
        ui.set_editing_songs(Rc::new(VecModel::<SetlistSongEntry>::default()).into());
    }
}

/// Persist setlists, logging (but not surfacing) failures — a failed save must
/// not block the edit the user just made.
fn save_setlists(setlists: &Rc<RefCell<Vec<setlist::Setlist>>>) {
    if let Err(e) = setlist::save(&setlists.borrow()) {
        log::warn!("saving setlists failed: {e}");
    }
}

/// Open the song at `idx` of the current nav list in the viewer. The nav list is
/// whatever the viewer is stepping through — the search hits or a setlist's
/// songs. Opening from a list and stepping Prev/Next both funnel through here.
fn open_result_at(
    ui: &AppWindow,
    nav: &Rc<RefCell<Vec<Song>>>,
    viewer: &Rc<RefCell<Option<ViewerState>>>,
    titles: &Rc<RefCell<HashMap<String, String>>>,
    idx: usize,
) {
    let (song, total) = {
        let nav = nav.borrow();
        let Some(song) = nav.get(idx).cloned() else {
            return;
        };
        (song, nav.len())
    };
    let path = song.file.to_string_lossy().into_owned();
    log::info!(
        "opening '{}' -> {} page {} ({path})",
        song.title,
        song.book,
        song.page
    );
    let count = pdf::page_count(&path).unwrap_or(0);
    let page = song.page.clamp(0, count.saturating_sub(1) as i32) as u16;

    // Show only the book's friendly name: the song title is in the page itself,
    // and would be wrong once the user pages Prev/Next to another song.
    ui.set_viewer_title(book_label(&titles.borrow(), &song.book).into());
    ui.set_result_index(idx as i32);
    ui.set_result_count(total as i32);
    // A different song starts back at fit-to-window zoom (page flips within a
    // song keep the zoom; see show_page).
    ui.set_viewer_zoom(1.0);
    ui.set_viewer_pan_x(0.0);
    ui.set_viewer_pan_y(0.0);
    let state = ViewerState {
        path,
        page,
        count,
        result_idx: idx,
        render_width: RENDER_WIDTH,
    };
    show_page(ui, &state);
    *viewer.borrow_mut() = Some(state);
    // A freshly opened song hasn't been added to the setlist yet; reset the
    // viewer's Add confirmation (only meaningful while in add mode).
    ui.set_viewer_added(false);
    ui.set_viewer_visible(true);
}

/// Append `song` to the setlist currently being edited and refresh the UI.
/// Shared by the list's Add button and the viewer's Add button.
fn add_song_to_editing(
    ui: &AppWindow,
    setlists: &Rc<RefCell<Vec<setlist::Setlist>>>,
    editing: &Rc<RefCell<Option<usize>>>,
    titles: &Rc<RefCell<HashMap<String, String>>>,
    song: Song,
) {
    let Some(i) = *editing.borrow() else {
        return;
    };
    if let Some(sl) = setlists.borrow_mut().get_mut(i) {
        sl.songs.push(setlist::SetlistSong {
            title: song.title,
            book: song.book,
            page: song.page,
        });
    }
    save_setlists(setlists);
    refresh_editor(ui, setlists, editing, titles);
    refresh_setlists(ui, setlists);
}

pub fn tunecall_main() -> Result<(), Box<dyn Error>> {
    std::panic::set_hook(Box::new(|info| {
        log::error!("Panic occurred: {}", info);
    }));

    let ui = AppWindow::new()?;

    // Create the PDF folder on first run so users have somewhere to drop books
    // (and the indexes that Reload downloads).
    let pdf_dir = storage::pdf_dir();
    if let Err(e) = std::fs::create_dir_all(&pdf_dir) {
        log::warn!("could not create {}: {e}", pdf_dir.display());
    }

    // The whole library, and the current search results (clones, so a clicked
    // row maps straight back to its song).
    let library: Rc<RefCell<Vec<Song>>> = Rc::new(RefCell::new(Vec::new()));
    let results: Rc<RefCell<Vec<Song>>> = Rc::new(RefCell::new(Vec::new()));
    let viewer: Rc<RefCell<Option<ViewerState>>> = Rc::new(RefCell::new(None));

    // The list the viewer is currently stepping through (search hits or a
    // setlist's songs). Kept separate from `results` so the Search tab's
    // displayed list and the viewer's Prev/Next can't desync.
    let nav: Rc<RefCell<Vec<Song>>> = Rc::new(RefCell::new(Vec::new()));

    // Setlists: the app's only writable user data, loaded from disk. `editing`
    // is the index of the setlist open in the editor (None = list view);
    // `add_hits` backs the in-editor "add songs" search.
    let setlists: Rc<RefCell<Vec<setlist::Setlist>>> = Rc::new(RefCell::new(setlist::load()));
    let editing: Rc<RefCell<Option<usize>>> = Rc::new(RefCell::new(None));
    let add_hits: Rc<RefCell<Vec<Song>>> = Rc::new(RefCell::new(Vec::new()));

    // Book stem → friendly title (for "557 Standards · p.34" instead of stems).
    let book_titles: Rc<RefCell<HashMap<String, String>>> = Rc::new(RefCell::new(HashMap::new()));

    reload_library(&ui, &library);
    refresh_book_titles(&book_titles);
    refresh_books(&ui);
    refresh_setlists(&ui, &setlists);
    refresh_editor(&ui, &setlists, &editing, &book_titles);
    // Land on the Books tab when nothing is installed yet (fresh setup needs
    // PDFs), otherwise the Search tab.
    ui.set_active_tab(if library.borrow().is_empty() {
        TAB_BOOKS
    } else {
        TAB_SEARCH
    });

    // App name / version / build, for the Books-tab footer (handy in bug reports).
    ui.set_about(
        format!(
            "TuneCall {} ({})",
            env!("CARGO_PKG_VERSION"),
            env!("GIT_HASH")
        )
        .into(),
    );

    // Debug-log toggle (Books tab). Reflect the saved state, and persist any
    // change. The Android logger only reads this at startup, so the change takes
    // effect on the next launch (the UI says so).
    ui.set_file_logging(settings::load().file_logging);
    // Folder the log lands in (data-dir root), shown in the "logging on" dialog
    // so the user can find it over USB.
    ui.set_log_dir(storage::data_dir().display().to_string().into());
    ui.on_set_file_logging(|enabled| {
        if let Err(e) = settings::save(&settings::Settings {
            file_logging: enabled,
        }) {
            log::warn!("saving settings failed: {e}");
        }
    });

    // The viewer asks to go fullscreen (hide the system bars) while it's open;
    // Android-only, a no-op elsewhere (the callback stays unconnected).
    #[cfg(target_os = "android")]
    ui.on_set_immersive(|enabled| immersive::set(enabled));

    ui.on_search({
        let ui_handle = ui.as_weak();
        let library = library.clone();
        let results = results.clone();
        let book_titles = book_titles.clone();
        move |text| {
            let ui = ui_handle.unwrap();
            let query = text.trim();
            if query.is_empty() {
                results.borrow_mut().clear();
                ui.set_results(empty_results());
                set_idle_status(&ui, &library);
                return;
            }
            let lib = library.borrow();
            let titles = book_titles.borrow();
            let hits = db::search(&lib, query, SEARCH_LIMIT);
            let items: Vec<SongResult> = hits
                .iter()
                .map(|s| SongResult {
                    title: s.title.clone().into(),
                    subtitle: book_subtitle(&titles, &s.book, s.page).into(),
                })
                .collect();
            let n = items.len();
            ui.set_results(Rc::new(VecModel::from(items)).into());
            ui.set_status_error(false);
            ui.set_status(if n == 0 {
                "No results".into()
            } else {
                format!("{n} result(s)").into()
            });
            *results.borrow_mut() = hits.into_iter().cloned().collect();
        }
    });

    ui.on_open_result({
        let ui_handle = ui.as_weak();
        let results = results.clone();
        let nav = nav.clone();
        let viewer = viewer.clone();
        let book_titles = book_titles.clone();
        move |idx| {
            let ui = ui_handle.unwrap();
            // The viewer navigates the current search hits. No setlist is being
            // built here, so the viewer's Add button stays hidden.
            ui.set_viewer_add_mode(false);
            *nav.borrow_mut() = results.borrow().clone();
            open_result_at(&ui, &nav, &viewer, &book_titles, idx as usize);
        }
    });

    ui.on_next_result({
        let ui_handle = ui.as_weak();
        let nav = nav.clone();
        let viewer = viewer.clone();
        let book_titles = book_titles.clone();
        move || {
            let ui = ui_handle.unwrap();
            let next = match viewer.borrow().as_ref() {
                Some(state) if state.result_idx + 1 < nav.borrow().len() => {
                    Some(state.result_idx + 1)
                }
                _ => None,
            };
            if let Some(idx) = next {
                open_result_at(&ui, &nav, &viewer, &book_titles, idx);
            }
        }
    });

    ui.on_prev_result({
        let ui_handle = ui.as_weak();
        let nav = nav.clone();
        let viewer = viewer.clone();
        let book_titles = book_titles.clone();
        move || {
            let ui = ui_handle.unwrap();
            let prev = match viewer.borrow().as_ref() {
                Some(state) if state.result_idx > 0 => Some(state.result_idx - 1),
                _ => None,
            };
            if let Some(idx) = prev {
                open_result_at(&ui, &nav, &viewer, &book_titles, idx);
            }
        }
    });

    ui.on_save_annotation({
        let ui_handle = ui.as_weak();
        let viewer = viewer.clone();
        move |x, y, text| {
            let ui = ui_handle.unwrap();
            let slot = viewer.borrow();
            let Some(state) = slot.as_ref() else { return };
            let stem = book_stem_from_path(&state.path);
            if let Err(e) = annotations::save(&stem, state.page, x, y, &text) {
                log::warn!("saving annotation failed: {e}");
            }
            load_and_set_annotations(&ui, state);
        }
    });

    // Reload = download the published indexes from the server, then reload from
    // disk. Runs on the Slint event loop (async-compat bridges reqwest's tokio),
    // so it can touch the Rc state directly without blocking the UI.
    ui.on_reload({
        let ui_handle = ui.as_weak();
        let library = library.clone();
        let results = results.clone();
        let setlists = setlists.clone();
        let editing = editing.clone();
        let book_titles = book_titles.clone();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_status_error(false);
            ui.set_status("Downloading indexes…".into());
            let ui_handle = ui_handle.clone();
            let library = library.clone();
            let results = results.clone();
            let setlists = setlists.clone();
            let editing = editing.clone();
            let book_titles = book_titles.clone();
            if let Err(e) = slint::spawn_local(async_compat::Compat::new(async move {
                let ui = ui_handle.unwrap();
                let download_err = match sync::download_indexes().await {
                    Ok(n) => {
                        log::info!("downloaded {n} index file(s)");
                        None
                    }
                    Err(e) => {
                        log::warn!("index download failed: {e}");
                        Some(e)
                    }
                };
                // Always rescan local files afterwards: installing a PDF is a
                // local action and must be picked up even when the index
                // download fails (e.g. offline).
                results.borrow_mut().clear();
                ui.set_results(empty_results());
                reload_library(&ui, &library);
                refresh_book_titles(&book_titles);
                refresh_books(&ui);
                // Installing a PDF can change a setlist song's availability.
                refresh_setlists(&ui, &setlists);
                refresh_editor(&ui, &setlists, &editing, &book_titles);
                // Report the download error last, so it isn't overwritten by the
                // idle status `reload_library` sets on success.
                if let Some(e) = download_err {
                    ui.set_status_error(true);
                    ui.set_status(format!("Download failed: {e}").into());
                }
            })) {
                log::error!("failed to schedule download: {e}");
            }
        }
    });

    ui.on_next_page({
        let ui_handle = ui.as_weak();
        let viewer = viewer.clone();
        move || {
            let ui = ui_handle.unwrap();
            let mut slot = viewer.borrow_mut();
            if let Some(state) = slot.as_mut()
                && state.count > 0
                && state.page + 1 < state.count
            {
                state.page += 1;
                show_page(&ui, state);
            }
        }
    });

    // The UI asks for a sharper rasterization once a zoom gesture settles (and
    // for the base width again when zoomed back out). Width changes only, so
    // no-ops are skipped; failures keep the current (blurrier) image.
    ui.on_render_zoomed({
        let ui_handle = ui.as_weak();
        let viewer = viewer.clone();
        move |zoom| {
            let ui = ui_handle.unwrap();
            let mut slot = viewer.borrow_mut();
            let Some(state) = slot.as_mut() else {
                return;
            };
            let width = (RENDER_WIDTH as f32 * zoom.clamp(1.0, MAX_RENDER_ZOOM)).round() as i32;
            if width == state.render_width {
                return;
            }
            match pdf::render_page(&state.path, state.page, width) {
                Ok(img) => {
                    ui.set_page_image(img);
                    state.render_width = width;
                }
                Err(e) => log::warn!("zoom re-render at width {width} failed: {e}"),
            }
        }
    });

    ui.on_prev_page({
        let ui_handle = ui.as_weak();
        let viewer = viewer.clone();
        move || {
            let ui = ui_handle.unwrap();
            let mut slot = viewer.borrow_mut();
            if let Some(state) = slot.as_mut()
                && state.page > 0
            {
                state.page -= 1;
                show_page(&ui, state);
            }
        }
    });

    // --- Setlists: create / rename / delete / edit / reorder / play. Every
    // mutation persists immediately (save_setlists) then refreshes the models.

    ui.on_create_setlist({
        let ui_handle = ui.as_weak();
        let setlists = setlists.clone();
        move |name| {
            let ui = ui_handle.unwrap();
            let name = name.trim().to_string();
            if name.is_empty() {
                return;
            }
            setlists.borrow_mut().push(setlist::Setlist {
                name,
                songs: Vec::new(),
            });
            save_setlists(&setlists);
            refresh_setlists(&ui, &setlists);
        }
    });

    // The name applies live as the editor's field is typed in. We deliberately
    // do NOT call `refresh_editor` here: that would push `editing-name` back into
    // the field mid-edit and jump the caret. Empty input is ignored so clearing
    // the field to retype keeps the last good name. Only the list view (which
    // shows the name) needs refreshing.
    ui.on_rename_setlist({
        let ui_handle = ui.as_weak();
        let setlists = setlists.clone();
        move |idx, new_name| {
            let ui = ui_handle.unwrap();
            let new_name = new_name.trim().to_string();
            if new_name.is_empty() {
                return;
            }
            if let Some(sl) = setlists.borrow_mut().get_mut(idx as usize) {
                sl.name = new_name;
            }
            save_setlists(&setlists);
            refresh_setlists(&ui, &setlists);
        }
    });

    ui.on_delete_setlist({
        let ui_handle = ui.as_weak();
        let setlists = setlists.clone();
        move |idx| {
            let ui = ui_handle.unwrap();
            let i = idx as usize;
            {
                let mut lists = setlists.borrow_mut();
                if i < lists.len() {
                    lists.remove(i);
                }
            }
            save_setlists(&setlists);
            refresh_setlists(&ui, &setlists);
        }
    });

    ui.on_edit_setlist({
        let ui_handle = ui.as_weak();
        let setlists = setlists.clone();
        let editing = editing.clone();
        let add_hits = add_hits.clone();
        let book_titles = book_titles.clone();
        move |idx| {
            let ui = ui_handle.unwrap();
            *editing.borrow_mut() = Some(idx as usize);
            add_hits.borrow_mut().clear();
            ui.set_add_results(empty_results());
            refresh_editor(&ui, &setlists, &editing, &book_titles);
        }
    });

    ui.on_close_setlist_editor({
        let ui_handle = ui.as_weak();
        let setlists = setlists.clone();
        let editing = editing.clone();
        let book_titles = book_titles.clone();
        move || {
            let ui = ui_handle.unwrap();
            *editing.borrow_mut() = None;
            refresh_editor(&ui, &setlists, &editing, &book_titles);
        }
    });

    ui.on_play_setlist({
        let ui_handle = ui.as_weak();
        let setlists = setlists.clone();
        let nav = nav.clone();
        let viewer = viewer.clone();
        let book_titles = book_titles.clone();
        move |idx| {
            let ui = ui_handle.unwrap();
            let songs: Vec<Song> = {
                let lists = setlists.borrow();
                let Some(sl) = lists.get(idx as usize) else {
                    return;
                };
                sl.songs
                    .iter()
                    .map(|s| Song {
                        title: s.title.clone(),
                        book: s.book.clone(),
                        file: db::resolve_pdf(&s.book).unwrap_or_default(),
                        page: s.page,
                    })
                    .collect()
            };
            if songs.is_empty() {
                return;
            }
            // Playing an existing setlist, not building one: hide the Add button.
            ui.set_viewer_add_mode(false);
            *nav.borrow_mut() = songs;
            open_result_at(&ui, &nav, &viewer, &book_titles, 0);
        }
    });

    ui.on_setlist_add_search({
        let ui_handle = ui.as_weak();
        let library = library.clone();
        let add_hits = add_hits.clone();
        let book_titles = book_titles.clone();
        move |text| {
            let ui = ui_handle.unwrap();
            let query = text.trim();
            if query.is_empty() {
                add_hits.borrow_mut().clear();
                ui.set_add_results(empty_results());
                return;
            }
            let lib = library.borrow();
            let titles = book_titles.borrow();
            let hits = db::search(&lib, query, SEARCH_LIMIT);
            let items: Vec<SongResult> = hits
                .iter()
                .map(|s| SongResult {
                    title: s.title.clone().into(),
                    subtitle: book_subtitle(&titles, &s.book, s.page).into(),
                })
                .collect();
            ui.set_add_results(Rc::new(VecModel::from(items)).into());
            *add_hits.borrow_mut() = hits.into_iter().cloned().collect();
        }
    });

    ui.on_setlist_add_song({
        let ui_handle = ui.as_weak();
        let setlists = setlists.clone();
        let editing = editing.clone();
        let add_hits = add_hits.clone();
        let book_titles = book_titles.clone();
        move |ri| {
            let ui = ui_handle.unwrap();
            let Some(song) = add_hits.borrow().get(ri as usize).cloned() else {
                return;
            };
            add_song_to_editing(&ui, &setlists, &editing, &book_titles, song);
        }
    });

    ui.on_preview_song({
        let ui_handle = ui.as_weak();
        let add_hits = add_hits.clone();
        let nav = nav.clone();
        let viewer = viewer.clone();
        let book_titles = book_titles.clone();
        move |ri| {
            let ui = ui_handle.unwrap();
            // add_hits are library songs (their PDF is installed), so the viewer
            // can render them straight away. The viewer navigates the whole add
            // search hit list (Prev/Next-result), so you can flip through the
            // candidates to compare before adding — same as the Search tab — and
            // offers an Add button so the chosen one goes straight in.
            ui.set_viewer_add_mode(true);
            *nav.borrow_mut() = add_hits.borrow().clone();
            open_result_at(&ui, &nav, &viewer, &book_titles, ri as usize);
        }
    });

    // Add button inside the viewer (preview-to-add flow): add the song currently
    // on screen — nav[result_idx] — to the setlist being edited, without going
    // back to the list.
    ui.on_viewer_add_song({
        let ui_handle = ui.as_weak();
        let setlists = setlists.clone();
        let editing = editing.clone();
        let nav = nav.clone();
        let viewer = viewer.clone();
        let book_titles = book_titles.clone();
        move || {
            let ui = ui_handle.unwrap();
            let Some(song) = viewer
                .borrow()
                .as_ref()
                .and_then(|state| nav.borrow().get(state.result_idx).cloned())
            else {
                return;
            };
            add_song_to_editing(&ui, &setlists, &editing, &book_titles, song);
            ui.set_viewer_added(true);
        }
    });

    ui.on_setlist_remove_song({
        let ui_handle = ui.as_weak();
        let setlists = setlists.clone();
        let editing = editing.clone();
        let book_titles = book_titles.clone();
        move |i| {
            let ui = ui_handle.unwrap();
            let Some(idx) = *editing.borrow() else {
                return;
            };
            if let Some(sl) = setlists.borrow_mut().get_mut(idx) {
                let j = i as usize;
                if j < sl.songs.len() {
                    sl.songs.remove(j);
                }
            }
            save_setlists(&setlists);
            refresh_editor(&ui, &setlists, &editing, &book_titles);
            refresh_setlists(&ui, &setlists);
        }
    });

    ui.on_setlist_move_up({
        let ui_handle = ui.as_weak();
        let setlists = setlists.clone();
        let editing = editing.clone();
        let book_titles = book_titles.clone();
        move |i| {
            let ui = ui_handle.unwrap();
            let Some(idx) = *editing.borrow() else {
                return;
            };
            if let Some(sl) = setlists.borrow_mut().get_mut(idx) {
                setlist::move_up(&mut sl.songs, i as usize);
            }
            save_setlists(&setlists);
            refresh_editor(&ui, &setlists, &editing, &book_titles);
        }
    });

    ui.on_setlist_move_down({
        let ui_handle = ui.as_weak();
        let setlists = setlists.clone();
        let editing = editing.clone();
        let book_titles = book_titles.clone();
        move |i| {
            let ui = ui_handle.unwrap();
            let Some(idx) = *editing.borrow() else {
                return;
            };
            if let Some(sl) = setlists.borrow_mut().get_mut(idx) {
                setlist::move_down(&mut sl.songs, i as usize);
            }
            save_setlists(&setlists);
            refresh_editor(&ui, &setlists, &editing, &book_titles);
        }
    });

    ui.on_close_viewer({
        let ui_handle = ui.as_weak();
        let viewer = viewer.clone();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_viewer_visible(false);
            ui.set_page_image(Image::default());
            ui.set_page_annotations(Rc::new(VecModel::<AnnotationItem>::default()).into());
            *viewer.borrow_mut() = None;
        }
    });

    log::debug!("calling run");
    ui.run()?;
    Ok(())
}
