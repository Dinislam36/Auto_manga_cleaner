use std::io;
use std::path::PathBuf;

fn main() -> io::Result<()> {
    let output_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());

    std::fs::copy(manifest_dir.join("src/resource/MangaiClean.lib"), output_dir.join("resource.lib"))?;

    println!("cargo:rustc-link-search=native={}", output_dir.to_str().unwrap());

    if version_check::is_min_version("1.61.0").unwrap_or(true) {
        println!("cargo:rustc-link-lib=static:+whole-archive=resource");
    } else {
        println!("cargo:rustc-link-lib=static=resource");
    }

    Ok(())
}