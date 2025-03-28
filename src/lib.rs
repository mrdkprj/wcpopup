//! Rust context menu for Windows and Linux(Gtk3).
//!
//! Supports dark/light theme and color/size/font configurations.
//! - Colors([`config::ColorScheme`])
//!     - Text color
//!     - Background color
//!     - Border color
//! - Size([`config::MenuSize`])
//!     - Menu padding
//!     - Menu item padding
//! - Font([`config::MenuFont`])
//!     - Font family
//!     - Size and weight
//!
//! ## Example
//!
//! Use ManuBuilder to create a Menu with MenuItems.
//!
//! ```no_run
//! fn example(window_handle: isize) {
//!   let mut builder = MenuBuilder::new(window_handle);
//!   // Using HWND
//!   // let mut builder = MenuBuilder::new_for_hwnd(hwnd);
//!   // Using gtk::ApplicationWindow or gtk::Window
//!   // let mut builder = MenuBuilder::new_for_window(window);
//!
//!   builder.check("menu_item1", "Menu Label 1", true, None);
//!   builder.separator();
//!   builder.text_with_accelerator("menu_item2", "Menu Label 2", None, "Ctrl+P");
//!   builder.text_with_accelerator("menu_item3", "Menu Label 3", None, "F11");
//!   builder.text("menu_item4", "Menu Label 4", None);
//!   builder.separator();
//!   builder.text_with_accelerator("menu_item5", "Menu Label 5", None, "Ctrl+S");
//!   builder.separator();
//!
//!   let mut submenu = builder.submenu("Submenu1", "Submenu", None);
//!   submenu.radio("submenu_item1", "Menu Label 1", "Submenu1", true, None);
//!   submenu.radio("submenu_item2", "Menu Label 2", "Submenu1", false, None);
//!   submenu.build().unwrap();
//!
//!   let menu = builder.build().unwrap();
//! }
//! ```
//!
//! Call Menu.popup_at() to show Menu and receive the selected MenuItem using MenuEvent.
//! ```rust
//! fn show_context_menu(x:i32, y:i32) {
//!     menu.popup_at(x, y);
//! }
//!
//! if let Ok(event) = MenuEvent::receiver().try_recv() {
//!     let selected_menu_item = event.item;
//! }
//! ```
//!
//! Or call Menu.popup_at_async() to show Menu and wait asynchronously for a selected MenuItem.
//! ```rust
//! async fn show_context_menu(x:i32, y:i32) {
//!     let selected_menu_item = menu.popup_at(x, y).await;
//! }
//! ```
//!
//! ## Platform-specific notes
//! #### Windows
//! WebView2 may receive all keyboard input instead of its parent window([#1703](https://github.com/MicrosoftEdge/WebView2Feedback/issues/1703)). You can disable it by either
//! 1. Enabling "webview" feature
//! ```
//! features = ["webview"]
//! ```
//!
//! 2. Enabling Webview2 "msWebView2BrowserHitTransparent" feature
//! ```
//! --enable-features=msWebView2BrowserHitTransparent
//! ```
//!
//! #### Linux
//! Gtk3 is required. MenuItem's text color is applied to SVG icon if the SVG file contains the "symbolic" term as the last component of the file name.
pub mod config;
mod platform;
use std::path::{Path, PathBuf};

use async_std::channel::{unbounded, Receiver, Sender};
#[cfg(target_os = "linux")]
use config::Corner;
use once_cell::sync::{Lazy, OnceCell};
pub use platform::platform_impl::{Menu, MenuBuilder, MenuItem};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq)]
enum ThemeChangeFactor {
    SystemSetting,
    User,
    App,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MenuType {
    Main,
    Submenu,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MenuItemType {
    Text,
    Checkbox,
    Radio,
    Submenu,
    Separator,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub(crate) struct RgbaIcon {
    rgba: Vec<u8>,
    width: u32,
    height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum MenuIconKind {
    Path(PathBuf),
    Rgba(RgbaIcon),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MenuIcon {
    pub(crate) icon: MenuIconKind,
}

impl MenuIcon {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            icon: MenuIconKind::Path(path.as_ref().to_path_buf()),
        }
    }

    pub fn from_rgba(rgba: Vec<u8>, width: u32, height: u32) -> Self {
        Self {
            icon: MenuIconKind::Rgba(RgbaIcon {
                rgba,
                width,
                height,
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct MenuEvent {
    pub item: MenuItem,
}

pub(crate) struct InnerMenuEvent {
    pub(crate) item: Option<MenuItem>,
}

pub type MenuEventReceiver = Receiver<MenuEvent>;
type MenuEventHandler = std::boxed::Box<dyn Fn(MenuEvent) + Send + Sync + 'static>;
type InnerMenuEventReceiver = Receiver<InnerMenuEvent>;

static MENU_CHANNEL: Lazy<(Sender<MenuEvent>, MenuEventReceiver)> = Lazy::new(unbounded);
static INNER_MENU_CHANNEL: Lazy<(Sender<InnerMenuEvent>, InnerMenuEventReceiver)> = Lazy::new(unbounded);
static MENU_EVENT_HANDLER: OnceCell<Option<MenuEventHandler>> = OnceCell::new();

impl MenuEvent {
    pub fn item(&self) -> &MenuItem {
        &self.item
    }

    pub fn receiver<'a>() -> &'a MenuEventReceiver {
        &MENU_CHANNEL.1
    }

    fn innner_receiver<'a>() -> &'a InnerMenuEventReceiver {
        &INNER_MENU_CHANNEL.1
    }

    pub fn set_event_handler<F: Fn(MenuEvent) + Send + Sync + 'static>(f: Option<F>) {
        if let Some(f) = f {
            let _ = MENU_EVENT_HANDLER.set(Some(std::boxed::Box::new(f)));
        } else {
            let _ = MENU_EVENT_HANDLER.set(None);
        }
    }

    fn send(event: MenuEvent) {
        if let Some(handler) = MENU_EVENT_HANDLER.get_or_init(|| None) {
            handler(event);
        } else {
            let _ = MENU_CHANNEL.0.send_blocking(event);
        }
    }

    fn send_inner(event: InnerMenuEvent) {
        let _ = INNER_MENU_CHANNEL.0.send_blocking(event);
    }
}
