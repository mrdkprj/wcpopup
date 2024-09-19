#[cfg(target_os = "linux")]
#[path = "gtk/mod.rs"]
pub(crate) mod platform_impl;
#[cfg(target_os = "windows")]
#[path = "windows/mod.rs"]
pub(crate) mod platform_impl;
