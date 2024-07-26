//! Context(popup) menu for Windows.
//!
//! You can customize text, border, background colors using [`ColorScheme`] and border size, margins, paddings using [`MenuSize`].
//! Windows Theme(Dark/Light) is also sopported.
//!
//! ## Example
//!
//! Use ManuBuilder to create a Menu with MenuItems, and then call Menu.popup_at() to show Menu.
//! When a MenuItem is clicked, SelectedMenuItem data is returned.
//!
//! ```no_run
//! fn example(hwnd: HWND) {
//!   let mut builder = MenuBuilder::new(hwnd);
//!
//!   builder.check("menu_item1", "Menu Label 1", "Value 1", true, None);
//!   builder.separator();
//!   builder.text_with_accelerator("menu_item2", "Menu Label 2", None, "Ctrl+P");
//!   builder.text_with_accelerator("menu_item3", "Menu Label 3", None, "F11");
//!   builder.text("menu_item4", "Menu Label 4", None);
//!   builder.separator();
//!   builder.text_with_accelerator("menu_item5", "Menu Label 5", None, "Ctrl+S");
//!   builder.separator();
//!
//!   let mut submenu = builder.submenu("Submenu", None);
//!   submenu.radio("submenu_item1", "Menu Label 1", "Menu Value 2", "Submenu1", true, None);
//!   submenu.radio("submenu_item2", "Menu Label 2", "Menu Value 3", "Submenu1", false, None);
//!   submenu.build().unwrap();
//!
//!   let menu = builder.build().unwrap();
//!
//!   let selected_item = menu.popup_at(100, 100);
//! }
//! ```

mod accelerator;
mod builder;
mod config;
mod menu_item;
mod util;
#[cfg(feature = "accelerator")]
use accelerator::{create_haccel, destroy_haccel};
pub use builder::*;
pub use config::*;
pub use menu_item::*;
use util::*;

#[cfg(feature = "accelerator")]
use std::collections::HashMap;
use std::{
    ffi::c_void,
    mem::{size_of, transmute},
    rc::Rc,
};
#[cfg(feature = "accelerator")]
use windows::Win32::UI::WindowsAndMessaging::{TranslateAcceleratorW, HACCEL, WM_COMMAND, WM_SYSCOMMAND};
use windows::{
    core::{w, Error, PCWSTR},
    Win32::{
        Foundation::{COLORREF, HANDLE, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM},
        Graphics::Gdi::{BeginPaint, ClientToScreen, CreateFontIndirectW, CreatePen, CreateSolidBrush, DeleteObject, DrawTextW, EndPaint, ExcludeClipRect, FillRect, GetMonitorInfoW, GetWindowDC, InflateRect, InvalidateRect, LineTo, MonitorFromPoint, MonitorFromWindow, MoveToEx, OffsetRect, PtInRect, ReleaseDC, ScreenToClient, SelectObject, SetBkMode, SetTextColor, UpdateWindow, DT_LEFT, DT_RIGHT, DT_SINGLELINE, DT_VCENTER, HBRUSH, HDC, MONITORINFO, MONITOR_DEFAULTTONEAREST, MONITOR_DEFAULTTONULL, PAINTSTRUCT, PS_SOLID, TRANSPARENT},
        System::{
            LibraryLoader::GetModuleHandleW,
            Threading::{AttachThreadInput, GetCurrentThreadId},
        },
        UI::{
            Controls::{CloseThemeData, DrawThemeBackgroundEx, OpenThemeDataEx, HTHEME, MC_CHECKMARKNORMAL, MENU_POPUPCHECK, MENU_POPUPSUBMENU, MSM_NORMAL, OTD_NONCLIENT},
            Input::KeyboardAndMouse::{GetCapture, ReleaseCapture, SendInput, SetActiveWindow, SetCapture, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_VIRTUALDESK, MOUSEINPUT, VIRTUAL_KEY, VK_ESCAPE, VK_LWIN, VK_RWIN},
            Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
            WindowsAndMessaging::{
                AnimateWindow, CallNextHookEx, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetAncestor, GetClientRect, GetCursorPos, GetMessageW, GetParent, GetPropW, GetSystemMetrics, GetWindow, GetWindowRect, GetWindowThreadProcessId, IsWindow, IsWindowVisible, KillTimer, LoadCursorW, PostMessageW, PostThreadMessageW, RegisterClassExW, RemovePropW, SetCursor, SetForegroundWindow, SetPropW, SetTimer, SetWindowPos, SetWindowsHookExW, ShowWindow, SystemParametersInfoW, TranslateMessage, UnhookWindowsHookEx, WindowFromPoint, AW_BLEND, CS_DROPSHADOW, CS_HREDRAW, CS_VREDRAW, GA_PARENT,
                GA_ROOTOWNER, GW_OWNER, HCURSOR, HHOOK, HICON, HWND_TOP, IDC_ARROW, MSG, SM_CXHSCROLL, SPI_GETMENUSHOWDELAY, SWP_ASYNCWINDOWPOS, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOOWNERZORDER, SWP_NOSIZE, SWP_NOZORDER, SW_HIDE, SW_SHOWNOACTIVATE, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, TIMERPROC, WH_KEYBOARD, WH_MOUSE, WM_ACTIVATE, WM_APP, WM_DESTROY, WM_KEYDOWN, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_PAINT, WM_PRINTCLIENT, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SETTINGCHANGE, WM_THEMECHANGED, WNDCLASSEXW, WS_CLIPSIBLINGS, WS_EX_TOOLWINDOW, WS_POPUP,
            },
        },
    },
};

