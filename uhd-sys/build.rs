extern crate bindgen;
extern crate metadeps;

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=CC");

    // This reads the metadata in Cargo.toml and sends Cargo the appropriate output to link the
    // libraries
    let libraries = metadeps::probe().unwrap();

    let uhd_include_path = libraries
        .get("uhd")
        .expect("uhd library not in map")
        .include_paths
        .first()
        .expect("no include path for UHD headers");
    generate_bindings(uhd_include_path);
}

fn generate_bindings(include_path: &Path) {
    let usrp_header = include_path.join("uhd.h");

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let out_path = out_dir.join("bindgen.rs");

    let mut builder = bindgen::builder()
        .whitelist_function("^uhd.+")
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .header(usrp_header.to_string_lossy().clone())
        // Add the include directory to ensure that #includes in the header work correctly
        .clang_arg(format!("-I{}", include_path.to_string_lossy().clone()));

    let target = env::var("TARGET").expect("No TARGET environment variable");
    if target.contains("linux") {
        if let Some(system_include_path) = detect_system_include_path() {
            builder = builder.clang_arg(format!("-I{}", system_include_path.display()));
        }
    } else if target == "aarch64-apple-darwin" {
        // On macOS Apple Silicon, boost libs from `brew install boost` are at this path
        println!("cargo:rustc-link-search=/opt/homebrew/lib/");
    }

    let bindings = builder.generate().expect("Failed to generate bindings");
    bindings
        .write_to_file(out_path)
        .expect("Failed to write bindings to file");
}

fn detect_system_include_path() -> Option<PathBuf> {
    let compiler = env::var("CC").unwrap_or_else(|_| String::from("cc"));
    let output = Command::new(&compiler)
        .arg("-print-file-name=include")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let include_dir = String::from_utf8(output.stdout).ok()?;
    let include_dir = include_dir.trim();
    if include_dir.is_empty() {
        return None;
    }

    let include_dir = PathBuf::from(include_dir);
    if include_dir.join("stddef.h").exists() {
        Some(include_dir)
    } else {
        None
    }
}
