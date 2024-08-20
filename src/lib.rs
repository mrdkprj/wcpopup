//! Context menu for Windows.
//!
//! You can customize text, border, background colors using [`ColorScheme`] and border size, paddings using [`MenuSize`].
//! Theme(Dark/Light/System) is also sopported.
//!
//! ## Example
//!
//! Use ManuBuilder to create a Menu with MenuItems, and then call Menu.popup_at() to show Menu.
//! When a MenuItem is clicked, the selected MenuItem data is returned.
//!
//! ```no_run
//! fn example(window_handle: isize) {
//!   let mut builder = MenuBuilder::new(window_handle);
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
//!
//!   let selected_item = menu.popup_at(100, 100);
//!   // Or popup_at_async
//!   // let selected_item = menu.popup_at_async(100, 100).await
//! }
//! ```
mod accelerator;
mod builder;
mod config;
mod direct2d;
mod menu_item;
mod util;
#[cfg(feature = "accelerator")]
use accelerator::{create_haccel, destroy_haccel};
pub use builder::*;
pub use config::*;
use direct2d::{colorref_to_d2d1_color_f, create_write_factory, get_device_context, get_text_format, set_fill_color, set_stroke_color, to_2d_rect, TextAlignment};
pub use menu_item::*;
use serde::Serialize;
use util::*;

#[cfg(feature = "accelerator")]
use std::rc::Rc;
use std::{
    ffi::c_void,
    mem::{size_of, transmute},
};
#[cfg(feature = "accelerator")]
use windows::Win32::UI::WindowsAndMessaging::{TranslateAcceleratorW, HACCEL, WM_COMMAND, WM_SYSCOMMAND};
use windows::{
    core::{w, Error, PCWSTR},
    Foundation::Numerics::Matrix3x2,
    Win32::{
        Foundation::{HANDLE, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM},
        Graphics::{
            Direct2D::{Common::D2D_POINT_2F, D2D1_DRAW_TEXT_OPTIONS_NONE, D2D1_ROUNDED_RECT},
            DirectWrite::{DWRITE_MEASURING_MODE_NATURAL, DWRITE_TEXT_ALIGNMENT_TRAILING},
            Gdi::{
                BeginPaint, ClientToScreen, EndPaint, GetMonitorInfoW, GetWindowDC, InvalidateRect, MonitorFromPoint, MonitorFromWindow, OffsetRect, PtInRect, ReleaseDC, ScreenToClient, UpdateWindow,
                HBRUSH, HDC, MONITORINFO, MONITOR_DEFAULTTONEAREST, MONITOR_DEFAULTTONULL, PAINTSTRUCT,
            },
        },
        System::{
            LibraryLoader::GetModuleHandleW,
            Threading::{AttachThreadInput, GetCurrentThreadId},
        },
        UI::{
            Input::KeyboardAndMouse::{
                GetCapture, ReleaseCapture, SendInput, SetActiveWindow, SetCapture, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_RIGHTDOWN,
                MOUSEEVENTF_VIRTUALDESK, MOUSEINPUT, VIRTUAL_KEY, VK_ESCAPE, VK_LWIN, VK_RWIN,
            },
            Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
            WindowsAndMessaging::{
                AnimateWindow, CallNextHookEx, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetAncestor, GetClientRect, GetCursorPos, GetMessageW, GetParent, GetPropW, GetWindow, GetWindowRect,
                GetWindowThreadProcessId, IsWindow, IsWindowVisible, KillTimer, LoadCursorW, PostMessageW, PostThreadMessageW, RegisterClassExW, RemovePropW, SetCursor, SetForegroundWindow, SetPropW,
                SetTimer, SetWindowPos, SetWindowsHookExW, ShowWindow, SystemParametersInfoW, TranslateMessage, UnhookWindowsHookEx, WindowFromPoint, AW_BLEND, CS_DROPSHADOW, CS_HREDRAW, CS_VREDRAW,
                GA_ROOTOWNER, GW_OWNER, HCURSOR, HHOOK, HICON, HWND_TOP, IDC_ARROW, MSG, SPI_GETMENUSHOWDELAY, SWP_ASYNCWINDOWPOS, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOOWNERZORDER,
                SWP_NOSIZE, SWP_NOZORDER, SW_HIDE, SW_SHOWNOACTIVATE, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, TIMERPROC, WH_KEYBOARD, WH_MOUSE, WM_ACTIVATE, WM_APP, WM_DESTROY, WM_ERASEBKGND,
                WM_KEYDOWN, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_PAINT, WM_PRINTCLIENT, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SETTINGCHANGE, WM_THEMECHANGED, WNDCLASSEXW,
                WS_CLIPSIBLINGS, WS_EX_LAYERED, WS_EX_TOOLWINDOW, WS_POPUP,
            },
        },
    },
};

const HOOK_PROP_NAME: &str = "WCPOPUP_KEYBOARD_HOOK";
// https://learn.microsoft.com/en-us/windows/apps/design/signature-experiences/geometry
const CORNER_RADIUS: i32 = 8;
const SHOW_SUBMENU_TIMER_ID: usize = 500;
const HIDE_SUBMENU_TIMER_ID: usize = 501;
const FADE_EFFECT_TIME: u32 = 120;

const WM_MENUSELECTED: u32 = WM_APP + 0x0001;
#[cfg(feature = "accelerator")]
const WM_MENUCOMMAND: u32 = WM_APP + 0x002;
const WM_CLOSEMENU: u32 = WM_APP + 0x0003;
const WM_INACTIVATE: u32 = WM_APP + 0x0004;

#[derive(Debug, PartialEq, Eq)]
enum ThemeChangeFactor {
    SystemSetting,
    User,
    App,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum MenuType {
    Main,
    Submenu,
}

#[derive(Clone, Copy, Debug)]
struct Size {
    width: i32,
    height: i32,
}

#[derive(Clone, Copy, Debug)]
struct Button {
    left: ButtonSize,
    right: ButtonSize,
}

#[derive(Clone, Copy, Debug)]
struct ButtonSize {
    width: i32,
    margins: i32,
}

#[derive(Debug, Clone)]
struct DisplayPoint {
    x: i32,
    y: i32,
    rtl: bool,
    reverse: bool,
}

/// Context Menu.
#[derive(Debug, Clone, Serialize)]
pub struct Menu {
    pub window_handle: isize,
    pub menu_type: MenuType,
    parent_window_handle: isize,
    width: i32,
    height: i32,
}

impl Default for Menu {
    fn default() -> Self {
        Self {
            window_handle: 0,
            menu_type: MenuType::Main,
            parent_window_handle: 0,
            height: 0,
            width: 0,
        }
    }
}

struct PopupInfo {
    menu_thread_id: u32,
    current_thread_id: u32,
    keyboard_hook: HHOOK,
    mouse_hook: HHOOK,
}

impl Menu {
    pub(crate) fn create_window(&self, parent: isize) -> isize {
        create_menu_window(parent).unwrap()
    }

