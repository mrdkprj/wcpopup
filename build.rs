use std::{env, path::PathBuf};

fn main() {
    println!("cargo::rerun-if-changed=build.rs");

    if cfg!(feature = "webview") && cfg!(windows) {
        let dll_src = PathBuf::from(env::var("DEP_WCPOPUPHOOK_DLL").unwrap());
        println!("cargo:rustc-env=WIN_HOOK_DLL={}", dll_src.display());
    }
}