const HOOK_PROP_NAME: &str = "WCPOPUP_KEYBOARD_HOOK";
const LR_BUTTON_SIZE: i32 = 25;
const ROUND_CORNER_MARGIN: i32 = 3;
const SUBMENU_OFFSET: i32 = -5;
const TIMER_ID: usize = 500;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuType {
    Main,
    Submenu,
}

struct Size {
    width: i32,
    height: i32,
}

#[derive(Debug, Clone)]
struct DisplayPoint {
    x: i32,
    y: i32,
    rtl: bool,
    reverse: bool,
}

/// Popup Menu
#[derive(Debug, Clone)]
pub struct Menu {
    pub hwnd: HWND,
    pub menu_type: MenuType,
    parent: HWND,
    width: i32,
    height: i32,
}

impl Default for Menu {
    fn default() -> Self {
        Self {
            hwnd: HWND(0),
            menu_type: MenuType::Main,
            parent: HWND(0),
            height: 0,
            width: 0,
        }
    }
}

#[derive(Debug, Clone)]
struct MenuData {
    menu_type: MenuType,
    items: Vec<MenuItem>,
    h_theme: Option<Rc<HTHEME>>,
    win_subclass_id: Option<u32>,
    selected_index: i32,
    width: i32,
    height: i32,
    visible_submenu_index: i32,
    theme: Theme,
    #[cfg(feature = "accelerator")]
    haccel: Option<Rc<HACCEL>>,
    #[cfg(feature = "accelerator")]
    accelerators: HashMap<u16, String>,
    size: MenuSize,
    color: ThemeColor,
    corner: Corner,
    thread_id: u32,
    parent: HWND,
}

struct PopupInfo {
    menu_thread_id: u32,
    current_thread_id: u32,
    keyboard_hook: HHOOK,
    mouse_hook: HHOOK,
}

impl Menu {
    pub(crate) fn create_window(&self, parent: HWND, theme: Theme) -> HWND {
        create_menu_window(parent, theme).unwrap()
    }

    pub fn theme(&self) -> Theme {
        let data = get_menu_data(self.hwnd);
        data.theme
    }

    /// Sets the theme for Menu.
    pub fn set_theme(&self, theme: Theme) {
        on_theme_change(self.hwnd, Some(theme), ThemeChangeFactor::User);
    }

    /// Gets all MenuItems of Menu.
    pub fn items(&self) -> Vec<MenuItem> {
        get_menu_data(self.hwnd).items.clone()
    }

    /// Adds a MenuItem to the end of MenuItems.
    pub fn append(&mut self, mut item: MenuItem) {
        #[cfg(feature = "accelerator")]
        Self::reset_haccel(self, &item, false);

        let data = get_menu_data_mut(self.hwnd);
        item.hwnd = self.hwnd;
        data.items.push(item);
        Self::recalculate(self, data);
    }

    /// Adds a MenuItem at the specified index.
    pub fn insert(&mut self, mut item: MenuItem, index: u32) {
        #[cfg(feature = "accelerator")]
        Self::reset_haccel(self, &item, false);

        let data = get_menu_data_mut(self.hwnd);
        item.hwnd = self.hwnd;
        data.items.insert(index as usize, item);
        Self::recalculate(self, data);
    }

