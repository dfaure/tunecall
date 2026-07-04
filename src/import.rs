//! One-shot "Import from folder…" button. Android-only: pops the Storage
//! Access Framework folder picker, then copies every `.pdf` in the chosen tree
//! into `pdf_dir`.
//!
//! Reaching SAF from a NativeActivity means all the picker plumbing lives in a
//! small helper Activity (`app/src/main/java/com/tunecall/app/Import.java`):
//! it launches `ACTION_OPEN_DOCUMENT_TREE`, receives `onActivityResult`, and
//! iterates the tree via `DocumentsContract` to copy files into
//! `getExternalFilesDir(null)/pdfs` (the same path Rust calls `pdf_dir`). When
//! it's done it invokes [`Java_com_tunecall_app_Import_nativeOnImported`],
//! which reaches back to `lib.rs` via a stashed callback so the library can
//! rescan on the Slint event loop.
//!
//! No storage permission is required: the URI grant comes from the user's
//! picker choice, and we don't persist it (this is a one-off import, not a
//! sync source). Nothing here compiles on desktop — the Books-tab button is
//! hidden there via `import-supported`.

use std::sync::Mutex;

/// Fired when the Java helper finishes the copy. Argument is the number of
/// PDFs successfully imported (or `-1` on picker/launch failure).
type Callback = Box<dyn Fn(i32) + Send + 'static>;

static ON_IMPORTED: Mutex<Option<Callback>> = Mutex::new(None);

/// Install the callback fired when the SAF import finishes. Called once from
/// startup with a closure that hops to the Slint event loop and refreshes the
/// library.
pub fn set_on_imported(cb: Callback) {
    *ON_IMPORTED.lock().unwrap() = Some(cb);
}

/// Launch the Android SAF folder picker via the Java helper. No-op on desktop.
#[cfg(target_os = "android")]
pub fn launch_folder_picker(app: &slint::android::AndroidApp) {
    if let Err(e) = try_launch(app) {
        log::warn!("launching folder picker failed: {e:?}");
        if let Some(cb) = ON_IMPORTED.lock().unwrap().as_ref() {
            cb(-1);
        }
    }
}

#[cfg(target_os = "android")]
fn try_launch(app: &slint::android::AndroidApp) -> Result<(), Box<dyn std::error::Error>> {
    use jni::JavaVM;
    use jni::objects::{JClass, JObject, JValue};

    // Safety: the pointers come from android-activity's AndroidApp, as
    // documented — same as the Immersive helper.
    let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast())? };
    let mut env = vm.attach_current_thread()?;
    let activity = unsafe { JObject::from_raw(app.activity_as_ptr().cast()) };

    // FindClass on the native thread can't see app classes (system class
    // loader); load Import through the activity's own class loader.
    let loader = env
        .call_method(
            &activity,
            "getClassLoader",
            "()Ljava/lang/ClassLoader;",
            &[],
        )?
        .l()?;
    let name: JObject = env.new_string("com.tunecall.app.Import")?.into();
    let class: JClass = env
        .call_method(
            &loader,
            "loadClass",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            &[JValue::Object(&name)],
        )?
        .l()?
        .into();

    env.call_static_method(
        &class,
        "launchPicker",
        "(Landroid/app/Activity;)V",
        &[JValue::Object(&activity)],
    )?;
    Ok(())
}

/// Called from `Import.java` on the copy thread once the import finishes.
/// Hands off to the stashed callback (which marshals back to the UI thread).
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_tunecall_app_Import_nativeOnImported(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
    count: jni::sys::jint,
) {
    log::info!("SAF import finished: {count} PDF(s)");
    if let Some(cb) = ON_IMPORTED.lock().unwrap().as_ref() {
        cb(count);
    }
}
