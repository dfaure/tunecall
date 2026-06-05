// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use slint::{Image, VecModel};

mod db;
mod pdf;
mod storage;

// Include the slint-generated code
slint::include_modules!();

/// Width, in pixels, that PDF pages are rasterized to. The UI scales the result
/// to fit, so this is really a quality/sharpness knob.
const RENDER_WIDTH: i32 = 1200;

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

/// Reload the PDF list from the DB into the UI model, and remember each row's
/// path (the UI only carries names; paths stay in Rust, indexed by row).
fn reload_list(ui: &AppWindow, paths: &Rc<RefCell<Vec<String>>>) {
    match db::list_pdfs() {
        Ok(entries) => {
            let mut stored = paths.borrow_mut();
            stored.clear();
            let mut items = Vec::with_capacity(entries.len());
            for entry in entries {
                items.push(PdfData {
                    name: entry.name.into(),
                });
                stored.push(entry.path);
            }
            let model: Rc<VecModel<PdfData>> = Rc::new(VecModel::from(items));
            ui.set_pdfs(model.into());
            if stored.is_empty() {
                ui.set_status(
                    format!(
                        "No PDFs found. Drop .pdf files into\n{}",
                        storage::pdf_dir().display()
                    )
                    .into(),
                );
            } else {
                ui.set_status(format!("{} PDF(s)", stored.len()).into());
            }
        }
        Err(e) => {
            log::warn!("list_pdfs failed: {e}");
            ui.set_status(format!("Database error: {e}").into());
        }
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

pub fn jambook_main() -> Result<(), Box<dyn Error>> {
    std::panic::set_hook(Box::new(|info| {
        log::error!("Panic occurred: {}", info);
    }));

    let ui = AppWindow::new()?;
    let paths: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let viewer: Rc<RefCell<Option<ViewerState>>> = Rc::new(RefCell::new(None));

    // Initial scan + populate.
    if let Err(e) = db::scan_and_store() {
        log::warn!("initial scan failed: {e}");
    }
    reload_list(&ui, &paths);

    ui.on_rescan({
        let ui_handle = ui.as_weak();
        let paths = paths.clone();
        move || {
            let ui = ui_handle.unwrap();
            match db::scan_and_store() {
                Ok(n) => log::info!("rescan found {n} PDF(s)"),
                Err(e) => log::warn!("rescan failed: {e}"),
            }
            reload_list(&ui, &paths);
        }
    });

    ui.on_open_pdf({
        let ui_handle = ui.as_weak();
        let paths = paths.clone();
        let viewer = viewer.clone();
        move |index| {
            let ui = ui_handle.unwrap();
            let Some(path) = paths.borrow().get(index as usize).cloned() else {
                return;
            };
            log::info!("opening pdf #{index}: {path}");
            let count = pdf::page_count(&path).unwrap_or(0);
            let state = ViewerState {
                path,
                page: 0,
                count,
            };
            show_page(&ui, &state);
            *viewer.borrow_mut() = Some(state);
            ui.set_viewer_visible(true);
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
