// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use slint::{Image, VecModel};

mod db;
mod pdf;
mod storage;

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
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: slint::android::AndroidApp) -> Result<(), Box<dyn Error>> {
    // Resolve the app-specific storage directory before anything touches it.
    if let Some(dir) = app
        .external_data_path()
        .or_else(|| app.internal_data_path())
    {
        storage::set_data_dir(dir);
    }

    // Log to file, on Android
    flexi_logger::Logger::try_with_env_or_str("debug,android_activity::activity_impl::glue=off")?
        .log_to_file(flexi_logger::FileSpec::try_from(
            "/storage/emulated/0/Download/tunecall_log.txt",
        )?)
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

pub fn tunecall_main() -> Result<(), Box<dyn Error>> {
    std::panic::set_hook(Box::new(|info| {
        log::error!("Panic occurred: {}", info);
    }));

    let ui = AppWindow::new()?;
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
            let Some(song) = results.borrow().get(idx as usize).cloned() else {
                return;
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
            let state = ViewerState { path, page, count };
            show_page(&ui, &state);
            *viewer.borrow_mut() = Some(state);
            ui.set_viewer_visible(true);
        }
    });

    ui.on_reload({
        let ui_handle = ui.as_weak();
        let library = library.clone();
        let results = results.clone();
        move || {
            let ui = ui_handle.unwrap();
            results.borrow_mut().clear();
            ui.set_results(empty_results());
            reload_library(&ui, &library);
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
