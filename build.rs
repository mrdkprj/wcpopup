use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo::rerun-if-changed=build.rs");

    if cfg!(feature = "webview2") {
        let dll_src = PathBuf::from(std::env::var("DEP_WCPOPUPHOOK_DLL").unwrap());

        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let dll_dest = out_dir.join("win_hook.dll");

        let msg = format!("Failed to copy DLL:{}", out_dir.to_str().unwrap());
        fs::copy(&dll_src, &dll_dest).unwrap_or_else(|_| panic!("{}", msg));

        println!("cargo:rustc-env=WIN_HOOK_DLL={}", dll_dest.display());
    }
}
