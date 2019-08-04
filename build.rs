use std::env;

pub fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let cargo_web = env::var("COMPILING_UNDER_CARGO_WEB").unwrap_or_default();
    if target_os == "emscripten" || cargo_web == "1" {
        println!("cargo:rustc-cfg=stdweb");
    }
}
