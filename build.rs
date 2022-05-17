// use bindgen;

use std::env;
use std::io::Result;
use std::process::Command;
use std::process::Output;

// perform make with argument
fn make(arg: &str) -> Result<Output> {
    let current_path = env::current_dir().unwrap();
    let path_name = format!("{}", current_path.display());
    println!("executing make command at {}", path_name);
    let result = Command::new("make")
        .args(&[arg])
        .current_dir(path_name)
        .output();

    match result {
        Err(e) => {
            return Err(e);
        }

        Ok(output) => {
            println!("status: {}", output.status);
            println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            return Ok(output);
        }
    }
}

fn configure() -> Result<Output> {
    make("all")
}
// fn generate_binding() {
//     let lo_include_path = std::env::var("LO_INCLUDE_PATH").ok();
//     if lo_include_path.is_none() {
//         panic!("no LO_INCLUDE_PATH found");
//     }
//     let bindings = bindgen::Builder::default()
//         .header("wrapper.h")
//         .clang_arg("-Wall")
//         .layout_tests(false)
//         .clang_arg(format!("-I{}", lo_include_path.unwrap()))
//         .blocklist_function("lok_init_wrapper")
//         .disable_header_comment()
//         .generate()
//         .expect("Unable to generate bindings");

//     bindings
//         .write_to_file("src/wrapper/bindings.rs")
//         .expect("Couldn't write bindings!");
// }

fn main() {
    let _ = configure();
    // generate_binding();
    println!("cargo:rustc-link-search=all=./");
    println!("cargo:rustc-link-lib=static=wrapper");
}
