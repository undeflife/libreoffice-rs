use bindgen;

use std::env;
use std::path::Path;
use std::process::Command;

// perform make with argument
fn make() {
    let include_path = env::var("C_INCLUDE_PATH").unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();
    Command::new("gcc")
        .args(&[
            "wrapper.c",
            "-c",
            "-fPIC",
            &format!("-I{}", include_path),
            "-o",
        ])
        .arg(&format!("{}/wrapper.o", out_dir))
        .status()
        .unwrap();
    Command::new("ar")
        .args(&["crus", "libwrapper.a", "wrapper.o"])
        .current_dir(&Path::new(&out_dir))
        .status()
        .unwrap();
    println!("cargo:rustc-link-search=native={}", out_dir);
}

fn generate_binding() {
    let lo_include_path = std::env::var("LO_INCLUDE_PATH").ok();
    if lo_include_path.is_none() {
        panic!("no LO_INCLUDE_PATH found");
    }
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("-Wall")
        .layout_tests(false)
        .clang_arg(format!("-I{}", lo_include_path.unwrap()))
        .blocklist_function("lok_init_wrapper")
        .disable_header_comment()
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file("src/wrapper/bindings.rs")
        .expect("Couldn't write bindings!");
}

fn main() {
    make();
    generate_binding();
    println!("cargo:rustc-link-lib=static=wrapper");
}