    pub fn config(&self) -> Config {
        get_menu_data(self.window_handle).config.clone()
    }

    pub fn theme(&self) -> Theme {
        let window_handle = if self.menu_type == MenuType::Main {
            self.window_handle
        } else {
            self.parent_window_handle
        };
        let data = get_menu_data(window_handle);
        data.current_theme
    }

    /// Sets the theme for Menu.
    pub fn set_theme(&self, theme: Theme) {
        let window_handle = if self.menu_type == MenuType::Main {
            self.window_handle
        } else {
            self.parent_window_handle
        };
        on_theme_change(window_handle, Some(theme), ThemeChangeFactor::User);
    }

    /// Gets all MenuItems of Menu.
    pub fn items(&self) -> Vec<MenuItem> {
        get_menu_data(self.window_handle).items.clone()
    }

    /// Gets the MenuItem with the specified id.
    pub fn get_menu_item_by_id(&self, id: &str) -> Option<MenuItem> {
        let window_handle = if self.menu_type == MenuType::Main {
            self.window_handle
        } else {
            self.parent_window_handle
        };
        let data = get_menu_data(window_handle);
        find_by_id(data, id)
    }

    /// Adds a MenuItem to the end of MenuItems.
    pub fn append(&mut self, mut item: MenuItem) {
        #[cfg(feature = "accelerator")]
        Self::reset_haccel(self, &item, false);

        let data = get_menu_data_mut(self.window_handle);
        item.menu_window_handle = self.window_handle;
        data.items.push(item);
        Self::recalculate(self, data);
    }

    /// Adds a MenuItem at the specified index.
    pub fn insert(&mut self, mut item: MenuItem, index: u32) {
        #[cfg(feature = "accelerator")]
        Self::reset_haccel(self, &item, false);

        let data = get_menu_data_mut(self.window_handle);
        item.menu_window_handle = self.window_handle;
        data.items.insert(index as usize, item);
        Self::recalculate(self, data);
    }

    /// Removes the MenuItem at the specified index.
    pub fn remove_at(&mut self, index: u32) {
        let data = get_menu_data_mut(self.window_handle);
        #[allow(unused_variables)]
        let removed_item = data.items.remove(index as usize);
        Self::recalculate(self, data);
        #[cfg(feature = "accelerator")]
        Self::reset_haccel(self, &removed_item, true);
    }

    /// Removes the MenuItem.
    pub fn remove(&mut self, item: &MenuItem) {
        let data = get_menu_data_mut(self.window_handle);
        let maybe_index = index_of_item(data, item.uuid);
        if let Some((data, index)) = maybe_index {
            #[allow(unused_variables)]
            let removed_item = data.items.remove(index);
            Self::recalculate(self, data);
            #[cfg(feature = "accelerator")]
            Self::reset_haccel(self, &removed_item, true);
        }
    }

    pub(crate) fn attach_owner_subclass(&self, id: usize) {
        unsafe {
            let hwnd = HWND(self.window_handle);
            let parent_hwnd = HWND(self.parent_window_handle);
            let ancestor = GetAncestor(parent_hwnd, GA_ROOTOWNER);
            let _ = SetWindowSubclass(
                if ancestor.0 == 0 {
                    parent_hwnd
                } else {
                    ancestor
                },
                Some(menu_owner_subclass_proc),
                id,
                Box::into_raw(Box::new(hwnd)) as _,
            );
        }
    }

    #[cfg(feature = "accelerator")]
    fn reset_haccel(&self, item: &MenuItem, should_remove: bool) {
        if item.accelerator.is_empty() {
            return;
        }

        let window_handle = if self.menu_type == MenuType::Main {
            self.window_handle
        } else {
            unsafe { GetParent(HWND(self.window_handle)).0 }
        };

        let data = get_menu_data_mut(window_handle);

        let mut accelerators = data.accelerators.clone();

        if should_remove {
            accelerators.remove_entry(&item.uuid);
        } else {
            accelerators.insert(item.uuid, item.accelerator.clone());
        }

        if let Some(haccel) = &data.haccel {
            destroy_haccel(HACCEL(haccel.0));
        }

        match create_haccel(&accelerators) {
            Some(accel) => data.haccel = Some(Rc::new(accel)),
            None => data.haccel = None,
        }

        data.accelerators = accelerators;

        set_menu_data(window_handle, data);
    }

    fn recalculate(&mut self, data: &mut MenuData) {
        let size = Self::calculate(self, &mut data.items, &data.config, data.current_theme, data.button).unwrap();
        data.width = size.width;
        data.height = size.height;
        set_menu_data(self.window_handle, data);
    }

    fn calculate(&mut self, items: &mut [MenuItem], config: &Config, theme: Theme, button: Button) -> Result<Size, Error> {
        let mut width = 0;
        let mut height = 0;

        // Add padding
        height += config.size.vertical_padding;
        // Add border size
        height += config.size.border_size;

        if config.corner == Corner::Round {
            height += CORNER_RADIUS;
        }

        let factory = create_write_factory();
        // Calculate item top, left, bottom and menu size
        for (index, item) in items.iter_mut().enumerate() {
            item.index = index as i32;

            item.top = height;
            item.left = config.size.border_size + config.size.horizontal_padding;
            let (item_width, item_height) = measure_item(&factory, config, item, theme, button)?;
            item.bottom = item.top + item_height;

            width = std::cmp::max(width, item_width);
            height += item_height;
        }

        // Calculate item right
        for item in items {
            item.right = item.left + width;
        }

        // Add padding
        width += config.size.horizontal_padding * 2;
        height += config.size.vertical_padding;

        if config.corner == Corner::Round {
            height += CORNER_RADIUS;
        }

        // Add border size
        width += config.size.border_size * 2;
        height += config.size.border_size;

        self.width = width;
        self.height = height;

        Ok(Size {
            width,
            height,
        })
    }

