use std::process::Command;

fn main() {
    // Short commit hash for the About line, captured at build time. Falls back to
    // "unknown" when git isn't available (e.g. a source tarball). The trailing
    // "+" marks a dirty working tree so a screenshot can't be mistaken for a
    // clean build.
    let hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".into());
    let dirty = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .is_some_and(|o| !o.stdout.is_empty());
    println!(
        "cargo:rustc-env=GIT_HASH={hash}{}",
        if dirty { "+" } else { "" }
    );
    // Rebuild when HEAD moves so the hash stays current.
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads");

    let config = slint_build::CompilerConfiguration::new().with_style("material".into());

    // material-light: too big
    // fluent-light: clean, very square, blue highlight below the lineedit, blue selection
    // cupertino-light: even smaller, clean too, blue highlight around the lineedit
    // cosmic-light: too gray
    //
    // Note that changing the config rebuilds many things.
    //
    // To avoid editing this file, you can comment out with_style and set the env var SLINT_STYLE
    // instead, on Linux (not an option on Android...). But it still requires rebuilding (i.e. set
    // SLINT_STYLE when calling `cargo run`).

    slint_build::compile_with_config("ui/app-window.slint", config).expect("Slint build failed");
}
