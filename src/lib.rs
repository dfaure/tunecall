// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::cell::RefCell;
use std::error::Error;
use std::path::PathBuf;
use std::rc::Rc;

use slint::{Image, VecModel};

mod config;
mod db;
mod index;
mod pdf;
mod storage;

// Include the slint-generated code
slint::include_modules!();

/// Width, in pixels, that PDF pages are rasterized to. The UI scales the result
/// to fit, so this is really a quality/sharpness knob.
const RENDER_WIDTH: i32 = 1200;

/// Max search results shown at once.
const SEARCH_LIMIT: i64 = 300;

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
            "/storage/emulated/0/Download/jambook_log.txt",
        )?)
        .format(flexi_logger::detailed_format)
        .start()?;

    log::info!("jambook started");
    slint::android::init(app).unwrap();
    log::debug!("slint::android initialized");
    let ret = jambook_main();
    if let Err(ref e) = ret {
        log::error!("{:?}", e);
    }
    // When we get here, exit process so Android restarts fresh next time
    std::process::exit(0);
}

/// Locate `MasterIndex.PDF` inside the PDF folder (case-insensitive).
fn master_index_path() -> Option<PathBuf> {
    let dir = storage::pdf_dir();
    std::fs::read_dir(&dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.eq_ignore_ascii_case("MasterIndex.PDF"))
        })
}

/// Parse the master index and (re)build the song table. Returns the song count.
fn import_index(config: &config::BooksConfig) -> anyhow::Result<usize> {
    let path = master_index_path().ok_or_else(|| {
        anyhow::anyhow!(
            "MasterIndex.PDF not found in {}",
            storage::pdf_dir().display()
        )
    })?;
    let lines = pdf::all_text_lines(&path.to_string_lossy())?;
    let entries = index::parse_master_index(&lines, &config.codes());
    let n = db::replace_songs(&entries)?;
    Ok(n)
}

/// Status line shown when no search is active.
fn set_idle_status(ui: &AppWindow) {
    match db::song_count() {
        Ok(0) => ui.set_status(
            format!(
                "No songs indexed. Put MasterIndex.PDF in\n{}\nand press Reimport.",
                storage::pdf_dir().display()
            )
            .into(),
        ),
        Ok(n) => ui.set_status(format!("{n} songs indexed. Type to search.").into()),
        Err(e) => ui.set_status(format!("Database error: {e}").into()),
    }
}

/// 0-based pdfium page for a printed page label, given the book's first_page.
/// Non-numeric labels (appendix pages like "A1") fall back to the book start.
fn pdfium_page_0based(printed_page: &str, first_page: i32) -> i32 {
    let first0 = (first_page - 1).max(0);
    match printed_page.parse::<i32>() {
        Ok(p) => first0 + (p - 1).max(0),
        Err(_) => first0,
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

pub fn jambook_main() -> Result<(), Box<dyn Error>> {
    std::panic::set_hook(Box::new(|info| {
        log::error!("Panic occurred: {}", info);
    }));

    let ui = AppWindow::new()?;
    let config = Rc::new(config::load_or_create().unwrap_or_else(|e| {
        log::error!("config load failed: {e}");
        config::BooksConfig::default()
    }));
    // Current search results, kept in Rust so a click can map back to the song.
    let results: Rc<RefCell<Vec<db::Song>>> = Rc::new(RefCell::new(Vec::new()));
    let viewer: Rc<RefCell<Option<ViewerState>>> = Rc::new(RefCell::new(None));

    // First-run import.
    if db::song_count().unwrap_or(0) == 0 {
        match import_index(&config) {
            Ok(n) => log::info!("imported {n} songs from the master index"),
            Err(e) => log::warn!("initial import failed: {e}"),
        }
    }
    set_idle_status(&ui);

    ui.on_search({
        let ui_handle = ui.as_weak();
        let results = results.clone();
        move |text| {
            let ui = ui_handle.unwrap();
            let query = text.trim();
            if query.is_empty() {
                results.borrow_mut().clear();
                ui.set_results(empty_results());
                set_idle_status(&ui);
                return;
            }
            match db::search_songs(query, SEARCH_LIMIT) {
                Ok(found) => {
                    let items: Vec<SongResult> = found
                        .iter()
                        .map(|s| SongResult {
                            title: s.title.clone().into(),
                            subtitle: format!("{} · p.{}", s.book_code, s.printed_page).into(),
                        })
                        .collect();
                    let n = items.len();
                    ui.set_results(Rc::new(VecModel::from(items)).into());
                    ui.set_status(if n == 0 {
                        "No results".into()
                    } else {
                        format!("{n} result(s)").into()
                    });
                    *results.borrow_mut() = found;
                }
                Err(e) => {
                    log::warn!("search failed: {e}");
                    ui.set_status(format!("Search error: {e}").into());
                }
            }
        }
    });

    ui.on_open_result({
        let ui_handle = ui.as_weak();
        let results = results.clone();
        let config = config.clone();
        let viewer = viewer.clone();
        move |idx| {
            let ui = ui_handle.unwrap();
            // Copy out the song so we can drop the borrow before doing PDF work.
            let Some((title, code, printed_page)) = results
                .borrow()
                .get(idx as usize)
                .map(|s| (s.title.clone(), s.book_code.clone(), s.printed_page.clone()))
            else {
                return;
            };

            let Some(book) = config.get(&code) else {
                ui.set_status(
                    format!(
                        "No mapping for book '{code}'. Edit {}",
                        config::config_path().display()
                    )
                    .into(),
                );
                return;
            };

            let path = storage::pdf_dir()
                .join(&book.file)
                .to_string_lossy()
                .into_owned();
            log::info!("opening '{title}' -> {code} p.{printed_page} in {path}");
            let count = pdf::page_count(&path).unwrap_or(0);
            let target = pdfium_page_0based(&printed_page, book.first_page);
            let page = target.clamp(0, count.saturating_sub(1) as i32) as u16;

            ui.set_viewer_title(format!("{title} — {code} (p.{printed_page})").into());
            let state = ViewerState { path, page, count };
            show_page(&ui, &state);
            *viewer.borrow_mut() = Some(state);
            ui.set_viewer_visible(true);
        }
    });

    ui.on_reimport({
        let ui_handle = ui.as_weak();
        let results = results.clone();
        let config = config.clone();
        move || {
            let ui = ui_handle.unwrap();
            results.borrow_mut().clear();
            ui.set_results(empty_results());
            match import_index(&config) {
                Ok(n) => {
                    log::info!("reimported {n} songs");
                    ui.set_status(format!("Imported {n} songs from the master index.").into());
                }
                Err(e) => {
                    log::warn!("reimport failed: {e}");
                    ui.set_status(format!("Import failed: {e}").into());
                }
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