    fn start_popup(&self, x: i32, y: i32, sync: bool) -> PopupInfo {
        let hwnd = HWND(self.window_handle);
        let parent = HWND(self.parent_window_handle);
        let menu_thread_id = unsafe { GetWindowThreadProcessId(hwnd, None) };
        let current_thread_id = unsafe { GetCurrentThreadId() };

        if !sync {
            let _ = unsafe { AttachThreadInput(current_thread_id, menu_thread_id, true) };
            let data = get_menu_data_mut(self.window_handle);
            data.thread_id = current_thread_id;
            set_menu_data(self.window_handle, data);
        }

        // Activate parent window
        let _ = unsafe { SetForegroundWindow(parent) };
        let _ = unsafe { SetActiveWindow(parent) };

        let pt = get_display_point(self.window_handle, x, y, self.width, self.height);
        let _ = unsafe { SetWindowPos(hwnd, HWND_TOP, pt.x, pt.y, self.width, self.height, SWP_ASYNCWINDOWPOS | SWP_NOOWNERZORDER | SWP_NOACTIVATE) };

        // Set menu hwnd to property to be used in keyboard hook
        unsafe { SetPropW(hwnd, PCWSTR::from_raw(encode_wide(HOOK_PROP_NAME).as_ptr()), HANDLE(self.window_handle)).unwrap() };

        // Set hooks
        let keyboard_hook = unsafe { SetWindowsHookExW(WH_KEYBOARD, Some(keyboard_hook), None, menu_thread_id).unwrap() };
        let mouse_hook = unsafe { SetWindowsHookExW(WH_MOUSE, Some(mouse_hook), None, menu_thread_id).unwrap() };

        PopupInfo {
            menu_thread_id,
            current_thread_id: if sync {
                0
            } else {
                current_thread_id
            },
            keyboard_hook,
            mouse_hook,
        }
    }

    fn finish_popup(&self, info: PopupInfo) {
        let hwnd = HWND(self.window_handle);
        let _ = unsafe { ReleaseCapture() };

        let _ = unsafe { ShowWindow(hwnd, SW_HIDE) };

        let _ = unsafe { RemovePropW(hwnd, PCWSTR::from_raw(encode_wide(HOOK_PROP_NAME).as_ptr())) };

        // Unhook hooks
        let _ = unsafe { UnhookWindowsHookEx(info.keyboard_hook) };
        let _ = unsafe { UnhookWindowsHookEx(info.mouse_hook) };

        if info.current_thread_id > 0 {
            let _ = unsafe { AttachThreadInput(info.current_thread_id, info.menu_thread_id, false) };
        }
    }

    /// Shows Menu asynchronously at the specified point and returns a selected MenuItem if any.
    pub async fn popup_at_async(&self, x: i32, y: i32) -> Option<MenuItem> {
        // Prepare
        let info = Self::start_popup(self, x, y, false);

        // Show menu window
        animate_show_window(self.window_handle);
        set_capture(self.window_handle);

        let mut msg = MSG::default();
        let mut selected_item: Option<MenuItem> = None;

        async {
            while unsafe { GetMessageW(&mut msg, None, 0, 0).as_bool() } {
                match msg.message {
                    WM_MENUSELECTED => {
                        selected_item = Some(unsafe { transmute::<isize, &MenuItem>(msg.lParam.0).clone() });
                        break;
                    }

                    #[cfg(feature = "accelerator")]
                    WM_MENUCOMMAND => {
                        selected_item = Some(unsafe { transmute::<isize, &MenuItem>(msg.lParam.0).clone() });
                        break;
                    }

                    WM_CLOSEMENU => {
                        break;
                    }

                    _ => {}
                }
            }
        }
        .await;

        Self::finish_popup(self, info);

        selected_item
    }