    /// Removes the MenuItem at the specified index.
    pub fn remove(&mut self, index: u32) {
        let data = get_menu_data_mut(self.hwnd);
        #[allow(unused_variables)]
        let removed_item = data.items.remove(index as usize);
        Self::recalculate(self, data);
        #[cfg(feature = "accelerator")]
        Self::reset_haccel(self, &removed_item, true);
    }

    pub(crate) fn attach_owner_subclass(&self, id: usize) {
        unsafe {
            let ancestor = GetAncestor(self.parent, GA_ROOTOWNER);
            let _ = SetWindowSubclass(
                if ancestor.0 == 0 {
                    self.parent
                } else {
                    ancestor
                },
                Some(menu_owner_subclass_proc),
                id,
                Box::into_raw(Box::new(self.hwnd)) as _,
            );
        }
    }

    #[cfg(feature = "accelerator")]
    fn reset_haccel(&self, item: &MenuItem, should_remove: bool) {
        if item.accelerator.is_empty() {
            return;
        }

        let hwnd = if self.menu_type == MenuType::Main {
            self.hwnd
        } else {
            unsafe { GetParent(self.hwnd) }
        };

        let data = get_menu_data_mut(hwnd);

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

        set_menu_data(hwnd, data);
    }

    fn recalculate(&mut self, data: &mut MenuData) {
        let size = Self::calculate(self, &mut data.items, &data.size, data.theme, data.corner).unwrap();
        data.width = size.width;
        data.height = size.height;
        set_menu_data(self.hwnd, data);
    }

    fn calculate(&mut self, items: &mut [MenuItem], size: &MenuSize, theme: Theme, corner: Corner) -> Result<Size, Error> {
        // Add top and left margin
        let mut width = 0;
        let mut height = size.vertical_margin;

        if corner == Corner::Round {
            height += ROUND_CORNER_MARGIN;
        }

        for (index, item) in items.iter_mut().enumerate() {
            item.index = index as i32;

            item.top = height;
            let (item_width, item_height) = measure_item(self.hwnd, size, item, theme)?;
            item.bottom = item.top + item_height;

            width = std::cmp::max(width, item_width);
            height += item_height;
        }

        // Add bottom and right margin
        width += size.horizontal_margin * 2;
        height += size.vertical_margin;

        if corner == Corner::Round {
            height += ROUND_CORNER_MARGIN;
        }

        width += size.border_size * 2;
        height += size.border_size * 2;

        self.width = width;
        self.height = height;

        Ok(Size {
            width,
            height,
        })
    }

    fn start_popup(&self, x: i32, y: i32, sync: bool) -> PopupInfo {
        let menu_thread_id = unsafe { GetWindowThreadProcessId(self.hwnd, None) };
        let current_thread_id = unsafe { GetCurrentThreadId() };

        if !sync {
            let _ = unsafe { AttachThreadInput(current_thread_id, menu_thread_id, true) };
            let data = get_menu_data_mut(self.hwnd);
            data.thread_id = current_thread_id;
            set_menu_data(self.hwnd, data);
        }

        // Activate parent window
        let _ = unsafe { SetForegroundWindow(self.parent) };
        let _ = unsafe { SetActiveWindow(self.parent) };

        let pt = get_display_point(self.hwnd, x, y, self.width, self.height);
        let _ = unsafe { SetWindowPos(self.hwnd, HWND_TOP, pt.x, pt.y, self.width, self.height, SWP_ASYNCWINDOWPOS | SWP_NOOWNERZORDER | SWP_NOACTIVATE) };

        // Set menu hwnd to property to be used in keyboard hook
        unsafe { SetPropW(self.hwnd, PCWSTR::from_raw(encode_wide(HOOK_PROP_NAME).as_ptr()), HANDLE(self.hwnd.0)).unwrap() };

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
        let _ = unsafe { ReleaseCapture() };

        let _ = unsafe { ShowWindow(self.hwnd, SW_HIDE) };

        let _ = unsafe { RemovePropW(self.hwnd, PCWSTR::from_raw(encode_wide(HOOK_PROP_NAME).as_ptr())) };

        // Unhook hooks
        let _ = unsafe { UnhookWindowsHookEx(info.keyboard_hook) };
        let _ = unsafe { UnhookWindowsHookEx(info.mouse_hook) };

        if info.current_thread_id > 0 {
            let _ = unsafe { AttachThreadInput(info.current_thread_id, info.menu_thread_id, false) };
        }
    }

