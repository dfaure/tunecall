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

    // The UI uses the Material Components library (vendored in material-1.0/),
    // registered here under the "@material" namespace. This replaces the old,
    // now-deprecated built-in "material" widget style: that styled the
    // std-widgets, whereas the library is a richer Material 3 component set
    // imported directly in the .slint files. See material-1.0/README.md.
    //
    // No widget style is set: the only std-widget still used is ListView (kept
    // for its row virtualization, which the library's ScrollView-based ListView
    // lacks), so it falls back to the default style — only its scrollbar shows.
    let config = slint_build::CompilerConfiguration::new().with_library_paths(
        std::collections::HashMap::from([(
            "material".to_string(),
            std::path::Path::new(&std::env::var_os("CARGO_MANIFEST_DIR").unwrap())
                .join("material-1.0/material.slint"),
        )]),
    );

    slint_build::compile_with_config("ui/app-window.slint", config).expect("Slint build failed");
}