    /// Shows Menu at the specified point and returns a selected MenuItem if any.
    pub fn popup_at(&self, x: i32, y: i32) -> Option<MenuItem> {
        // Prepare
        let info = Self::start_popup(self, x, y, true);

        // Show menu window
        animate_show_window(self.window_handle);
        set_capture(self.window_handle);

        let mut msg = MSG::default();
        let mut selected_item: Option<MenuItem> = None;

        while unsafe { GetMessageW(&mut msg, None, 0, 0).as_bool() } {
            match msg.message {
                WM_MENUSELECTED => {
                    selected_item = Some(unsafe { transmute::<isize, &MenuItem>(msg.lParam.0).clone() });
                    break;
                }

                #[cfg(feature = "accelerator")]
                WM_MENUCOMMAND => {
                    selected_item = Some(unsafe { transmute::<isize, &MenuItem>(msg.lParam.0).clone() });
                    break;
                }

                WM_CLOSEMENU => {
                    break;
                }

                _ => {
                    let _ = unsafe { TranslateMessage(&msg) };
                    unsafe { DispatchMessageW(&msg) };
                }
            }
        }

        Self::finish_popup(self, info);

        selected_item
    }
}

fn animate_show_window(window_handle: isize) {
    let hwnd = HWND(window_handle);
    let _ = unsafe { AnimateWindow(hwnd, FADE_EFFECT_TIME, AW_BLEND) };
    let _ = unsafe { ShowWindow(hwnd, SW_SHOWNOACTIVATE) };
}

fn set_capture(window_handle: isize) {
    let hwnd = HWND(window_handle);
    // Prevent mouse input on window beneath menu
    unsafe { SetCapture(hwnd) };

    let cursor = unsafe { LoadCursorW(None, IDC_ARROW).unwrap() };
    let _ = unsafe { SetCursor(cursor) };
}

unsafe extern "system" fn keyboard_hook(ncode: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // Prevent keyboard input while Menu is open
    if ncode >= 0 {
        let capture_window = unsafe { GetCapture() };
        let data = unsafe { GetPropW(capture_window, PCWSTR::from_raw(encode_wide(HOOK_PROP_NAME).as_ptr())) };

        unsafe { PostMessageW(HWND(data.0), WM_KEYDOWN, wparam, lparam).unwrap() };
        return LRESULT(1);
    }

    CallNextHookEx(None, ncode, wparam, lparam)
}

unsafe extern "system" fn mouse_hook(ncode: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if ncode >= 0 {
        let capture_window = unsafe { GetCapture() };
        let data = unsafe { GetPropW(capture_window, PCWSTR::from_raw(encode_wide(HOOK_PROP_NAME).as_ptr())) };

        match wparam.0 as u32 {
            // Do not direct buttondown event since it is sent to default_window_proc
            WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_LBUTTONUP | WM_RBUTTONUP => {}
            _ => {
                let _ = unsafe { PostMessageW(HWND(data.0), wparam.0 as u32, WPARAM(0), LPARAM(0)) };
            }
        };
    };

    CallNextHookEx(None, ncode, wparam, lparam)
}

unsafe extern "system" fn default_window_proc(window: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_INACTIVATE => {
            if IsWindowVisible(window).as_bool() {
                let id = init_menu_data(window.0);
                post_message(window, id, WM_CLOSEMENU, WPARAM(0), LPARAM(0));
            }

            LRESULT(0)
        }

        WM_DESTROY => {
            free_library();
            let data = get_menu_data_mut(window.0);
            if data.menu_type == MenuType::Main {
                let _ = unsafe { RemovePropW(window, PCWSTR::from_raw(encode_wide(HOOK_PROP_NAME).as_ptr())) };
                let _ = RemoveWindowSubclass(window, Some(menu_owner_subclass_proc), data.win_subclass_id.unwrap() as usize);
            }

            #[cfg(feature = "accelerator")]
            if data.menu_type == MenuType::Main {
                if let Some(accel) = &data.haccel {
                    destroy_haccel(HACCEL(accel.0));
                }
            }

            let _ = Box::from_raw(data);

            DefWindowProcW(window, msg, wparam, lparam)
        }

        WM_PRINTCLIENT => {
            let hdc = HDC(wparam.0 as isize);
            let data = get_menu_data(window.0);
            paint_background(window, data, Some(hdc));
            paint(hdc, data, &data.items).unwrap();
            LRESULT(1)
        }

        WM_ERASEBKGND => {
            let data = get_menu_data(window.0);
            paint_background(window, data, None);
            LRESULT(0)
        }

        WM_PAINT => {
            let data = get_menu_data(window.0);
            on_paint(window, data).unwrap();
            LRESULT(0)
        }

        WM_KEYDOWN => {
            let should_close_menu = matches!(VIRTUAL_KEY(wparam.0 as u16), VK_ESCAPE | VK_LWIN | VK_RWIN);

            if should_close_menu {
                let id = init_menu_data(window.0);
                post_message(window, id, WM_CLOSEMENU, WPARAM(0), LPARAM(0));
                return LRESULT(0);
            }

            #[cfg(feature = "accelerator")]
            let data = get_menu_data(window.0);
            #[cfg(feature = "accelerator")]
            if let Some(accel) = &data.haccel {
                let keydown_msg = MSG {
                    hwnd: window,
                    wParam: wparam,
                    lParam: lparam,
                    message: msg,
                    time: 0,
                    pt: POINT::default(),
                };
                TranslateAcceleratorW(window, HACCEL(accel.0), &keydown_msg);
            }

            LRESULT(0)
        }

        #[cfg(feature = "accelerator")]
        WM_COMMAND | WM_SYSCOMMAND => {
            if HIWORD(wparam.0 as u32) != 1 {
                return LRESULT(0);
            }

            let data = get_menu_data_mut(window.0);
            let maybe_index = index_of_item(data, LOWORD(wparam.0 as u32));
            if let Some((data, index)) = maybe_index {
                if on_menu_item_selected(data, index) {
                    let menu_item = data.items[index].clone();
                    let id = init_menu_data(window.0);
                    post_message(window, id, WM_MENUCOMMAND, WPARAM(0), LPARAM(Box::into_raw(Box::new(menu_item)) as _));
                }
            }

            LRESULT(0)
        }

        WM_MOUSEMOVE => {
            on_mouse_move(window);
            LRESULT(0)
        }

        WM_LBUTTONDOWN | WM_RBUTTONDOWN => {
            on_mouse_down(window, msg);
            LRESULT(0)
        }

        WM_LBUTTONUP | WM_RBUTTONUP => {
            on_mouse_up(window);
            LRESULT(0)
        }

        WM_MOUSEWHEEL => {
            let id = init_menu_data(window.0);
            post_message(window, id, WM_CLOSEMENU, WPARAM(0), LPARAM(0));
            LRESULT(0)
        }

        WM_SETTINGCHANGE => {
            let wide_string_ptr = lparam.0 as *const u16;
            let lparam_str = PCWSTR::from_raw(wide_string_ptr).to_string().unwrap_or_default();
            if lparam_str == "ImmersiveColorSet" {
                on_theme_change(window.0, None, ThemeChangeFactor::SystemSetting);
            }

            DefWindowProcW(window, msg, wparam, lparam)
        }

        _ => DefWindowProcW(window, msg, wparam, lparam),
    }
}

fn on_mouse_move(window: HWND) {
    let mut pt = POINT::default();
    let _ = unsafe { GetCursorPos(&mut pt) };
    let data = get_menu_data_mut(window.0);

    if data.visible_submenu_index >= 0 {
        let submenu_window_handle = data.items[data.visible_submenu_index as usize].submenu.as_ref().unwrap().window_handle;
        let submenu_data = get_menu_data_mut(submenu_window_handle);
        change_selection(submenu_data, submenu_window_handle, pt);
        set_menu_data(submenu_window_handle, submenu_data);
    }

    let should_show_submenu = change_selection(data, window.0, pt);
    set_menu_data(window.0, data);

    // Show submenu after submenu index is stored
    if should_show_submenu {
        show_submenu(window);
    }
}

fn on_mouse_down(window: HWND, msg: u32) {
    // If mouse input occurs outside of menu
    if get_hwnd_from_point(window).is_none() {
        // Immediately release capture so that the event is sent to the target window
        let _ = unsafe { ReleaseCapture() };

        // Close menu
        let id = init_menu_data(window.0);
        post_message(window, id, WM_CLOSEMENU, WPARAM(0), LPARAM(0));

        // If mouse input occurs on parent window, send mouse input
        send_mouse_input(window, msg);
    }
}

fn on_mouse_up(window: HWND) {
    let maybe_hwnd = get_hwnd_from_point(window);
    if maybe_hwnd.is_none() {
        return;
    }

    let hwnd = maybe_hwnd.unwrap();
    let data = get_menu_data_mut(hwnd.0);
    let index = index_from_point(hwnd, get_cursor_point(window), data);

    if index < 0 {
        return;
    }

    if on_menu_item_selected(data, index as usize) {
        set_menu_data(hwnd.0, data);
        let menu_item = data.items[index as usize].clone();
        let id = init_menu_data(window.0);
        post_message(hwnd, id, WM_MENUSELECTED, WPARAM(0), LPARAM(Box::into_raw(Box::new(menu_item)) as _));
    }
}

fn post_message(hwnd: HWND, thread_id: u32, message: u32, wparam: WPARAM, lparam: LPARAM) {
    if thread_id > 0 {
        unsafe { PostThreadMessageW(thread_id, message, wparam, lparam).unwrap() };
    } else {
        unsafe { PostMessageW(hwnd, message, wparam, lparam).unwrap() };
    }
}

