//! Immersive fullscreen toggle for Android: hides the system bars (status and
//! navigation) while the PDF viewer is open so the page gets the whole screen,
//! and restores them otherwise. Slint's android backend exposes no fullscreen
//! API, and hiding the bars must run on the UI thread, so we call a small Java
//! helper (`app/src/main/java/com/tunecall/app/Immersive.java`) over JNI.
//!
//! Hiding the bars also drops the backend's reported safe-area insets to 0, so
//! the viewer's existing inset padding reclaims the freed space on its own — no
//! viewer layout change is needed here.

use std::sync::OnceLock;

use jni::JavaVM;
use jni::objects::{JClass, JObject, JValue};

/// The Android app handle, stashed from `android_main`; needed to reach the
/// JavaVM and the activity for JNI calls.
static APP: OnceLock<slint::android::AndroidApp> = OnceLock::new();

/// Remember the app handle so [`set`] can make JNI calls later.
pub fn init(app: slint::android::AndroidApp) {
    let _ = APP.set(app);
}

/// Enter (`true`) or leave (`false`) immersive fullscreen. Errors are logged,
/// not propagated: failing to toggle a bar must never take down the viewer.
pub fn set(enabled: bool) {
    if let Err(e) = try_set(enabled) {
        log::warn!("immersive set({enabled}) failed: {e:?}");
    }
}

fn try_set(enabled: bool) -> Result<(), Box<dyn std::error::Error>> {
    let app = APP.get().ok_or("immersive: app handle not set")?;
    // Safety: the pointers come from android-activity's AndroidApp, as documented.
    let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast())? };
    let mut env = vm.attach_current_thread()?;
    let activity = unsafe { JObject::from_raw(app.activity_as_ptr().cast()) };

    // FindClass on the native thread can't see app classes (it uses the system
    // class loader), so load Immersive through the activity's own class loader.
    let loader = env
        .call_method(&activity, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])?
        .l()?;
    let name: JObject = env.new_string("com.tunecall.app.Immersive")?.into();
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
        "setImmersive",
        "(Landroid/app/Activity;Z)V",
        &[JValue::Object(&activity), JValue::Bool(enabled as u8)],
    )?;
    Ok(())
}
