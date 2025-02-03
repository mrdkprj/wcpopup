mod accelerator;
mod builder;
mod direct2d;
mod menu_item;
mod util;
use crate::{config::*, InnerMenuEvent, MenuEvent, MenuItemType, MenuType, ThemeChangeFactor};
#[cfg(feature = "accelerator")]
use accelerator::{create_haccel, destroy_haccel, translate_accel};
pub use builder::*;
use direct2d::{colorref_to_d2d1_color_f, create_menu_image, create_write_factory, get_device_context, get_text_format, set_fill_color, set_stroke_color, to_2d_rect, TextAlignment};
pub use menu_item::*;
use serde::{Deserialize, Serialize};
use std::mem::size_of;
#[cfg(feature = "accelerator")]
use std::rc::Rc;
use util::*;
use windows::Win32::Graphics::Direct2D::D2D1_BITMAP_INTERPOLATION_MODE_NEAREST_NEIGHBOR;
#[cfg(feature = "accelerator")]
use windows::Win32::UI::WindowsAndMessaging::{MSG, WM_COMMAND, WM_SYSCOMMAND};
use windows::{
    core::{w, Error, PCWSTR},
    Foundation::Numerics::Matrix3x2,
    Win32::{
        Foundation::{HANDLE, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM},
        Graphics::{
            Direct2D::{Common::D2D_POINT_2F, D2D1_DRAW_TEXT_OPTIONS_NONE, D2D1_ROUNDED_RECT},
            DirectWrite::{DWRITE_MEASURING_MODE_NATURAL, DWRITE_TEXT_ALIGNMENT_TRAILING},
            Gdi::{
                BeginPaint, ClientToScreen, EndPaint, GetMonitorInfoW, GetWindowDC, InvalidateRect, MonitorFromPoint, MonitorFromWindow, PtInRect, ReleaseDC, ScreenToClient, UpdateWindow, HBRUSH,
                HDC, MONITORINFO, MONITOR_DEFAULTTONEAREST, MONITOR_DEFAULTTONULL, PAINTSTRUCT,
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
                AnimateWindow, CallNextHookEx, CreateWindowExW, DefWindowProcW, DestroyWindow, GetAncestor, GetClientRect, GetCursorPos, GetParent, GetPropW, GetWindow, GetWindowRect,
                GetWindowThreadProcessId, IsWindow, IsWindowVisible, KillTimer, LoadCursorW, PostMessageW, RegisterClassExW, RemovePropW, SetCursor, SetForegroundWindow, SetPropW, SetTimer,
                SetWindowPos, SetWindowsHookExW, ShowWindow, SystemParametersInfoW, UnhookWindowsHookEx, WindowFromPoint, AW_BLEND, CS_DROPSHADOW, CS_HREDRAW, CS_VREDRAW, GA_ROOTOWNER, GW_OWNER,
                HCURSOR, HHOOK, HICON, HWND_TOP, IDC_ARROW, SPI_GETMENUSHOWDELAY, SWP_ASYNCWINDOWPOS, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOOWNERZORDER, SWP_NOSIZE, SWP_NOZORDER,
                SW_HIDE, SW_SHOWNOACTIVATE, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, TIMERPROC, WH_KEYBOARD, WH_MOUSE, WM_ACTIVATE, WM_APP, WM_DESTROY, WM_ERASEBKGND, WM_KEYDOWN, WM_LBUTTONDOWN,
                WM_LBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_PAINT, WM_PRINTCLIENT, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SETTINGCHANGE, WM_THEMECHANGED, WNDCLASSEXW, WS_CLIPSIBLINGS, WS_EX_LAYERED,
                WS_EX_TOOLWINDOW, WS_POPUP,
            },
        },
    },
};

const HOOK_PROP_NAME: &str = "WCPOPUP_KEYBOARD_HOOK";
/* https://learn.microsoft.com/en-us/windows/apps/design/signature-experiences/geometry */
pub(crate) const CORNER_RADIUS: i32 = 8;
const SHOW_SUBMENU_TIMER_ID: usize = 500;
const HIDE_SUBMENU_TIMER_ID: usize = 501;
const FADE_EFFECT_TIME: u32 = 120;

const WM_INACTIVATE: u32 = WM_APP + 0x0004;

/// Context Menu.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Menu {
    pub window_handle: isize,
    pub menu_type: MenuType,
    parent_window_handle: isize,
}