fn on_menu_item_selected(data: &mut MenuData, index: usize) -> bool {
    // If submenu, ignore
    if data.items[index].menu_item_type == MenuItemType::Submenu {
        return false;
    }

    // If disabled, ignore
    if data.items[index].disabled {
        return false;
    }

    // toggle checkbox
    if data.items[index].menu_item_type == MenuItemType::Checkbox {
        data.items[index].checked = !data.items[index].checked;
    }

    // toggle radio checkbox
    if data.items[index].menu_item_type == MenuItemType::Radio {
        toggle_radio(data, index);
    }

    true
}

fn get_parent_window(child: HWND) -> HWND {
    let owner = unsafe { GetWindow(child, GW_OWNER) };

    if owner.0 != 0 {
        return owner;
    }

    unsafe { GetParent(child) }
}

fn send_mouse_input(hwnd: HWND, msg: u32) {
    let mut count = 0;

    let mut parent = get_parent_window(hwnd);
    let mut cursor_point = POINT::default();
    let _ = unsafe { GetCursorPos(&mut cursor_point) };

    while parent.0 != 0 {
        let mut rect = RECT::default();
        let _ = unsafe { GetWindowRect(parent, &mut rect) };
        if unsafe { PtInRect(&mut rect as *const _ as _, cursor_point) }.as_bool() {
            count += 1;
        }
        parent = get_parent_window(parent);
    }

    if count > 0 {
        let mut flags = MOUSEEVENTF_VIRTUALDESK | MOUSEEVENTF_ABSOLUTE;
        flags |= if msg == WM_LBUTTONDOWN {
            MOUSEEVENTF_LEFTDOWN
        } else {
            MOUSEEVENTF_RIGHTDOWN
        };

        let input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: cursor_point.x,
                    dy: cursor_point.y,
                    mouseData: 0,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        unsafe { SendInput(&[input], size_of::<INPUT>() as i32) };
    }
}

unsafe extern "system" fn menu_owner_subclass_proc(window: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM, _uidsubclass: usize, dwrefdata: usize) -> LRESULT {
    match msg {
        WM_ACTIVATE => {
            if wparam == WPARAM(0) {
                let hwnd_ptr = dwrefdata as *const HWND;
                let hwnd = unsafe { *hwnd_ptr };
                if IsWindow(hwnd).as_bool() {
                    PostMessageW(hwnd, WM_INACTIVATE, WPARAM(0), LPARAM(0)).unwrap();
                }
            }
            DefSubclassProc(window, msg, wparam, lparam)
        }

        WM_THEMECHANGED => {
            let hwnd_ptr = dwrefdata as *const HWND;
            let hwnd = unsafe { *hwnd_ptr };
            on_theme_change(hwnd.0, None, ThemeChangeFactor::App);
            DefSubclassProc(window, msg, wparam, lparam)
        }

        _ => DefSubclassProc(window, msg, wparam, lparam),
    }
}

fn init_menu_data(window_handle: isize) -> u32 {
    let data = get_menu_data_mut(window_handle);

    let thread_id = if data.menu_type == MenuType::Main {
        data.thread_id
    } else {
        let parent_data = get_menu_data(data.parent);
        parent_data.thread_id
    };

    data.selected_index = -1;
    data.thread_id = 0;

    if data.visible_submenu_index >= 0 {
        let submenu_window_handle = data.items[data.visible_submenu_index as usize].submenu.as_ref().unwrap().window_handle;
        hide_submenu(submenu_window_handle);
    }
    data.visible_submenu_index = -1;

    set_menu_data(window_handle, data);

    thread_id
}

fn find_by_id(data: &MenuData, id: &str) -> Option<MenuItem> {
    let item_id = id.to_string();
    for item in &data.items {
        if item.id == item_id {
            return Some(item.clone());
        }

        if item.menu_item_type == MenuItemType::Submenu {
            let submenu_window_handle = item.submenu.as_ref().unwrap().window_handle;
            let submenu_data = get_menu_data(submenu_window_handle);
            if let Some(menu_item) = find_by_id(submenu_data, id) {
                return Some(menu_item);
            }
        }
    }
    None
}

fn index_of_item(data: &mut MenuData, uuid: u16) -> Option<(&mut MenuData, usize)> {
    for (index, item) in data.items.iter().enumerate() {
        if item.uuid == uuid {
            return Some((data, index));
        }

        if item.menu_item_type == MenuItemType::Submenu {
            let submenu_window_handle = item.submenu.as_ref().unwrap().window_handle;
            let submenu_data = get_menu_data_mut(submenu_window_handle);
            if let Some(index) = index_of_item(submenu_data, uuid) {
                return Some(index);
            }
        }
    }
    None
}

fn paint_background(hwnd: HWND, data: &MenuData, hdc: Option<HDC>) {
    let dc = if let Some(hdc) = hdc {
        hdc
    } else {
        unsafe { GetWindowDC(hwnd) }
    };

    if dc.0 == 0 {
        return;
    }

    let scheme = get_color_scheme(data);

    let mut client_rect = RECT::default();
    unsafe { GetClientRect(hwnd, &mut client_rect).unwrap() };

    unsafe { data.dc_render_target.BindDC(dc, &client_rect).unwrap() };
    unsafe { data.dc_render_target.BeginDraw() };

    let brush = unsafe { data.dc_render_target.CreateSolidColorBrush(&colorref_to_d2d1_color_f(scheme.border), None).unwrap() };

    unsafe { data.dc_render_target.FillRectangle(&to_2d_rect(&client_rect), &brush) };

    let menu_rect = RECT {
        left: client_rect.left + data.config.size.border_size,
        top: client_rect.top + data.config.size.border_size,
        right: client_rect.right - data.config.size.border_size,
        bottom: client_rect.bottom - data.config.size.border_size,
    };

    let brush = unsafe { data.dc_render_target.CreateSolidColorBrush(&colorref_to_d2d1_color_f(scheme.background_color), None).unwrap() };

    if data.config.corner == Corner::Round {
        unsafe {
            data.dc_render_target.FillRoundedRectangle(
                &D2D1_ROUNDED_RECT {
                    rect: to_2d_rect(&menu_rect),
                    radiusX: CORNER_RADIUS as f32,
                    radiusY: CORNER_RADIUS as f32,
                },
                &brush,
            )
        };
    } else {
        unsafe { data.dc_render_target.FillRectangle(&to_2d_rect(&menu_rect), &brush) };
    }

    unsafe { data.dc_render_target.EndDraw(None, None).unwrap() };

    if hdc.is_none() {
        unsafe { ReleaseDC(hwnd, dc) };
    }
}

