extern crate bindgen;
extern crate cc;

use std::env;
use std::path::PathBuf;

fn sdk_path(target: &str) -> Result<Option<String>, std::io::Error> {
    use std::process::Command;

    let sdk = if target.contains("apple-darwin") {
        "macosx"
    } else if target == "x86_64-apple-ios" || target == "i386-apple-ios" {
        "iphonesimulator"
    } else if target == "aarch64-apple-ios" {
        "iphoneos"
    } else {
        return Ok(None);
    };

    let output = Command::new("xcrun")
        .args(&["--sdk", sdk, "--show-sdk-path"])
        .output()?
        .stdout;
    let prefix_str = std::str::from_utf8(&output).expect("invalid output from `xcrun`");
    Ok(Some(prefix_str.trim_end().to_string()))
}

fn main() {
    let mut build = cc::Build::new();
    build.file("CRoaring/roaring.c");

    if cfg!(feature = "compat") {
        build.define("DISABLEAVX", Some("1"));
    } else {
        if let Ok(target_arch) = env::var("ROARING_ARCH") {
            build.flag_if_supported(&format!("-march={}", target_arch));
        } else {
            build.flag_if_supported("-march=native");
        }
    }

    build.compile("libroaring.a");

    let target = std::env::var("TARGET").unwrap();

    let mut builder = bindgen::Builder::default()
        .blacklist_type("max_align_t")
        .header("CRoaring/roaring.h")
        .generate_inline_functions(true);
    if let Some(sdk_path) = sdk_path(&target).unwrap() {
        builder = builder.clang_args(&["-isysroot", &sdk_path]);
    }
    if target == "aarch64-linux-android" {
        // disable NEON for aarch64-linux-android
        builder = builder.clang_arg("-DDISABLENEON");
    }

    let bindings = builder.generate().expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("croaring-sys.rs"))
        .expect("Couldn't write bindings!");
}