impl Default for Menu {
    fn default() -> Self {
        Self {
            window_handle: 0,
            menu_type: MenuType::Main,
            parent_window_handle: 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Size {
    width: i32,
    height: i32,
}

#[derive(Default, Clone, Copy, Debug)]
pub(crate) struct IconSpace {
    left: IconSize,
    mid: IconSize,
    right: IconSize,
}

#[derive(Default, Clone, Copy, Debug)]
pub(crate) struct IconSize {
    width: i32,
    lmargin: i32,
    rmargin: i32,
}

#[derive(Debug, Clone)]
struct DisplayPoint {
    x: i32,
    y: i32,
    rtl: bool,
    reverse: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct PopupInfo {
    window_handle: isize,
    thread_attached: bool,
    current_thread_id: u32,
    menu_thread_id: u32,
    keyboard_hook: isize,
    mouse_hook: isize,
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
    pub fn append(&mut self, item: MenuItem) {
        self.add_item(item, None);
    }

    /// Adds a MenuItem at the specified index.
    pub fn insert(&mut self, item: MenuItem, index: u32) {
        self.add_item(item, Some(index as usize));
    }

    fn add_item(&mut self, mut item: MenuItem, index: Option<usize>) {
        let data = get_menu_data_mut(self.window_handle);
        if item.menu_item_type == MenuItemType::Submenu {
            self.create_submenu(data, &mut item);
        }

        #[cfg(feature = "accelerator")]
        self.reset_haccel(&item, false);

        item.menu_window_handle = self.window_handle;
        refresh_menu_icon(data, &item, false);

        /* Add before recalc */
        if let Some(index) = index {
            data.items.insert(index, item);
        } else {
            data.items.push(item);
        }

        recalculate(data);

        if let Some(index) = index {
            Self::reset_radio(data, data.items[index].clone());
        } else {
            Self::reset_radio(data, data.items.last().unwrap().clone());
        }

        set_menu_data(self.window_handle, data);
    }

    fn create_submenu(&mut self, data: &MenuData, item: &mut MenuItem) {
        let mut builder = MenuBuilder::new_for_submenu(self, &data.config, data.current_theme, item.items.as_mut().unwrap());
        let memnu = builder.build().unwrap();
        item.submenu = Some(memnu);
    }

    fn reset_radio(data: &mut MenuData, new_item: MenuItem) {
        if new_item.menu_item_type == MenuItemType::Radio && new_item.checked {
            toggle_radio(data, new_item.index as usize);
        }
    }

    /// Removes the MenuItem at the specified index.
    pub fn remove_at(&mut self, index: u32) {
        let data = get_menu_data_mut(self.window_handle);
        if index as usize > data.items.len() - 1 {
            return;
        }

        let removed_item = data.items.remove(index as usize);
        refresh_menu_icon(data, &removed_item, true);
        recalculate(data);

        #[cfg(feature = "accelerator")]
        self.reset_haccel(&removed_item, true);

        if removed_item.menu_item_type == MenuItemType::Submenu {
            let _ = unsafe { DestroyWindow(hwnd!(removed_item.submenu.unwrap().window_handle)) };
        }

        set_menu_data(self.window_handle, data);
    }

    /// Removes the MenuItem.
    pub fn remove(&mut self, item: &MenuItem) {
        let data = get_menu_data_mut(self.window_handle);
        let maybe_index = data.items.iter().position(|i| i.uuid == item.uuid);
        if let Some(index) = maybe_index {
            self.remove_at(index as u32);
        }
    }

    #[cfg(feature = "accelerator")]
    fn reset_haccel(&self, item: &MenuItem, should_remove: bool) {
        let items = if item.menu_item_type == MenuItemType::Submenu {
            &item.submenu.as_ref().unwrap().items()
        } else {
            &vec![item.clone()]
        };

        let mut items_with_accel = Vec::new();
        for item in items {
            if !item.accelerator.is_empty() {
                items_with_accel.push(item);
            }
        }

        if items_with_accel.is_empty() {
            return;
        }

        let window_handle = if self.menu_type == MenuType::Main {
            self.window_handle
        } else {
            unsafe { GetParent(hwnd!(self.window_handle)).unwrap().0 as _ }
        };

        let data = get_menu_data_mut(window_handle);

        let mut accelerators = data.accelerators.clone();

        for item_with_accel in items_with_accel {
            if should_remove {
                accelerators.remove_entry(&item_with_accel.uuid);
            } else {
                accelerators.insert(item_with_accel.uuid, item_with_accel.accelerator.clone());
            }
        }

        destroy_haccel(data);

        match create_haccel(&accelerators) {
            Some(accel) => data.haccel = Some(Rc::new(accel)),
            None => data.haccel = None,
        }

        data.accelerators = accelerators;

        set_menu_data(window_handle, data);
    }

    pub(crate) fn attach_owner_subclass(&self, id: usize) {
        let hwnd = hwnd!(self.window_handle);

        let _ = unsafe { SetWindowSubclass(get_parent_hwnd(self.parent_window_handle), Some(menu_owner_subclass_proc), id, Box::into_raw(Box::new(hwnd)) as _) };
    }

    fn start_popup(&self, x: i32, y: i32, size: Size, attach_thread: bool) {
        let hwnd = hwnd!(self.window_handle);
        let parent = hwnd!(self.parent_window_handle);
        let menu_thread_id = unsafe { GetWindowThreadProcessId(hwnd, None) };
        let current_thread_id = unsafe { GetCurrentThreadId() };

        if attach_thread {
            let _ = unsafe { AttachThreadInput(current_thread_id, menu_thread_id, true) };
        }

        /* Activate parent window */
        let _ = unsafe { SetForegroundWindow(parent) };
        let _ = unsafe { SetActiveWindow(parent) };

        let pt = get_display_point(self.window_handle, x, y, size.width, size.height);
        let _ = unsafe { SetWindowPos(hwnd, HWND_TOP, pt.x, pt.y, size.width, size.height, SWP_ASYNCWINDOWPOS | SWP_NOOWNERZORDER | SWP_NOACTIVATE) };

        /* Set menu hwnd to property to be used in keyboard hook */
        unsafe { SetPropW(hwnd, to_pcwstr(HOOK_PROP_NAME), HANDLE(self.window_handle as _)).unwrap() };

        /* Set hooks */
        let keyboard_hook = unsafe { SetWindowsHookExW(WH_KEYBOARD, Some(keyboard_hook), None, menu_thread_id).unwrap() };
        let mouse_hook = unsafe { SetWindowsHookExW(WH_MOUSE, Some(mouse_hook), None, menu_thread_id).unwrap() };

        let info = PopupInfo {
            window_handle: self.window_handle,
            thread_attached: attach_thread,
            menu_thread_id,
            current_thread_id,
            keyboard_hook: vtoi!(keyboard_hook.0),
            mouse_hook: vtoi!(mouse_hook.0),
        };

        let data = get_menu_data_mut(self.window_handle);
        data.popup_info = Some(info);
        set_menu_data(self.window_handle, data);
    }

    /// Shows Menu at the specified point.
    pub fn popup_at(&self, x: i32, y: i32) {
        let data = get_menu_data(self.window_handle);
        self.start_popup(x, y, data.size, false);

        animate_show_window(self.window_handle);
        set_capture(self.window_handle);
    }

    /// Shows Menu asynchronously at the specified point and returns the selected MenuItem if any.
    pub async fn popup_at_async(&self, x: i32, y: i32) -> Option<MenuItem> {
        let data = get_menu_data(self.window_handle);
        self.start_popup(x, y, data.size, true);

        animate_show_window(self.window_handle);
        set_capture(self.window_handle);

        let mut item = None;

        if let Ok(event) = MenuEvent::innner_receiver().recv().await {
            item = event.item;
        }

        item
    }
}

fn get_parent_hwnd(parent_window_handle: isize) -> HWND {
    let parent_hwnd = hwnd!(parent_window_handle);
    let ancestor = unsafe { GetAncestor(parent_hwnd, GA_ROOTOWNER) };
    if ancestor.0.is_null() {
        parent_hwnd
    } else {
        ancestor
    }
}

fn refresh_menu_icon(data: &mut MenuData, item: &MenuItem, should_remove: bool) {
    if let Some(icon) = &item.icon {
        if should_remove {
            let _ = data.icon_map.remove(&item.uuid);
        } else {
            let bitmap = create_menu_image(&data.dc_render_target, icon, data.icon_space.left.width);
            data.icon_map.insert(item.uuid, bitmap);
        }
    }
}

fn animate_show_window(window_handle: isize) {
    let hwnd = hwnd!(window_handle);
    let _ = unsafe { AnimateWindow(hwnd, FADE_EFFECT_TIME, AW_BLEND) };
    let _ = unsafe { ShowWindow(hwnd, SW_SHOWNOACTIVATE) };
}

fn set_capture(window_handle: isize) {
    let hwnd = hwnd!(window_handle);
    /* Prevent mouse input on window beneath menu */
    unsafe { SetCapture(hwnd) };

    let cursor = unsafe { LoadCursorW(None, IDC_ARROW).unwrap() };
    let _ = unsafe { SetCursor(cursor) };
}

unsafe extern "system" fn keyboard_hook(ncode: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    /* Prevent keyboard input while Menu is open */
    if ncode >= 0 {
        let capture_window = unsafe { GetCapture() };
        let data = unsafe { GetPropW(capture_window, to_pcwstr(HOOK_PROP_NAME)) };

        unsafe { PostMessageW(HWND(data.0), WM_KEYDOWN, wparam, lparam).unwrap() };
        return LRESULT(1);
    }

    CallNextHookEx(None, ncode, wparam, lparam)
}

unsafe extern "system" fn mouse_hook(ncode: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if ncode >= 0 {
        let capture_window = unsafe { GetCapture() };
        let data = unsafe { GetPropW(capture_window, to_pcwstr(HOOK_PROP_NAME)) };

        match wparam.0 as u32 {
            /* Do not direct buttondown event since it is sent to default_window_proc */
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
                init_menu_data(vtoi!(window.0));
                post_message(None);
            }

            LRESULT(0)
        }

        WM_DESTROY => {
            let data = get_menu_data_mut(vtoi!(window.0));

            if data.menu_type == MenuType::Main {
                free_library();
                let _ = unsafe { RemovePropW(window, to_pcwstr(HOOK_PROP_NAME)) };
                if let Ok(parent) = GetParent(window) {
                    let hwnd = get_parent_hwnd(parent.0 as _);
                    let _ = RemoveWindowSubclass(hwnd, Some(menu_owner_subclass_proc), data.win_subclass_id.unwrap() as usize);
                }
            }

            #[cfg(feature = "accelerator")]
            if data.menu_type == MenuType::Main {
                destroy_haccel(data);
            }

            let _ = Box::from_raw(data);

            DefWindowProcW(window, msg, wparam, lparam)
        }

        WM_PRINTCLIENT => {
            let hdc = HDC(wparam.0 as _);
            let data = get_menu_data(vtoi!(window.0));
            paint_background(window, data, Some(hdc));
            paint(hdc, data, &data.items).unwrap();
            LRESULT(1)
        }

        WM_ERASEBKGND => {
            let data = get_menu_data(vtoi!(window.0));
            paint_background(window, data, None);
            LRESULT(0)
        }

        WM_PAINT => {
            let data = get_menu_data(vtoi!(window.0));
            on_paint(window, data).unwrap();
            LRESULT(0)
        }

        WM_KEYDOWN => {
            let should_close_menu = matches!(VIRTUAL_KEY(wparam.0 as u16), VK_ESCAPE | VK_LWIN | VK_RWIN);

            if should_close_menu {
                init_menu_data(vtoi!(window.0));
                post_message(None);
                return LRESULT(0);
            }

            #[cfg(feature = "accelerator")]
            {
                let keydown_msg = MSG {
                    hwnd: window,
                    wParam: wparam,
                    lParam: lparam,
                    message: msg,
                    time: 0,
                    pt: POINT::default(),
                };
                translate_accel(window, keydown_msg);
            }

            LRESULT(0)
        }

        #[cfg(feature = "accelerator")]
        WM_COMMAND | WM_SYSCOMMAND => {
            if HIWORD(wparam.0 as u32) != 1 {
                return LRESULT(0);
            }

            let data = get_menu_data_mut(vtoi!(window.0));
            let maybe_index = index_of_item(data, LOWORD(wparam.0 as u32));
            if let Some((data, index)) = maybe_index {
                if on_menu_item_selected(data, index) {
                    let menu_item = data.items[index].clone();
                    init_menu_data(vtoi!(window.0));
                    post_message(Some(&menu_item));
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
            init_menu_data(vtoi!(window.0));
            post_message(None);
            LRESULT(0)
        }

        WM_SETTINGCHANGE => {
            let wide_string_ptr = lparam.0 as *const u16;
            let lparam_str = PCWSTR::from_raw(wide_string_ptr).to_string().unwrap_or_default();
            if lparam_str == "ImmersiveColorSet" {
                on_theme_change(vtoi!(window.0), None, ThemeChangeFactor::SystemSetting);
            }

            DefWindowProcW(window, msg, wparam, lparam)
        }

        _ => DefWindowProcW(window, msg, wparam, lparam),
    }
}

fn on_mouse_move(window: HWND) {
    let mut pt = POINT::default();
    let _ = unsafe { GetCursorPos(&mut pt) };
    let window_handle = vtoi!(window.0);
    let data = get_menu_data_mut(window_handle);

    if data.visible_submenu_index >= 0 {
        let submenu_window_handle = data.items[data.visible_submenu_index as usize].submenu.as_ref().unwrap().window_handle;
        let submenu_data = get_menu_data_mut(submenu_window_handle);
        change_selection(submenu_data, submenu_window_handle, pt);
    }

    let changed = change_selection(data, window_handle, pt);

    if changed {
        toggle_submenu(window_handle, data);
    }
}

fn on_mouse_down(window: HWND, msg: u32) {
    /* If mouse input occurs outside of menu */
    if get_hwnd_from_point(window).is_none() {
        /* Immediately release capture so that the event is sent to the target window */
        let _ = unsafe { ReleaseCapture() };

        /* Close menu */
        init_menu_data(vtoi!(window.0));
        post_message(None);

        /* If mouse input occurs on parent window, send mouse input */
        send_mouse_input(window, msg);
    }
}

fn on_mouse_up(window: HWND) {
    let maybe_hwnd = get_hwnd_from_point(window);
    if maybe_hwnd.is_none() {
        return;
    }

    let hwnd = maybe_hwnd.unwrap();
    let data = get_menu_data_mut(vtoi!(hwnd.0));
    let index = index_from_point(hwnd, get_cursor_point(window), data);

    if index < 0 {
        return;
    }

    if on_menu_item_selected(data, index as usize) {
        set_menu_data(vtoi!(hwnd.0), data);
        let menu_item = data.items[index as usize].clone();
        init_menu_data(vtoi!(window.0));
        post_message(Some(&menu_item));
    }
}

fn post_message(menu_item: Option<&MenuItem>) {
    if let Some(item) = menu_item {
        MenuEvent::send(MenuEvent {
            item: item.clone(),
        });
        MenuEvent::send_inner(InnerMenuEvent {
            item: Some(item.clone()),
        });
    } else {
        MenuEvent::send_inner(InnerMenuEvent {
            item: None,
        });
    }
}

fn on_menu_item_selected(data: &mut MenuData, index: usize) -> bool {
    /* Ignore submenu */
    if data.items[index].menu_item_type == MenuItemType::Submenu {
        return false;
    }

    /* Ignore invisible */
    if !data.items[index].visible {
        return false;
    }

    /* Ignore disabled */
    if data.items[index].disabled {
        return false;
    }

    /* Toggle radio checkbox */
    if data.items[index].menu_item_type == MenuItemType::Radio {
        toggle_radio(data, index);
    }

    /* Toggle checkbox */
    if data.items[index].menu_item_type == MenuItemType::Checkbox {
        data.items[index].checked = !data.items[index].checked;
    }

    true
}

fn get_parent_window(child: HWND) -> HWND {
    if let Ok(owner) = unsafe { GetWindow(child, GW_OWNER) } {
        return owner;
    }

    if let Ok(parent) = unsafe { GetParent(child) } {
        return parent;
    }

    HWND(std::ptr::null_mut())
}

fn send_mouse_input(hwnd: HWND, msg: u32) {
    let mut count = 0;

    let mut parent = get_parent_window(hwnd);
    let mut cursor_point = POINT::default();
    let _ = unsafe { GetCursorPos(&mut cursor_point) };

    while !parent.0.is_null() {
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
            on_theme_change(vtoi!(hwnd.0), None, ThemeChangeFactor::App);
            DefSubclassProc(window, msg, wparam, lparam)
        }

        _ => DefSubclassProc(window, msg, wparam, lparam),
    }
}

fn finish_popup(info: &PopupInfo) {
    let hwnd = hwnd!(info.window_handle);
    let _ = unsafe { ReleaseCapture() };

    let _ = unsafe { ShowWindow(hwnd, SW_HIDE) };

    let _ = unsafe { RemovePropW(hwnd, to_pcwstr(HOOK_PROP_NAME)) };

    /* Unhook hooks */
    let _ = unsafe { UnhookWindowsHookEx(HHOOK(info.keyboard_hook as _)) };
    let _ = unsafe { UnhookWindowsHookEx(HHOOK(info.mouse_hook as _)) };

    if info.thread_attached {
        let _ = unsafe { AttachThreadInput(info.current_thread_id, info.menu_thread_id, false) };
    }
}

fn init_menu_data(window_handle: isize) {
    let data = get_menu_data_mut(window_handle);

    data.selected_index = -1;

    if data.visible_submenu_index >= 0 {
        let submenu_window_handle = data.items[data.visible_submenu_index as usize].submenu.as_ref().unwrap().window_handle;
        hide_submenu(submenu_window_handle);
    }
    data.visible_submenu_index = -1;

    if data.menu_type == MenuType::Main {
        let info = data.popup_info.as_ref().unwrap();
        finish_popup(info);
    } else {
        let parent_data = get_menu_data(data.parent);
        let info = parent_data.popup_info.as_ref().unwrap();
        finish_popup(info);
    }

    set_menu_data(window_handle, data);
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

#[cfg(feature = "accelerator")]
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

    if dc.0.is_null() {
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

    if dc.0.is_null() {
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
        right: data.size.width,
        bottom: data.size.height,
    };

    unsafe { data.dc_render_target.BindDC(dc, &client_rect).unwrap() };
    unsafe { data.dc_render_target.BeginDraw() };

    for item in items {
        /* Ignore invisible MenuItem */
        if !item.visible {
            continue;
        }

        let whole_item_rect = get_item_rect(item);

        let disabled = item.disabled;
        let checked = item.checked;
        let selected = item.index == data.selected_index && !disabled;

        fill_background(data, &whole_item_rect, scheme, selected)?;

        match item.menu_item_type {
            MenuItemType::Separator => {
                /* Use whole item rect to draw from left to right */
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

                if item.icon.is_some() {
                    draw_icon(data, item, &item_rect, scheme, disabled)?;
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

fn to_vertical_middle(top: i32, item_height: i32, target_height: i32) -> i32 {
    top + ((item_height as f32 - target_height as f32) / 2.0).ceil() as i32
}

fn draw_checkmark(data: &MenuData, item_rect: &RECT, scheme: &ColorScheme, disabled: bool) -> Result<(), Error> {
    let space = data.icon_space.left;
    let check_rect = RECT {
        left: item_rect.left + space.lmargin,
        top: item_rect.top,
        right: item_rect.left + space.lmargin + space.width + space.rmargin,
        bottom: item_rect.top + space.width,
    };

    let color = if disabled {
        scheme.disabled
    } else {
        scheme.color
    };

    let dc5 = get_device_context(&data.dc_render_target);

    let element = unsafe { data.check_svg.GetRoot() }?;
    set_fill_color(&element, color);
    set_stroke_color(&element, color);

    let top = to_vertical_middle(check_rect.top, item_rect.bottom - item_rect.top, check_rect.bottom - check_rect.top);
    let translation = Matrix3x2::translation(check_rect.left as f32, top as f32);
    unsafe { dc5.SetTransform(&translation) };
    unsafe { dc5.DrawSvgDocument(&data.check_svg) };
    unsafe { dc5.SetTransform(&Matrix3x2::identity()) };

    Ok(())
}

fn draw_icon(data: &MenuData, item: &MenuItem, item_rect: &RECT, scheme: &ColorScheme, disabled: bool) -> Result<(), Error> {
    let space = data.icon_space.mid;
    let check_margin = data.icon_space.left.lmargin + data.icon_space.left.width + data.icon_space.left.rmargin;
    let mut icon_rect = RECT {
        left: item_rect.left + check_margin + space.lmargin,
        top: item_rect.top,
        right: item_rect.left + check_margin + space.lmargin + space.width + space.rmargin,
        bottom: item_rect.top + space.width,
    };

    match data.icon_map.get(&item.uuid).unwrap() {
        MenuImageType::Bitmap(bitmap) => {
            icon_rect.right = icon_rect.left + data.icon_space.mid.width;
            icon_rect.top = to_vertical_middle(icon_rect.top, item_rect.bottom - item_rect.top, icon_rect.bottom - icon_rect.top);
            icon_rect.bottom = icon_rect.top + space.width;
            unsafe { data.dc_render_target.DrawBitmap(bitmap, Some(&to_2d_rect(&icon_rect)), 1.0, D2D1_BITMAP_INTERPOLATION_MODE_NEAREST_NEIGHBOR, None) };
        }
        MenuImageType::Svg(svg) => {
            let dc5 = get_device_context(&data.dc_render_target);
            let color = if disabled {
                scheme.disabled
            } else {
                scheme.color
            };
            let element = unsafe { svg.GetRoot() }?;
            set_fill_color(&element, color);
            let top = to_vertical_middle(icon_rect.top, item_rect.bottom - item_rect.top, icon_rect.bottom - icon_rect.top);
            let translation = Matrix3x2::translation(icon_rect.left as f32, top as f32);
            unsafe { dc5.SetTransform(&translation) };
            unsafe { dc5.DrawSvgDocument(svg) };
            unsafe { dc5.SetTransform(&Matrix3x2::identity()) };
        }
    }

    Ok(())
}

fn draw_menu_text(data: &MenuData, item: &MenuItem, item_rect: &RECT, scheme: &ColorScheme, disabled: bool) -> Result<(), Error> {
    /* Keep space for check, icon and submenu */
    let check_margin = data.icon_space.left.lmargin + data.icon_space.left.width + data.icon_space.left.rmargin;
    /*
        Use icon margin if
        - Menu has no check item but has any icon item
        - reserve_icon_size is true
        - This item has icon
    */
    let icon_margin = if (check_margin == 0 && !data.icon_map.is_empty()) || data.config.icon.as_ref().unwrap().reserve_icon_size || item.icon.is_some() {
        data.icon_space.mid.width + data.icon_space.mid.lmargin + data.icon_space.mid.rmargin
    } else {
        0
    };
    /* Use only right icon margin for items other than submenu */
    let arrow_margin = if item.menu_item_type == MenuItemType::Submenu {
        data.icon_space.right.lmargin + data.icon_space.right.width + data.icon_space.right.rmargin
    } else {
        data.icon_space.right.rmargin
    };
    let text_rect = RECT {
        left: item_rect.left + check_margin + icon_margin,
        top: item_rect.top,
        right: item_rect.right - arrow_margin,
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
        let color = if disabled {
            scheme.disabled
        } else {
            scheme.accelerator
        };
        let brush = unsafe { data.dc_render_target.CreateSolidColorBrush(&colorref_to_d2d1_color_f(color), None) }?;
        unsafe { format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_TRAILING) }?;
        unsafe { data.dc_render_target.DrawText(encode_wide(&item.accelerator).as_mut(), &format, &text_2d_rect, &brush, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL) };
    }

    Ok(())
}

fn draw_submenu_arrow(data: &MenuData, item_rect: &RECT, scheme: &ColorScheme, disabled: bool) -> Result<(), Error> {
    let margin = data.icon_space.right.lmargin;
    let icon_width = data.icon_space.right.width;
    let arrow_rect = RECT {
        left: item_rect.right - (margin + icon_width),
        top: item_rect.top,
        right: item_rect.right - margin,
        bottom: item_rect.top + icon_width,
    };

    let color = if disabled {
        scheme.disabled
    } else {
        scheme.color
    };

    let dc5 = get_device_context(&data.dc_render_target);

    let element = unsafe { data.submenu_svg.GetRoot() }?;
    set_fill_color(&element, color);
    set_stroke_color(&element, color);

    let top = to_vertical_middle(arrow_rect.top, item_rect.bottom - item_rect.top, arrow_rect.bottom - arrow_rect.top);
    let translation = Matrix3x2::translation(arrow_rect.left as f32, top as f32);
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

    /* Add 0.5 to disable antialiasing for line */
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
            data.config.size.separator_size as f32,
            None,
        )
    }

    Ok(())
}

fn get_display_point(window_handle: isize, x: i32, y: i32, cx: i32, cy: i32) -> DisplayPoint {
    let hwnd = hwnd!(window_handle);
    let mut rtl = false;
    let mut reverse = false;

    let mut ppt = POINT {
        x,
        y,
    };

    let mut hmon = unsafe { MonitorFromPoint(ppt, MONITOR_DEFAULTTONULL) };

    if hmon.0.is_null() {
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
    let hwnd = hwnd!(window_handle);
    /* Menu is yet to be visible due to timer or animation */
    if unsafe { !IsWindowVisible(hwnd) }.as_bool() {
        return false;
    }

    let selected_index = index_from_point(hwnd, screen_point, data);

    if data.visible_submenu_index >= 0 && selected_index < 0 {
        return false;
    }

    let selection_changed = data.selected_index != selected_index;

    if selection_changed {
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

    set_menu_data(window_handle, data);

    selection_changed
}

fn toggle_submenu(window_handle: isize, data: &mut MenuData) {
    if data.selected_index < 0 {
        return;
    }

    if data.visible_submenu_index >= 0 && data.visible_submenu_index != data.selected_index {
        let submenu_window_handle = data.items[data.visible_submenu_index as usize].submenu.as_ref().unwrap().window_handle;
        animate_hide_submenu(submenu_window_handle);
        data.visible_submenu_index = -1;
    }

    if data.visible_submenu_index < 0 && data.items[data.selected_index as usize].menu_item_type == MenuItemType::Submenu {
        if data.items[data.selected_index as usize].disabled {
            data.visible_submenu_index = -1;
        } else {
            data.visible_submenu_index = data.selected_index;
        }
    }

    set_menu_data(window_handle, data);

    if data.visible_submenu_index >= 0 {
        show_submenu(window_handle);
    }
}

fn show_submenu(window_handle: isize) {
    let proc: TIMERPROC = Some(delay_show_submenu);
    let mut show_delay: u32 = 0;
    let _ = unsafe { SystemParametersInfoW(SPI_GETMENUSHOWDELAY, 0, Some(&mut show_delay as *mut _ as *mut _), SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0)) };
    unsafe { SetTimer(hwnd!(window_handle), SHOW_SUBMENU_TIMER_ID, show_delay, proc) };
}

unsafe extern "system" fn delay_show_submenu(hwnd: HWND, _msg: u32, id: usize, _time: u32) {
    KillTimer(hwnd, id).unwrap();

    let main_menu_data = get_menu_data(vtoi!(hwnd.0));

    if main_menu_data.visible_submenu_index >= 0 {
        let submenu_item = &main_menu_data.items[main_menu_data.visible_submenu_index as usize];
        let submenu_window_handle = submenu_item.submenu.as_ref().unwrap().window_handle;
        let submenu_data = get_menu_data(submenu_window_handle);

        /* If submenu has no item, do not show submenu */
        if submenu_data.items.is_empty() {
            return;
        }

        let mut main_menu_rect = RECT::default();
        GetWindowRect(hwnd, &mut main_menu_rect).unwrap();

        let pt = get_display_point(submenu_window_handle, main_menu_rect.right, main_menu_rect.top + submenu_item.top, submenu_data.size.width, submenu_data.size.height);

        let x = if pt.rtl {
            main_menu_rect.left - submenu_data.size.width - main_menu_data.config.size.submenu_offset
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
                y: submenu_item.bottom - submenu_data.size.height,
            };
            let _ = ClientToScreen(hwnd, &mut reversed_point);
            /* Add top/bottom padding, round corner size, and border size */
            reversed_point.y + main_menu_data.config.size.vertical_padding * 2 + round_corner_size + main_menu_data.config.size.border_size
        } else {
            /* Reduce top padding and border size */
            main_menu_rect.top + submenu_item.top - main_menu_data.config.size.vertical_padding - round_corner_size - main_menu_data.config.size.border_size
        };

        SetWindowPos(hwnd!(submenu_window_handle), HWND_TOP, x, y, submenu_data.size.width, submenu_data.size.height, SWP_ASYNCWINDOWPOS | SWP_NOOWNERZORDER | SWP_NOACTIVATE).unwrap();
        animate_show_window(submenu_window_handle);
    }
}

fn hide_submenu(window_handle: isize) {
    let data = get_menu_data_mut(window_handle);
    data.selected_index = -1;
    set_menu_data(window_handle, data);
    let hwnd = hwnd!(window_handle);
    let _ = unsafe { ShowWindow(hwnd, SW_HIDE) };
}

fn animate_hide_submenu(window_handle: isize) {
    let hwnd = hwnd!(window_handle);
    let proc: TIMERPROC = Some(delay_hide_submenu);
    let mut hide_delay: u32 = 0;
    let _ = unsafe { SystemParametersInfoW(SPI_GETMENUSHOWDELAY, 0, Some(&mut hide_delay as *mut _ as *mut _), SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0)) };
    unsafe { SetTimer(hwnd, HIDE_SUBMENU_TIMER_ID, hide_delay - 100, proc) };
}

unsafe extern "system" fn delay_hide_submenu(hwnd: HWND, _msg: u32, id: usize, _time: u32) {
    KillTimer(hwnd, id).unwrap();
    let data = get_menu_data_mut(vtoi!(hwnd.0));
    data.selected_index = -1;
    set_menu_data(vtoi!(hwnd.0), data);
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
    if rect.top == 0 && rect.bottom == data.size.height {
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

    if pt.x >= 0 && pt.x < data.size.width && pt.y >= 0 && pt.y < data.size.height {
        for item in &data.items {
            /* Ignore invisible */
            if item.menu_item_type == MenuItemType::Separator || !item.visible {
                continue;
            }

            if pt.y >= item.top && pt.y <= item.bottom {
                selected_index = item.index;
                break;
            }
        }
    }

    selected_index
}

fn get_hwnd_from_point(hwnd: HWND) -> Option<HWND> {
    let data = get_menu_data(vtoi!(hwnd.0));
    let submenu = if data.visible_submenu_index >= 0 {
        hwnd!(data.items[data.visible_submenu_index as usize].submenu.as_ref().unwrap().window_handle)
    } else {
        HWND(std::ptr::null_mut())
    };

    let pt = get_cursor_point(hwnd);

    let window = unsafe { WindowFromPoint(pt) };

    if !submenu.0.is_null() && window == submenu {
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

    /* Don't respont to setting change event unless theme is System */
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
    let hwnd = hwnd!(window_handle);
    let _ = unsafe { UpdateWindow(hwnd) };

    for menu_item in &data.items {
        let item = menu_item;
        if item.menu_item_type == MenuItemType::Submenu {
            let submenu_window_handle = item.submenu.as_ref().unwrap().window_handle;
            let submenu_hwnd = hwnd!(submenu_window_handle);
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

    let hwnd = unsafe { CreateWindowExW(ex_style, class_name, PCWSTR::null(), window_styles, 0, 0, 0, 0, hwnd!(parent), None, GetModuleHandleW(PCWSTR::null()).unwrap_or_default(), None).unwrap() };

    let _ = unsafe { SetWindowPos(hwnd, None, 0, 0, 0, 0, SWP_NOZORDER | SWP_NOOWNERZORDER | SWP_NOMOVE | SWP_NOSIZE | SWP_FRAMECHANGED) };

    Ok(vtoi!(hwnd.0))
}