fn on_paint(hwnd: HWND, data: &MenuData) -> Result<(), Error> {
    let mut paint_struct = PAINTSTRUCT::default();
    let dc = unsafe { BeginPaint(hwnd, &mut paint_struct) };

    if dc.0 == 0 {
        return Ok(());
    }

    let index = index_from_rect(data, paint_struct.rcPaint);

    if index.is_none() {
        paint(dc, data, &data.items)?;
    } else {
        paint(dc, data, &vec![data.items[index.unwrap() as usize].clone()])?;
    }

    let _ = unsafe { EndPaint(hwnd, &paint_struct) };

    Ok(())
}

fn paint(dc: HDC, data: &MenuData, items: &Vec<MenuItem>) -> Result<(), Error> {
    let scheme = get_color_scheme(data);

    let client_rect = RECT {
        left: 0,
        top: 0,
        right: data.width,
        bottom: data.height,
    };

    unsafe { data.dc_render_target.BindDC(dc, &client_rect).unwrap() };
    unsafe { data.dc_render_target.BeginDraw() };

    for item in items {
        let whole_item_rect = get_item_rect(item);

        let disabled = item.disabled;
        let checked = item.checked;
        let selected = item.index == data.selected_index && !disabled;

        fill_background(data, &whole_item_rect, scheme, selected)?;

        match item.menu_item_type {
            MenuItemType::Separator => {
                // Use whole item rect to draw from left to right
                draw_separator(data, &whole_item_rect, scheme)?;
            }

            _ => {
                let item_rect = RECT {
                    left: whole_item_rect.left + data.config.size.item_horizontal_padding,
                    top: whole_item_rect.top + data.config.size.item_vertical_padding,
                    right: whole_item_rect.right - data.config.size.item_horizontal_padding,
                    bottom: whole_item_rect.bottom - data.config.size.item_vertical_padding,
                };

                if checked {
                    draw_checkmark(data, &item_rect, scheme, disabled)?;
                }

                if item.menu_item_type == MenuItemType::Submenu {
                    draw_submenu_arrow(data, &item_rect, scheme, disabled)?;
                }

                draw_menu_text(data, item, &item_rect, scheme, disabled)?;
            }
        }
    }

    unsafe { data.dc_render_target.EndDraw(None, None).unwrap() };

    Ok(())
}

fn fill_background(data: &MenuData, item_rect: &RECT, scheme: &ColorScheme, selected: bool) -> Result<(), Error> {
    let brush = if selected {
        unsafe { data.dc_render_target.CreateSolidColorBrush(&colorref_to_d2d1_color_f(scheme.hover_background_color), None)? }
    } else {
        unsafe { data.dc_render_target.CreateSolidColorBrush(&colorref_to_d2d1_color_f(scheme.background_color), None)? }
    };

    unsafe { data.dc_render_target.FillRectangle(&to_2d_rect(item_rect), &brush) };

    Ok(())
}

fn draw_menu_text(data: &MenuData, item: &MenuItem, item_rect: &RECT, scheme: &ColorScheme, disabled: bool) -> Result<(), Error> {
    // Keep space for check mark and submenu mark
    let text_rect = RECT {
        left: item_rect.left + (data.button.left.width + data.button.left.margins),
        top: item_rect.top,
        right: item_rect.right - (data.button.right.width + data.button.right.margins),
        bottom: item_rect.bottom,
    };

    let text_2d_rect = to_2d_rect(&text_rect);
    let factory = create_write_factory();
    let format = get_text_format(&factory, data.current_theme, &data.config, TextAlignment::Leading)?;

    let color = if disabled {
        scheme.disabled
    } else {
        scheme.color
    };
    let brush = unsafe { data.dc_render_target.CreateSolidColorBrush(&colorref_to_d2d1_color_f(color), None) }?;

    unsafe { data.dc_render_target.DrawText(encode_wide(&item.label).as_mut(), &format, &text_2d_rect, &brush, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL) };

    if !item.accelerator.is_empty() {
        unsafe { format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_TRAILING) }?;
        unsafe { data.dc_render_target.DrawText(encode_wide(&item.accelerator).as_mut(), &format, &text_2d_rect, &brush, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL) };
    }

    Ok(())
}

fn draw_checkmark(data: &MenuData, item_rect: &RECT, scheme: &ColorScheme, disabled: bool) -> Result<(), Error> {
    let margin = data.button.left.margins / 2;
    let button = data.button.left.width;
    let mut check_rect = RECT {
        left: item_rect.left + margin,
        top: item_rect.top,
        right: item_rect.left + margin + button + margin,
        bottom: item_rect.top + button,
    };

    // center vertically
    let _ = unsafe { OffsetRect(&mut check_rect, 0, ((item_rect.bottom - item_rect.top) - (check_rect.bottom - check_rect.top)) / 2) };

    let color = if disabled {
        scheme.disabled
    } else {
        scheme.color
    };

    let dc5 = get_device_context(&data.dc_render_target);

    let element = unsafe { data.check_svg.GetRoot() }?;
    set_fill_color(&element, color);
    set_stroke_color(&element, color);

    let translation = Matrix3x2::translation(check_rect.left as f32, check_rect.top as f32);
    unsafe { dc5.SetTransform(&translation) };
    unsafe { dc5.DrawSvgDocument(&data.check_svg) };
    unsafe { dc5.SetTransform(&Matrix3x2::identity()) };

    Ok(())
}

fn draw_submenu_arrow(data: &MenuData, item_rect: &RECT, scheme: &ColorScheme, disabled: bool) -> Result<(), Error> {
    let margin = data.button.right.margins / 2;
    let button = data.button.right.width;
    let mut arrow_rect = RECT {
        left: item_rect.right - (margin + button),
        top: item_rect.top,
        right: item_rect.right - margin,
        bottom: item_rect.top + button,
    };

    // center vertically
    let _ = unsafe { OffsetRect(&mut arrow_rect as *mut _, 0, ((item_rect.bottom - item_rect.top) - (arrow_rect.bottom - arrow_rect.top)) / 2) };
    let color = if disabled {
        scheme.disabled
    } else {
        scheme.color
    };

    let dc5 = get_device_context(&data.dc_render_target);

    let element = unsafe { data.submenu_svg.GetRoot() }?;
    set_fill_color(&element, color);
    set_stroke_color(&element, color);

    let translation = Matrix3x2::translation(arrow_rect.left as f32, arrow_rect.top as f32);
    unsafe { dc5.SetTransform(&translation) };
    unsafe { dc5.DrawSvgDocument(&data.submenu_svg) };
    unsafe { dc5.SetTransform(&Matrix3x2::identity()) };

    Ok(())
}