    /// Shows Menu asynchronously at the specified point and returns a selected MenuItem if any.
    pub async fn popup_at_async(&self, x: i32, y: i32) -> Option<SelectedMenuItem> {
        // Prepare
        let info = Self::start_popup(self, x, y, false);

        // Show menu window
        animate_show_window(self.hwnd);
        set_capture(self.hwnd);

        let mut msg = MSG::default();
        let mut selected_item: Option<SelectedMenuItem> = None;

        async {
            while unsafe { GetMessageW(&mut msg, None, 0, 0).as_bool() } {
                match msg.message {
                    WM_MENUSELECTED => {
                        selected_item = Some(unsafe { transmute::<isize, &SelectedMenuItem>(msg.lParam.0).clone() });
                        break;
                    }

                    #[cfg(feature = "accelerator")]
                    WM_MENUCOMMAND => {
                        selected_item = Some(unsafe { transmute::<isize, &SelectedMenuItem>(msg.lParam.0).clone() });
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
    pub fn popup_at(&self, x: i32, y: i32) -> Option<SelectedMenuItem> {
        // Prepare
        let info = Self::start_popup(self, x, y, true);

        // Show menu window
        animate_show_window(self.hwnd);
        set_capture(self.hwnd);

        let mut msg = MSG::default();
        let mut selected_item: Option<SelectedMenuItem> = None;

        while unsafe { GetMessageW(&mut msg, None, 0, 0).as_bool() } {
            match msg.message {
                WM_MENUSELECTED => {
                    selected_item = Some(unsafe { transmute::<isize, &SelectedMenuItem>(msg.lParam.0).clone() });
                    break;
                }

                #[cfg(feature = "accelerator")]
                WM_MENUCOMMAND => {
                    selected_item = Some(unsafe { transmute::<isize, &SelectedMenuItem>(msg.lParam.0).clone() });
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

fn animate_show_window(hwnd: HWND) {
    let _ = unsafe { AnimateWindow(hwnd, FADE_EFFECT_TIME, AW_BLEND) };
    let _ = unsafe { ShowWindow(hwnd, SW_SHOWNOACTIVATE) };
}

fn set_capture(hwnd: HWND) {
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
                let id = init_menu_data(window);
                post_message(window, id, WM_CLOSEMENU, WPARAM(0), LPARAM(0));
            }

            LRESULT(0)
        }

        WM_DESTROY => {
            free_library();
            let data = get_menu_data_mut(window);
            if data.menu_type == MenuType::Main {
                let _ = unsafe { RemovePropW(window, PCWSTR::from_raw(encode_wide(HOOK_PROP_NAME).as_ptr())) };
                let _ = RemoveWindowSubclass(window, Some(menu_owner_subclass_proc), data.win_subclass_id.unwrap() as usize);
                let h_theme = get_h_theme(window, data);
                CloseThemeData(h_theme).unwrap();
            }

            #[cfg(feature = "accelerator")]
            if data.menu_type == MenuType::Main {
                if let Some(accel) = &data.haccel {
                    destroy_haccel(HACCEL(accel.0));
                }
            }

            DefWindowProcW(window, msg, wparam, lparam)
        }

        WM_PRINTCLIENT => {
            let hdc = HDC(wparam.0 as isize);
            let data = get_menu_data(window);
            paint_background(window, data, Some(hdc));
            let h_theme = get_h_theme(window, data);
            paint(hdc, data, &data.items, h_theme).unwrap();
            LRESULT(1)
        }

        WM_PAINT => {
            let data = get_menu_data(window);
            let h_theme = get_h_theme(window, data);
            on_paint(window, data, h_theme).unwrap();
            LRESULT(0)
        }

        WM_KEYDOWN => {
            let should_close_menu = matches!(VIRTUAL_KEY(wparam.0 as u16), VK_ESCAPE | VK_LWIN | VK_RWIN);

            if should_close_menu {
                let id = init_menu_data(window);
                post_message(window, id, WM_CLOSEMENU, WPARAM(0), LPARAM(0));
                return LRESULT(0);
            }

            #[cfg(feature = "accelerator")]
            let data = get_menu_data(window);
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

            let data = get_menu_data_mut(window);
            let maybe_index = index_of_item(data, LOWORD(wparam.0 as u32));
            if let Some((data, index)) = maybe_index {
                if on_menu_item_selected(data, index) {
                    let menu_item = SelectedMenuItem::from(&data.items[index]);
                    let id = init_menu_data(window);
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
            let id = init_menu_data(window);
            post_message(window, id, WM_CLOSEMENU, WPARAM(0), LPARAM(0));
            LRESULT(0)
        }

        WM_SETTINGCHANGE => {
            let wide_string_ptr = lparam.0 as *const u16;
            let lparam_str = PCWSTR::from_raw(wide_string_ptr).to_string().unwrap_or_default();
            if lparam_str == "ImmersiveColorSet" {
                on_theme_change(window, None, ThemeChangeFactor::SystemSetting);
            }

            DefWindowProcW(window, msg, wparam, lparam)
        }

        _ => DefWindowProcW(window, msg, wparam, lparam),
    }
}

fn on_mouse_move(window: HWND) {
    let mut pt = POINT::default();
    let _ = unsafe { GetCursorPos(&mut pt) };
    let data = get_menu_data_mut(window);

    if data.visible_submenu_index >= 0 {
        let submenu_hwnd = data.items[data.visible_submenu_index as usize].submenu.as_ref().unwrap().hwnd;
        let submenu_data = get_menu_data_mut(submenu_hwnd);
        change_selection(submenu_data, submenu_hwnd, pt);
        set_menu_data(submenu_hwnd, submenu_data);
    }

    let should_show_submenu = change_selection(data, window, pt);
    change_selection(data, window, pt);
    set_menu_data(window, data);

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
        let id = init_menu_data(window);
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
    let data = get_menu_data_mut(hwnd);
    let index = index_from_point(hwnd, get_cursor_point(window), data);

    if index < 0 {
        return;
    }

    if on_menu_item_selected(data, index as usize) {
        set_menu_data(hwnd, data);
        let menu_item = SelectedMenuItem::from(&data.items[index as usize]);
        let id = init_menu_data(window);
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
    if (data.items[index].state.0 & MENU_DISABLED.0) != 0 {
        return false;
    }

    // toggle checkbox
    if data.items[index].menu_item_type == MenuItemType::Checkbox {
        let checked = (data.items[index].state.0 & MENU_CHECKED.0) != 0;
        toggle_checked(&mut data.items[index], !checked);
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

    unsafe { GetAncestor(child, GA_PARENT) }
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
            on_theme_change(hwnd, None, ThemeChangeFactor::App);
            DefSubclassProc(window, msg, wparam, lparam)
        }

        _ => DefSubclassProc(window, msg, wparam, lparam),
    }
}

fn init_menu_data(hwnd: HWND) -> u32 {
    let data = get_menu_data_mut(hwnd);

    let thread_id = if data.menu_type == MenuType::Main {
        data.thread_id
    } else {
        let parent_data = get_menu_data(data.parent);
        parent_data.thread_id
    };

    data.selected_index = -1;
    data.thread_id = 0;

    if data.visible_submenu_index >= 0 {
        let submenu_hwnd = data.items[data.visible_submenu_index as usize].submenu.as_ref().unwrap().hwnd;
        hide_submenu(submenu_hwnd);
    }
    data.visible_submenu_index = -1;

    set_menu_data(hwnd, data);

    thread_id
}

#[cfg(feature = "accelerator")]
fn index_of_item(data: &mut MenuData, id: u16) -> Option<(&mut MenuData, usize)> {
    for (index, item) in data.items.iter().enumerate() {
        if item.menu_item_type == MenuItemType::Submenu {
            let hwnd = item.submenu.as_ref().unwrap().hwnd;
            let submenu_data = get_menu_data_mut(hwnd);
            if let Some(index) = index_of_item(submenu_data, id) {
                return Some(index);
            }
        } else if item.uuid == id {
            return Some((data, index));
        }
    }
    None
}

fn get_color_scheme(data: &MenuData) -> &ColorScheme {
    let is_dark = if data.theme == Theme::System {
        is_sys_dark_color()
    } else {
        data.theme == Theme::Dark
    };

    if is_dark {
        &data.color.dark
    } else {
        &data.color.light
    }
}

fn paint_background(hwnd: HWND, data: &MenuData, hdc: Option<HDC>) {
    unsafe {
        let dc = if let Some(hdc) = hdc {
            hdc
        } else {
            GetWindowDC(hwnd)
        };

        if dc.0 == 0 {
            return;
        }

        let scheme = get_color_scheme(data);

        let mut client_rect = RECT::default();
        GetClientRect(hwnd, &mut client_rect).unwrap();

        let hbr = CreateSolidBrush(COLORREF(scheme.border));
        FillRect(dc, &client_rect, hbr);
        let _ = DeleteObject(hbr);

        let menu_rect = RECT {
            left: client_rect.left + data.size.border_size,
            top: client_rect.top + data.size.border_size,
            right: client_rect.right - data.size.border_size,
            bottom: client_rect.bottom - data.size.border_size,
        };

        let hbr = CreateSolidBrush(COLORREF(scheme.background_color));
        FillRect(dc, &menu_rect, hbr);
        let _ = DeleteObject(hbr);

        if hdc.is_none() {
            ReleaseDC(hwnd, dc);
        }
    }
}

fn on_paint(hwnd: HWND, data: &MenuData, h_theme: HTHEME) -> Result<(), Error> {
    let mut paint_struct = PAINTSTRUCT::default();
    let dc = unsafe { BeginPaint(hwnd, &mut paint_struct) };

    if dc.0 == 0 {
        return Ok(());
    }

    let index = index_from_rect(data, paint_struct.rcPaint);

    if index.is_none() {
        paint(dc, data, &data.items, h_theme)?;
    } else {
        paint(dc, data, &vec![data.items[index.unwrap() as usize].clone()], h_theme)?;
    }

    let _ = unsafe { EndPaint(hwnd, &paint_struct) };

    Ok(())
}

fn paint(dc: HDC, data: &MenuData, items: &Vec<MenuItem>, h_theme: HTHEME) -> Result<(), Error> {
    let scheme = get_color_scheme(data);
    let selected_color = unsafe { CreateSolidBrush(COLORREF(scheme.hover_background_color)) };
    let normal_color = unsafe { CreateSolidBrush(COLORREF(scheme.background_color)) };

    for item in items {
        let whole_item_rect = get_item_rect(data, item);

        let disabled = (item.state.0 & MENU_DISABLED.0) != 0;
        let checked = (item.state.0 & MENU_CHECKED.0) != 0;

        if item.index == data.selected_index && !disabled {
            unsafe { FillRect(dc, &whole_item_rect, selected_color) };
        } else {
            unsafe { FillRect(dc, &whole_item_rect, normal_color) };
        }

        // Rect for text, checkmark and submenu icon
        let border_size = data.size.border_size;
        let item_rect = RECT {
            left: data.size.horizontal_margin + border_size,
            top: item.top + border_size,
            right: data.width - data.size.horizontal_margin - border_size,
            bottom: item.bottom + border_size,
        };

        match item.menu_item_type {
            MenuItemType::Separator => {
                // Use whole item rect to draw from left to right
                draw_separator(dc, scheme, whole_item_rect)?;
            }

            _ => {
                if checked {
                    let mut rect = RECT {
                        left: item_rect.left,
                        top: item_rect.top,
                        right: item_rect.left + LR_BUTTON_SIZE,
                        bottom: item_rect.top + LR_BUTTON_SIZE,
                    };
                    // center vertically
                    let _ = unsafe { OffsetRect(&mut rect, 0, ((item_rect.bottom - item_rect.top) - (rect.bottom - rect.top)) / 2) };
                    let mut check_rect = rect;
                    let _ = unsafe { InflateRect(&mut check_rect as *mut _, -1, -1) };
                    unsafe { DrawThemeBackgroundEx(h_theme, dc, MENU_POPUPCHECK.0, MC_CHECKMARKNORMAL.0, &check_rect, None)? };
                }

                let mut text_rect = item_rect;
                // Keep space for check mark and submenu mark
                text_rect.left += LR_BUTTON_SIZE;
                text_rect.right -= LR_BUTTON_SIZE;

                if item.menu_item_type == MenuItemType::Submenu {
                    let mut arrow_rect = item_rect;
                    let arrow_size = unsafe { GetSystemMetrics(SM_CXHSCROLL) };
                    arrow_rect.left = item_rect.right - arrow_size;

                    // center vertically
                    let _ = unsafe { OffsetRect(&mut arrow_rect as *mut _, 0, ((item_rect.bottom - item_rect.top) - (arrow_rect.bottom - arrow_rect.top)) / 2) };
                    unsafe { DrawThemeBackgroundEx(h_theme, dc, MENU_POPUPSUBMENU.0, MSM_NORMAL.0, &arrow_rect, None)? };
                }

                draw_menu_text(dc, scheme, &text_rect, item, data, disabled)?;
                unsafe { ExcludeClipRect(dc, item_rect.left, item_rect.top, item_rect.right, item_rect.bottom) };
            }
        }
    }

    let _ = unsafe { DeleteObject(selected_color) };
    let _ = unsafe { DeleteObject(normal_color) };

    Ok(())
}

fn draw_separator(dc: HDC, scheme: &ColorScheme, rect: RECT) -> Result<(), Error> {
    let mut separator_rect = rect;

    separator_rect.top += (rect.bottom - rect.top) / 2;

    let pen = unsafe { CreatePen(PS_SOLID, 1, COLORREF(scheme.border)) };
    let old_pen = unsafe { SelectObject(dc, pen) };
    let _ = unsafe { MoveToEx(dc, separator_rect.left, separator_rect.top, None) };
    let _ = unsafe { LineTo(dc, separator_rect.right, separator_rect.top) };
    unsafe { SelectObject(dc, old_pen) };
    let _ = unsafe { DeleteObject(pen) };

    Ok(())
}

fn draw_menu_text(dc: HDC, scheme: &ColorScheme, rect: &RECT, item: &MenuItem, data: &MenuData, disabled: bool) -> Result<(), Error> {
    let mut text_rect = *rect;

    unsafe { SetBkMode(dc, TRANSPARENT) };
    if disabled {
        unsafe { SetTextColor(dc, COLORREF(scheme.disabled)) };
    } else {
        unsafe { SetTextColor(dc, COLORREF(scheme.color)) };
    }

    let menu_font = get_font(data.theme, &data.size, false)?;
    let font = unsafe { CreateFontIndirectW(&menu_font) };
    let old_font = unsafe { SelectObject(dc, font) };

    unsafe { DrawTextW(dc, &mut encode_wide(&item.label), &mut text_rect, DT_SINGLELINE | DT_LEFT | DT_VCENTER) };

    if !item.accelerator.is_empty() {
        unsafe { SetTextColor(dc, COLORREF(scheme.disabled)) };
        unsafe { DrawTextW(dc, &mut encode_wide(&item.accelerator), &mut text_rect, DT_SINGLELINE | DT_RIGHT | DT_VCENTER) };
    }

    unsafe { SelectObject(dc, old_font) };
    let _ = unsafe { DeleteObject(font) };

    Ok(())
}

fn get_display_point(hwnd: HWND, x: i32, y: i32, cx: i32, cy: i32) -> DisplayPoint {
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

fn change_selection(data: &mut MenuData, hwnd: HWND, screen_point: POINT) -> bool {
    // Menu can be not visible yet due to timer or animation
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
            let rect = get_item_rect(data, item);
            let _ = unsafe { InvalidateRect(hwnd, Some(&rect), false) };
        }

        if data.selected_index >= 0 {
            let item = &data.items[data.selected_index as usize];
            let rect = get_item_rect(data, item);
            let _ = unsafe { InvalidateRect(hwnd, Some(&rect), false) };
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
        let hwnd = data.items[data.visible_submenu_index as usize].submenu.as_ref().unwrap().hwnd;
        hide_submenu(hwnd);
        data.visible_submenu_index = -1;
    }

    if data.visible_submenu_index < 0 && data.items[selected_index as usize].menu_item_type == MenuItemType::Submenu {
        data.visible_submenu_index = selected_index;
        should_show_submenu = true;
    }

    should_show_submenu
}

fn show_submenu(hwnd: HWND) {
    let proc: TIMERPROC = Some(delay_show_submenu);
    let mut show_delay: u32 = 0;
    let _ = unsafe { SystemParametersInfoW(SPI_GETMENUSHOWDELAY, 0, Some(&mut show_delay as *mut _ as *mut c_void), SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0)) };
    unsafe { SetTimer(hwnd, TIMER_ID, show_delay, proc) };
}

unsafe extern "system" fn delay_show_submenu(hwnd: HWND, _msg: u32, id: usize, _time: u32) {
    KillTimer(hwnd, id).unwrap();

    let main_menu_data = get_menu_data(hwnd);

    if main_menu_data.visible_submenu_index >= 0 {
        let mut main_menu_rect = RECT::default();
        GetWindowRect(hwnd, &mut main_menu_rect).unwrap();
        let item = &main_menu_data.items[main_menu_data.visible_submenu_index as usize];
        let submenu_hwnd = item.submenu.as_ref().unwrap().hwnd;
        let submenu_data = get_menu_data(submenu_hwnd);

        let pt = get_display_point(submenu_hwnd, main_menu_rect.right, main_menu_rect.top + item.top, submenu_data.width, submenu_data.height);

        let x = if pt.rtl {
            main_menu_rect.left - submenu_data.width - SUBMENU_OFFSET
        } else {
            main_menu_rect.right + SUBMENU_OFFSET
        };

        let round_corner_margin = if main_menu_data.corner == Corner::Round {
            ROUND_CORNER_MARGIN
        } else {
            0
        };
        let y = if pt.reverse {
            let mut reversed_point = POINT {
                x: 0,
                y: item.bottom - submenu_data.height,
            };
            let _ = ClientToScreen(hwnd, &mut reversed_point);
            // Add top + bottom margin
            reversed_point.y + main_menu_data.size.vertical_margin * 2 + round_corner_margin
        } else {
            // Reduce top margin
            main_menu_rect.top + item.top - main_menu_data.size.vertical_margin - round_corner_margin
        };

        SetWindowPos(submenu_hwnd, HWND_TOP, x, y, submenu_data.width, submenu_data.height, SWP_ASYNCWINDOWPOS | SWP_NOOWNERZORDER | SWP_NOACTIVATE).unwrap();
        animate_show_window(submenu_hwnd);
    }
}

fn hide_submenu(hwnd: HWND) {
    let data = get_menu_data_mut(hwnd);
    data.selected_index = -1;
    set_menu_data(hwnd, data);
    let _ = unsafe { ShowWindow(hwnd, SW_HIDE) };
}

fn get_item_rect(data: &MenuData, item: &MenuItem) -> RECT {
    let border_size = data.size.border_size;
    RECT {
        left: border_size,
        top: item.top + border_size,
        right: data.width - border_size,
        bottom: item.bottom + border_size,
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
    let data = get_menu_data(hwnd);
    let submenu = if data.visible_submenu_index >= 0 {
        data.items[data.visible_submenu_index as usize].submenu.as_ref().unwrap().hwnd
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

fn get_h_theme(hwnd: HWND, data: &MenuData) -> HTHEME {
    if data.h_theme.is_some() {
        return HTHEME(data.h_theme.as_ref().unwrap().0);
    }

    let parent = unsafe { GetParent(hwnd) };
    let parent_data = get_menu_data(parent);
    HTHEME(parent_data.h_theme.as_ref().unwrap().0)
}

fn on_theme_change(hwnd: HWND, maybe_preferred_theme: Option<Theme>, factor: ThemeChangeFactor) {
    let data = get_menu_data_mut(hwnd);
    if data.menu_type == MenuType::Submenu {
        return;
    }

    let current_them = data.theme;

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

    allow_dark_mode_for_window(hwnd, should_be_dark);
    set_preferred_app_mode(new_theme);

    let old_htheme = get_h_theme(hwnd, data);
    unsafe { CloseThemeData(old_htheme).unwrap() };
    let h_theme = unsafe { OpenThemeDataEx(hwnd, w!("Menu"), OTD_NONCLIENT) };
    data.h_theme = Some(Rc::new(h_theme));

    data.theme = new_theme;
    set_menu_data(hwnd, data);
    let _ = unsafe { UpdateWindow(hwnd) };

    for menu_item in &data.items {
        let item = menu_item;
        if item.menu_item_type == MenuItemType::Submenu {
            let submenu_hwnd = item.submenu.as_ref().unwrap().hwnd;
            let data = get_menu_data_mut(submenu_hwnd);
            data.theme = new_theme;
            set_menu_data(submenu_hwnd, data);
            let _ = unsafe { UpdateWindow(submenu_hwnd) };
        }
    }
}

fn create_menu_window(parent: HWND, theme: Theme) -> Result<HWND, Error> {
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
    let ex_style = WS_EX_TOOLWINDOW;

    let hwnd = unsafe { CreateWindowExW(ex_style, PCWSTR::from_raw(class_name.as_ptr()), PCWSTR::null(), window_styles, 0, 0, 0, 0, parent, None, GetModuleHandleW(PCWSTR::null()).unwrap_or_default(), None) };
    let _ = unsafe { SetWindowPos(hwnd, None, 0, 0, 0, 0, SWP_NOZORDER | SWP_NOOWNERZORDER | SWP_NOMOVE | SWP_NOSIZE | SWP_FRAMECHANGED) };

    let should_be_dark = if theme == Theme::System {
        is_sys_dark_color()
    } else {
        theme == Theme::Dark
    };
    allow_dark_mode_for_window(hwnd, should_be_dark);
    set_preferred_app_mode(theme);

    Ok(hwnd)
}
