// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use slint::{Image, VecModel};

mod db;
mod pdf;
mod storage;
mod sync;

use db::Song;

// Include the slint-generated code
slint::include_modules!();

/// Width, in pixels, that PDF pages are rasterized to. The UI scales the result
/// to fit, so this is really a quality/sharpness knob.
const RENDER_WIDTH: i32 = 1200;

/// Max search results shown at once.
const SEARCH_LIMIT: usize = 300;

/// What's currently shown in the viewer.
struct ViewerState {
    path: String,
    page: u16,
    count: u16,
    /// Index into the current search results, so Prev/Next result can find the
    /// neighboring song.
    result_idx: usize,
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

    // Log to a file inside the app data dir (shared storage like Download/ is
    // not writable on modern Android without extra permissions).
    flexi_logger::Logger::try_with_env_or_str("debug,android_activity::activity_impl::glue=off")?
        .log_to_file(
            flexi_logger::FileSpec::default()
                .directory(storage::data_dir())
                .basename("tunecall"),
        )
        .format(flexi_logger::detailed_format)
        .start()?;

    log::info!("tunecall started");
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
    let n = library.borrow().len();
    if n == 0 {
        ui.set_status(
            format!(
                "No song indexes found in\n{}\nGenerate <book>.db files with the indexer.",
                storage::pdf_dir().display()
            )
            .into(),
        );
    } else {
        ui.set_status(format!("{n} songs indexed. Type to search.").into());
    }
}

/// Render the page described by `state` into the viewer.
fn show_page(ui: &AppWindow, state: &ViewerState) {
    ui.set_viewer_error("".into());
    ui.set_page_number(state.page as i32 + 1);
    ui.set_page_count(state.count as i32);
    match pdf::render_page(&state.path, state.page, RENDER_WIDTH) {
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

/// Open the search result at `idx` in the viewer. Both opening from the results
/// list and stepping Prev/Next across results funnel through here.
fn open_result_at(
    ui: &AppWindow,
    results: &Rc<RefCell<Vec<Song>>>,
    viewer: &Rc<RefCell<Option<ViewerState>>>,
    idx: usize,
) {
    let (song, total) = {
        let results = results.borrow();
        let Some(song) = results.get(idx).cloned() else {
            return;
        };
        (song, results.len())
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

    // Show only the book (PDF) name: the song title is in the page itself,
    // and would be wrong once the user pages Prev/Next to another song.
    ui.set_viewer_title(song.book.into());
    ui.set_result_index(idx as i32);
    ui.set_result_count(total as i32);
    let state = ViewerState {
        path,
        page,
        count,
        result_idx: idx,
    };
    show_page(ui, &state);
    *viewer.borrow_mut() = Some(state);
    ui.set_viewer_visible(true);
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

    reload_library(&ui, &library);

    ui.on_search({
        let ui_handle = ui.as_weak();
        let library = library.clone();
        let results = results.clone();
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
            let hits = db::search(&lib, query, SEARCH_LIMIT);
            let items: Vec<SongResult> = hits
                .iter()
                .map(|s| SongResult {
                    title: s.title.clone().into(),
                    subtitle: s.book.clone().into(),
                })
                .collect();
            let n = items.len();
            ui.set_results(Rc::new(VecModel::from(items)).into());
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
        let viewer = viewer.clone();
        move |idx| {
            let ui = ui_handle.unwrap();
            open_result_at(&ui, &results, &viewer, idx as usize);
        }
    });

    ui.on_next_result({
        let ui_handle = ui.as_weak();
        let results = results.clone();
        let viewer = viewer.clone();
        move || {
            let ui = ui_handle.unwrap();
            let next = match viewer.borrow().as_ref() {
                Some(state) if state.result_idx + 1 < results.borrow().len() => {
                    Some(state.result_idx + 1)
                }
                _ => None,
            };
            if let Some(idx) = next {
                open_result_at(&ui, &results, &viewer, idx);
            }
        }
    });

    ui.on_prev_result({
        let ui_handle = ui.as_weak();
        let results = results.clone();
        let viewer = viewer.clone();
        move || {
            let ui = ui_handle.unwrap();
            let prev = match viewer.borrow().as_ref() {
                Some(state) if state.result_idx > 0 => Some(state.result_idx - 1),
                _ => None,
            };
            if let Some(idx) = prev {
                open_result_at(&ui, &results, &viewer, idx);
            }
        }
    });

    // Reload = download the published indexes from the server, then reload from
    // disk. Runs on the Slint event loop (async-compat bridges reqwest's tokio),
    // so it can touch the Rc state directly without blocking the UI.
    ui.on_reload({
        let ui_handle = ui.as_weak();
        let library = library.clone();
        let results = results.clone();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_status("Downloading indexes…".into());
            let ui_handle = ui_handle.clone();
            let library = library.clone();
            let results = results.clone();
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
                refresh_books(&ui);
                // Report the download error last, so it isn't overwritten by the
                // idle status `reload_library` sets on success.
                if let Some(e) = download_err {
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

    // Books = every indexed book, marking whether its PDF is installed. PDFs are
    // not bundled (copyright), so this tells the user which PDFs they can install
    // to enable a book. A setup-time aid, hence its own simple overlay.
    ui.on_show_books({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            refresh_books(&ui);
            ui.set_books_visible(true);
        }
    });

    ui.on_close_books({
        let ui_handle = ui.as_weak();
        move || {
            ui_handle.unwrap().set_books_visible(false);
        }
    });

    ui.on_close_viewer({
        let ui_handle = ui.as_weak();
        let viewer = viewer.clone();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_viewer_visible(false);
            ui.set_page_image(Image::default());
            *viewer.borrow_mut() = None;
        }
    });

    log::debug!("calling run");
    ui.run()?;
    Ok(())
}
