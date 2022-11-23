use std::io;
use std::path::PathBuf;

fn main() -> io::Result<()> {
    let output_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());

    // this is HOST os, not TARGET
    // for some reason linking the .lib is not working on windows and I don't really care why, use the winresource crate
    // TODO: make the winresource crate work on linux
    if cfg!(windows) {
        winresource::WindowsResource::new()
            .append_rc_content(
                &std::fs::read_to_string(manifest_dir.join("src/resource/MangaiClean_utf8.rc"))
                    .unwrap(),
            )
            .compile()?;
    } else {
        std::fs::copy(
            manifest_dir.join("src/resource/MangaiClean.lib"),
            output_dir.join("resource.lib"),
        )?;

        println!(
            "cargo:rustc-link-search=native={}",
            output_dir.to_str().unwrap()
        );

        if version_check::is_min_version("1.61.0").unwrap_or(true) {
            println!("cargo:rustc-link-lib=static:+whole-archive=resource");
        } else {
            println!("cargo:rustc-link-lib=static=resource");
        }
    }

    Ok(())
}