fn draw_separator(data: &MenuData, rect: &RECT, scheme: &ColorScheme) -> Result<(), Error> {
    let separator_rect = RECT {
        left: rect.left,
        top: rect.top + (rect.bottom - rect.top) / 2,
        right: rect.right,
        bottom: rect.bottom,
    };
    let rect = to_2d_rect(&separator_rect);

    let brush = unsafe { data.dc_render_target.CreateSolidColorBrush(&colorref_to_d2d1_color_f(scheme.border), None).unwrap() };

    // Add 0.5 to disable antialiasing for line
    unsafe {
        data.dc_render_target.DrawLine(
            D2D_POINT_2F {
                x: rect.left + 0.5,
                y: rect.top + 0.5,
            },
            D2D_POINT_2F {
                x: rect.right + 0.5,
                y: rect.top + 0.5,
            },
            &brush,
            1.0,
            None,
        )
    }

    Ok(())
}

fn get_display_point(window_handle: isize, x: i32, y: i32, cx: i32, cy: i32) -> DisplayPoint {
    let hwnd = HWND(window_handle);
    let mut rtl = false;
    let mut reverse = false;

    let mut ppt = POINT {
        x,
        y,
    };

    let mut hmon = unsafe { MonitorFromPoint(ppt, MONITOR_DEFAULTTONULL) };

    if hmon.0 == 0 {
        hmon = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
    }

    let mut minf = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };

    let _ = unsafe { GetMonitorInfoW(hmon, &mut minf) };

    if ppt.y < minf.rcWork.top {
        ppt.y = minf.rcMonitor.top;
    }

    if ppt.x < minf.rcWork.left {
        ppt.x = minf.rcMonitor.left;
    }

    if ppt.y + cy >= minf.rcWork.bottom {
        ppt.y -= cy;
        reverse = true;
    }

    if ppt.x + cx >= minf.rcWork.right {
        ppt.x -= cx;
        rtl = true;
    }

    DisplayPoint {
        x: ppt.x,
        y: ppt.y,
        rtl,
        reverse,
    }
}

fn change_selection(data: &mut MenuData, window_handle: isize, screen_point: POINT) -> bool {
    let hwnd = HWND(window_handle);
    // Menu is yet to be visible due to timer or animation
    if unsafe { !IsWindowVisible(hwnd) }.as_bool() {
        return false;
    }

    let selected_index = index_from_point(hwnd, screen_point, data);

    if data.visible_submenu_index >= 0 && selected_index < 0 {
        return false;
    }

    let mut should_show_submenu = false;

    if data.selected_index != selected_index {
        should_show_submenu = toggle_submenu(data, selected_index);

        if selected_index >= 0 {
            let item = &data.items[selected_index as usize];
            if !item.disabled {
                let rect = get_item_rect(item);
                let _ = unsafe { InvalidateRect(hwnd, Some(&rect), false) };
            }
        }

        if data.selected_index >= 0 {
            let item = &data.items[data.selected_index as usize];
            if !item.disabled {
                let rect = get_item_rect(item);
                let _ = unsafe { InvalidateRect(hwnd, Some(&rect), false) };
            }
        }
    };

    data.selected_index = selected_index;

    should_show_submenu
}

fn toggle_submenu(data: &mut MenuData, selected_index: i32) -> bool {
    let mut should_show_submenu = false;

    if selected_index < 0 {
        return false;
    }

    if data.visible_submenu_index >= 0 && data.visible_submenu_index != selected_index {
        let submenu_window_handle = data.items[data.visible_submenu_index as usize].submenu.as_ref().unwrap().window_handle;
        animate_hide_submenu(submenu_window_handle);
        data.visible_submenu_index = -1;
    }

    if data.visible_submenu_index < 0 && data.items[selected_index as usize].menu_item_type == MenuItemType::Submenu {
        if data.items[selected_index as usize].disabled {
            data.visible_submenu_index = -1;
        } else {
            data.visible_submenu_index = selected_index;
            should_show_submenu = true;
        }
    }

    should_show_submenu
}

fn show_submenu(hwnd: HWND) {
    let proc: TIMERPROC = Some(delay_show_submenu);
    let mut show_delay: u32 = 0;
    let _ = unsafe { SystemParametersInfoW(SPI_GETMENUSHOWDELAY, 0, Some(&mut show_delay as *mut _ as *mut c_void), SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0)) };
    unsafe { SetTimer(hwnd, SHOW_SUBMENU_TIMER_ID, show_delay, proc) };
}

unsafe extern "system" fn delay_show_submenu(hwnd: HWND, _msg: u32, id: usize, _time: u32) {
    KillTimer(hwnd, id).unwrap();

    let main_menu_data = get_menu_data(hwnd.0);

    if main_menu_data.visible_submenu_index >= 0 {
        let item = &main_menu_data.items[main_menu_data.visible_submenu_index as usize];
        let submenu_window_handle = item.submenu.as_ref().unwrap().window_handle;
        let submenu_data = get_menu_data(submenu_window_handle);

        // If submenu has no item, do not show submenu
        if submenu_data.items.is_empty() {
            return;
        }

        let mut main_menu_rect = RECT::default();
        GetWindowRect(hwnd, &mut main_menu_rect).unwrap();

        let pt = get_display_point(submenu_window_handle, main_menu_rect.right, main_menu_rect.top + item.top, submenu_data.width, submenu_data.height);

        let x = if pt.rtl {
            main_menu_rect.left - submenu_data.width - main_menu_data.config.size.submenu_offset
        } else {
            main_menu_rect.right + main_menu_data.config.size.submenu_offset
        };

        let round_corner_size = if main_menu_data.config.corner == Corner::Round {
            CORNER_RADIUS
        } else {
            0
        };
        let y = if pt.reverse {
            let mut reversed_point = POINT {
                x: 0,
                y: item.bottom - submenu_data.height,
            };
            let _ = ClientToScreen(hwnd, &mut reversed_point);
            // Add top/bottom padding, round corner size, and border size
            reversed_point.y + main_menu_data.config.size.vertical_padding * 2 + round_corner_size + main_menu_data.config.size.border_size
        } else {
            // Reduce top padding and border size
            main_menu_rect.top + item.top - main_menu_data.config.size.vertical_padding - round_corner_size - main_menu_data.config.size.border_size
        };

        let submenu_hwnd = HWND(submenu_window_handle);
        SetWindowPos(submenu_hwnd, HWND_TOP, x, y, submenu_data.width, submenu_data.height, SWP_ASYNCWINDOWPOS | SWP_NOOWNERZORDER | SWP_NOACTIVATE).unwrap();
        animate_show_window(submenu_window_handle);
    }
}

