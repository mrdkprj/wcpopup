use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let dll_src = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "\\win_hook.dll"));
    let dll_dest = out_dir.join("project_b.dll");

    let msg = format!("Failed to copy DLL:{}", out_dir.to_str().unwrap());
    fs::copy(&dll_src, &dll_dest).unwrap_or_else(|_| panic!("{}", msg));

    println!("cargo:rustc-env=PROJECT_B_DLL={}", dll_dest.display());
}
