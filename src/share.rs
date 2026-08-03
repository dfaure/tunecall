//! Android share-intent bridge. Hands a file to `Intent.ACTION_SEND` via a small
//! Java helper (`app/src/main/java/com/tunecall/app/Share.java`), so the user
//! can send the exported page to any share target (Gmail, Signal, Drive, …).
//!
//! Same JNI pattern as [`crate::immersive`] and [`crate::import`]: reach through
//! the activity's class loader (FindClass on the native thread can only see the
//! system loader), then call a static method. No callback — the chooser owns
//! the flow from here.
//!
//! Desktop: this module is empty; the Slint side hides the share button off
//! Android, and the Rust caller stubs it out.

#[cfg(target_os = "android")]
pub fn share_pdf(path: &std::path::Path, subject: &str) -> Result<(), Box<dyn std::error::Error>> {
    use jni::JavaVM;
    use jni::objects::{JClass, JObject, JValue};

    let app = crate::immersive::app().ok_or("share: app handle not set")?;
    // Safety: the pointers come from android-activity's AndroidApp, as
    // documented — same as Immersive / Import.
    let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast())? };
    let mut env = vm.attach_current_thread()?;
    let activity = unsafe { JObject::from_raw(app.activity_as_ptr().cast()) };

    let loader = env
        .call_method(
            &activity,
            "getClassLoader",
            "()Ljava/lang/ClassLoader;",
            &[],
        )?
        .l()?;
    let name: JObject = env.new_string("com.tunecall.app.Share")?.into();
    let class: JClass = env
        .call_method(
            &loader,
            "loadClass",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            &[JValue::Object(&name)],
        )?
        .l()?
        .into();

    let path_str = path.to_str().ok_or("share: file path is not valid UTF-8")?;
    let j_path: JObject = env.new_string(path_str)?.into();
    let j_subject: JObject = env.new_string(subject)?.into();

    env.call_static_method(
        &class,
        "sharePdf",
        "(Landroid/app/Activity;Ljava/lang/String;Ljava/lang/String;)V",
        &[
            JValue::Object(&activity),
            JValue::Object(&j_path),
            JValue::Object(&j_subject),
        ],
    )?;
    Ok(())
}