fn hide_submenu(window_handle: isize) {
    let data = get_menu_data_mut(window_handle);
    data.selected_index = -1;
    set_menu_data(window_handle, data);
    let hwnd = HWND(window_handle);
    let _ = unsafe { ShowWindow(hwnd, SW_HIDE) };
}

fn animate_hide_submenu(window_handle: isize) {
    let hwnd = HWND(window_handle);
    let proc: TIMERPROC = Some(delay_hide_submenu);
    let mut hide_delay: u32 = 0;
    let _ = unsafe { SystemParametersInfoW(SPI_GETMENUSHOWDELAY, 0, Some(&mut hide_delay as *mut _ as *mut c_void), SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0)) };
    unsafe { SetTimer(hwnd, HIDE_SUBMENU_TIMER_ID, hide_delay - 100, proc) };
}

unsafe extern "system" fn delay_hide_submenu(hwnd: HWND, _msg: u32, id: usize, _time: u32) {
    KillTimer(hwnd, id).unwrap();
    let data = get_menu_data_mut(hwnd.0);
    data.selected_index = -1;
    set_menu_data(hwnd.0, data);
    let _ = unsafe { ShowWindow(hwnd, SW_HIDE) };
}

fn get_item_rect(item: &MenuItem) -> RECT {
    RECT {
        left: item.left,
        top: item.top,
        right: item.right,
        bottom: item.bottom,
    }
}

fn get_cursor_point(_hwnd: HWND) -> POINT {
    let mut pt = POINT::default();
    unsafe { GetCursorPos(&mut pt).unwrap() };
    pt
}

fn index_from_rect(data: &MenuData, rect: RECT) -> Option<i32> {
    if rect.top == 0 && rect.bottom == data.height {
        return None;
    }

    for item in &data.items {
        if rect.top == item.top && rect.bottom == item.bottom {
            return Some(item.index);
        }
    }

    None
}

fn index_from_point(hwnd: HWND, screen_pt: POINT, data: &MenuData) -> i32 {
    let mut selected_index: i32 = -1;
    let mut pt = screen_pt;
    let _ = unsafe { ScreenToClient(hwnd, &mut pt) };

    if pt.x >= 0 && pt.x < data.width && pt.y >= 0 && pt.y < data.height {
        for item in &data.items {
            if pt.y >= item.top && pt.y <= item.bottom && item.menu_item_type != MenuItemType::Separator {
                selected_index = item.index;
                break;
            }
        }
    }

    selected_index
}

fn get_hwnd_from_point(hwnd: HWND) -> Option<HWND> {
    let data = get_menu_data(hwnd.0);
    let submenu = if data.visible_submenu_index >= 0 {
        HWND(data.items[data.visible_submenu_index as usize].submenu.as_ref().unwrap().window_handle)
    } else {
        HWND(0)
    };

    let pt = get_cursor_point(hwnd);

    let window = unsafe { WindowFromPoint(pt) };

    if submenu.0 != 0 && window == submenu {
        return Some(submenu);
    }

    if hwnd == window {
        return Some(hwnd);
    }

    None
}

fn on_theme_change(window_handle: isize, maybe_preferred_theme: Option<Theme>, factor: ThemeChangeFactor) {
    let data = get_menu_data_mut(window_handle);
    if data.menu_type == MenuType::Submenu {
        return;
    }

    let current_them = data.current_theme;

    // Don't respont to setting change event unless theme is System
    if current_them != Theme::System && factor == ThemeChangeFactor::SystemSetting {
        return;
    }

    let should_be_dark = match factor {
        ThemeChangeFactor::User => {
            let preferred_theme = maybe_preferred_theme.unwrap();
            if preferred_theme == Theme::System {
                is_sys_dark_color()
            } else {
                preferred_theme == Theme::Dark
            }
        }
        ThemeChangeFactor::App => {
            if current_them == Theme::System {
                is_sys_dark_color()
            } else {
                should_apps_use_dark_mode()
            }
        }
        ThemeChangeFactor::SystemSetting => is_sys_dark_color(),
    };

    let new_theme = match maybe_preferred_theme {
        Some(preferred_theme) => preferred_theme,
        None => {
            if current_them == Theme::System {
                current_them
            } else if should_be_dark {
                Theme::Dark
            } else {
                Theme::Light
            }
        }
    };

    data.current_theme = new_theme;
    set_window_border_color(window_handle, data).unwrap();
    set_menu_data(window_handle, data);
    let hwnd = HWND(window_handle);
    let _ = unsafe { UpdateWindow(hwnd) };

    for menu_item in &data.items {
        let item = menu_item;
        if item.menu_item_type == MenuItemType::Submenu {
            let submenu_window_handle = item.submenu.as_ref().unwrap().window_handle;
            let submenu_hwnd = HWND(submenu_window_handle);
            let submenu_data = get_menu_data_mut(submenu_window_handle);
            submenu_data.current_theme = new_theme;
            set_window_border_color(submenu_window_handle, data).unwrap();
            set_menu_data(submenu_window_handle, submenu_data);
            let _ = unsafe { UpdateWindow(submenu_hwnd) };
        }
    }
}

fn create_menu_window(parent: isize) -> Result<isize, Error> {
    let class_name = w!("WC_POPUP");

    let class = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW | CS_DROPSHADOW,
        lpfnWndProc: Some(default_window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: unsafe { HINSTANCE(GetModuleHandleW(PCWSTR::null()).unwrap_or_default().0) },
        hIcon: HICON::default(),
        hCursor: HCURSOR::default(),
        hbrBackground: HBRUSH::default(),
        lpszMenuName: PCWSTR::null(),
        lpszClassName: class_name,
        hIconSm: HICON::default(),
    };

    unsafe { RegisterClassExW(&class) };

    let window_styles = WS_POPUP | WS_CLIPSIBLINGS;
    let ex_style = WS_EX_TOOLWINDOW | WS_EX_LAYERED;

    let hwnd = unsafe {
        CreateWindowExW(ex_style, PCWSTR::from_raw(class_name.as_ptr()), PCWSTR::null(), window_styles, 0, 0, 0, 0, HWND(parent), None, GetModuleHandleW(PCWSTR::null()).unwrap_or_default(), None)
    };

    let _ = unsafe { SetWindowPos(hwnd, None, 0, 0, 0, 0, SWP_NOZORDER | SWP_NOOWNERZORDER | SWP_NOMOVE | SWP_NOSIZE | SWP_FRAMECHANGED) };

    Ok(hwnd.0)
}
