pub mod util;
pub mod config;
pub mod menu_item;
use util::*;
use config::*;
use menu_item::*;

use std::{ffi::c_void, mem::{size_of, transmute}, sync::atomic::{AtomicU32, Ordering}};
use once_cell::sync::Lazy;
use windows::{
    core::{w, PCSTR, PCWSTR, Error},
    Win32::{
        Foundation::{COLORREF, HINSTANCE, HMODULE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM},
        Graphics::Gdi::{
            BeginPaint, ClientToScreen, CreateFontIndirectW, CreatePen, CreateSolidBrush, DeleteObject, DrawTextW, EndPaint,
            ExcludeClipRect, FillRect, GetDC, GetMonitorInfoW, GetObjectW, GetWindowDC, InflateRect, InvalidateRect,
            LineTo, MonitorFromPoint, MonitorFromWindow, MoveToEx, OffsetRect, PtInRect, ReleaseDC, ScreenToClient, SelectObject,
            SetBkMode, SetTextColor, UpdateWindow,
            DT_CALCRECT, DT_LEFT, DT_RIGHT, DT_SINGLELINE, DT_VCENTER, MONITOR_DEFAULTTONEAREST, MONITOR_DEFAULTTONULL,PS_SOLID, TRANSPARENT,
            HBRUSH, HDC, HFONT, HGDIOBJ, HPEN, LOGFONTW, MONITORINFO, PAINTSTRUCT
        },
        System::LibraryLoader::{GetModuleHandleW, GetProcAddress, LoadLibraryW},
        UI::{
            Controls::{CloseThemeData, DrawThemeBackgroundEx, OpenThemeDataEx, HTHEME, MC_CHECKMARKNORMAL, MENU_POPUPCHECK, MENU_POPUPSUBMENU, MSM_NORMAL, OTD_NONCLIENT},
            Input::KeyboardAndMouse::{
                GetActiveWindow, ReleaseCapture, SendInput, SetCapture,
                INPUT, INPUT_0, MOUSEINPUT,
                INPUT_MOUSE, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_VIRTUALDESK
            },
            Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DispatchMessageW, GetAncestor, GetClientRect, GetCursorPos, GetMessageW, GetParent,
                GetSystemMetrics, GetWindowRect, IsWindowVisible, KillTimer, PostMessageW, RegisterClassExW, SetTimer,
                SetWindowLongPtrW, SetWindowPos, ShowWindow, SystemParametersInfoW, TranslateMessage, WindowFromPoint,
                CS_DROPSHADOW, CS_HREDRAW, CS_VREDRAW, GA_ROOTOWNER, GWL_USERDATA, HWND_TOP, SM_CXHSCROLL, SM_CXMENUCHECK, SM_CYMENU,
                SPI_GETNONCLIENTMETRICS, SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE, SWP_NOOWNERZORDER, SW_HIDE, SW_SHOWNOACTIVATE, SPI_GETMENUSHOWDELAY,
                WM_APP, WM_DESTROY, WM_ERASEBKGND, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_PAINT, WM_RBUTTONDOWN,
                WM_RBUTTONUP, WM_THEMECHANGED, WS_CLIPSIBLINGS, WS_EX_TOOLWINDOW, WS_POPUP,
                HCURSOR, HICON, MSG, NONCLIENTMETRICSW, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, TIMERPROC, WNDCLASSEXW,
            }
        }
    }
};

static HUXTHEME: Lazy<HMODULE> = Lazy::new(|| unsafe { LoadLibraryW(w!("uxtheme.dll")).unwrap_or_default() });
static COUNTER:AtomicU32 = AtomicU32::new(400);

const MAIN_MENU_INDEX:i32 = 0;
const LR_BUTTON_SIZE:i32 = 25;
const SUBMENU_OFFSET:i32 = -5;
const TIMER_ID:usize = 500;

const WM_MENUSELECTED:u32 = WM_APP + 0x0001;
const WM_CLOSEMENU:u32 = WM_APP + 0x0002;
const WM_INACTIVATE:u32 = WM_APP + 0x0003;

#[derive(Debug, Clone)]
struct DisplayPoint{
    x:i32,
    y:i32,
    rtl:bool,
    reverse:bool,
}


#[derive(Debug, Clone)]
pub struct RMenu {
    pub hwnd:HWND,
    parent:HWND,
    items:Vec<InnerMenuItem>,
    width:i32,
    height:i32,
    index:i32,
    theme:Theme,
    size:MenuSize,
    color:ThemeColor
}

impl Default for RMenu {
    fn default() -> Self {
        Self {
            hwnd: HWND(0),
            parent:HWND(0),
            items:Vec::new(),
            height:0,
            width:0,
            index:MAIN_MENU_INDEX,
            theme:Theme::Light,
            size:MenuSize::default(),
            color:ThemeColor::default(),
        }
    }
}

#[derive(Debug, Clone)]
struct MenuData {
    index:i32,
    items:Vec<InnerMenuItem>,
    htheme:Option<HTHEME>,
    win_subclass_id:Option<u32>,
    selected_index:i32,
    width:i32,
    height:i32,
    visible_submenu_index:i32,
    theme:Theme,
    size:MenuSize,
    color:ThemeColor,
}

impl RMenu {

    pub fn new(parent:HWND) -> Self {
        let mut menu = RMenu::default();
        menu.parent = parent;
        menu.hwnd = create_container_hwnd(parent, menu.theme.clone()).unwrap();
        menu
    }

    pub fn new_with_theme(parent:HWND, theme:Theme) -> Self {
        let mut menu = RMenu::default();
        menu.parent = parent;
        menu.theme = theme;
        menu.hwnd = create_container_hwnd(parent, menu.theme.clone()).unwrap();
        menu
    }

    pub fn new_from_config(parent:HWND, config:Config) -> Self {
        let mut menu = RMenu::default();
        menu.parent = parent;
        menu.theme = config.theme;
        menu.size = config.size;
        menu.color = config.color;
        menu.hwnd = create_container_hwnd(parent, menu.theme.clone()).unwrap();
        menu
    }

    pub fn items(&self) -> Vec<MenuItem> {
        let data = get_menu_data(self.hwnd);
        data.items.iter().map(|item| MenuItem::new(self.hwnd, item)).collect()
    }

    pub fn theme(&self) -> Theme {
        let data = get_menu_data(self.hwnd);
        data.theme.clone()
    }

    pub fn set_theme(self, theme:Theme){
        on_theme_change(self.hwnd, Some(theme));
    }

    fn create_state(disabled:Option<bool>, checked:Option<bool>) -> MenuItemState {
        let mut state = MENU_NORMAL.0;
        if disabled.is_some() && disabled.unwrap() {
            state |= MENU_DISABLED.0;
        }

        if checked.is_some() && checked.unwrap() {
            state |= MENU_CHECKED.0;
        }

        MenuItemState(state)
    }

    pub fn text(&mut self, id:&str, label:&str, disabled:Option<bool>) -> &Self {
        self.items.push(InnerMenuItem::new(id, label, "", None, None, Self::create_state(disabled, None), RMT_TEXT));
        self
    }

    pub fn text_with_accelerator(&mut self, id:&str, label:&str, disabled:Option<bool>, accelerator:&str) -> &Self {
        self.items.push(InnerMenuItem::new(id, label, "", Some(accelerator), None, Self::create_state(disabled, None), RMT_TEXT));
        self
    }

    pub fn check(&mut self, id:&str, label:&str, value:&str, checked:bool, disabled:Option<bool>) -> &Self {
        self.items.push(InnerMenuItem::new(id, label, value, None, None, Self::create_state(disabled, Some(checked)), RMT_CHECKBOX));
        self
    }

    pub fn check_with_accelerator(&mut self, id:&str, label:&str, value:&str, checked:bool, disabled:Option<bool>, accelerator:&str) -> &Self {
        self.items.push(InnerMenuItem::new(id, label, value, Some(accelerator), None, Self::create_state(disabled, Some(checked)), RMT_CHECKBOX));
        self
    }

    pub fn radio(&mut self, id:&str, label:&str, value:&str, name:&str, checked:bool, disabled:Option<bool>) -> &Self {
        self.items.push(InnerMenuItem::new(id, label, value, None, Some(name), Self::create_state(disabled, Some(checked)), RMT_RADIO));
        self
    }

    pub fn radio_with_accelerator(&mut self, id:&str, label:&str, value:&str, name:&str, checked:bool, disabled:Option<bool>, accelerator:&str) -> &Self {
        self.items.push(InnerMenuItem::new(id, label, value, Some(accelerator), Some(name), Self::create_state(disabled, Some(checked)), RMT_RADIO));
        self
    }

    pub fn submenu(&mut self, label:&str) -> Self {
        let mut item = InnerMenuItem::new(label, label, "", None, None, MENU_NORMAL, RMT_SUBMENU);
        let mut submenu = RMenu::new_with_theme(self.hwnd, self.theme.clone());
        submenu.index = item.index;
        submenu.size = self.size.clone();
        submenu.color = self.color.clone();
        item.submenu = Some(submenu.hwnd);
        self.items.push(item);
        submenu
    }

    pub fn separator(&mut self) -> &Self {
        self.items.push(InnerMenuItem::new("","","",None, None, MENU_NORMAL, RMT_SEPARATOR));
        self
    }

    pub fn build(&mut self) -> Result<(), Error> {

        // Add top and left margin
        let mut width = self.size.horizontal_margin;
        let mut height = self.size.vertical_margin;

        for i in 0..self.items.len() {

            let item = &mut self.items[i];
            item.index = i as i32;

            item.top = height;
            let (item_width, item_height) = measure_item(self.hwnd, &self.size, &item, self.theme.clone())?;
            item.bottom = item.top + item_height;

            width = std::cmp::max(width, item_width);
            height += item_height;

        }

        // Add bottom and right margin
        width += self.size.horizontal_margin;
        height += self.size.vertical_margin;

        width += self.size.border_width * 2;
        height += self.size.border_width * 2;
        self.width = width;
        self.height = height;

        let is_main_menu = self.index == MAIN_MENU_INDEX;
        let data = MenuData {
            index:self.index,
            items:self.items.clone(),
            htheme: if is_main_menu { Some(unsafe { OpenThemeDataEx(self.hwnd, w!("Menu"), OTD_NONCLIENT) }) } else { None },
            win_subclass_id: if is_main_menu { Some(COUNTER.fetch_add(1, Ordering::Relaxed)) } else { None },
            height,
            width,
            selected_index:-1,
            visible_submenu_index:-1,
            theme:self.theme.clone(),
            size:self.size.clone(),
            color:self.color.clone(),
        };

        if is_main_menu {
            Self::attach_owner_subclass(self, data.win_subclass_id.unwrap() as usize);
        }

        unsafe { SetWindowLongPtrW(self.hwnd, GWL_USERDATA, Box::into_raw(Box::new(data)) as _) };

        Ok(())

    }

    fn attach_owner_subclass(&self, id:usize) {
        unsafe {
            let ancestor = GetAncestor(self.parent, GA_ROOTOWNER);
            SetWindowSubclass(
                if ancestor.0 == 0 { self.parent } else { ancestor },
                Some(menu_owner_subclass_proc),
                id,
                Box::into_raw(Box::new(self.hwnd)) as _
            );
        }
    }

    pub fn popup_at(&self, x:i32, y:i32) -> Option<&SelectedMenuItem> {

        let pt = get_display_point(self.hwnd, x, y, self.width, self.height);
        unsafe {
            let _ = SetWindowPos(self.hwnd, HWND_TOP, pt.x, pt.y, self.width, self.height, SWP_ASYNCWINDOWPOS | SWP_NOOWNERZORDER | SWP_NOACTIVATE);
            ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
            // Prevent mouse input on window beneath menu
            SetCapture(self.hwnd);
        };

        let mut msg = MSG::default();
        let mut selected_item:Option<&SelectedMenuItem> = None;

        while unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {

            if self.parent != unsafe { GetActiveWindow() } {
                // Send WM_INACTIVATE message so that MenuData is initialized
                let _ = unsafe { PostMessageW(self.hwnd, WM_INACTIVATE, WPARAM(0), LPARAM(0)) };
            }

            match msg.message {

                WM_MENUSELECTED => {
                    selected_item = Some(unsafe { transmute::<isize, &SelectedMenuItem>(msg.lParam.0) });
                    break;
                }

                WM_CLOSEMENU => {
                    break;
                }

                _ => {
                    unsafe { TranslateMessage(&msg) };
                    unsafe { DispatchMessageW(&msg) };
                }
            }

        }

        let _ = unsafe { ReleaseCapture() };

        unsafe { ShowWindow(self.hwnd, SW_HIDE) };

        selected_item

    }

}

unsafe extern "system" fn default_window_proc(
    window: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {

    match msg {

        WM_INACTIVATE => {
            if IsWindowVisible(window).as_bool() {
                init_menu_data(window);
                PostMessageW(window, WM_CLOSEMENU, WPARAM(0), LPARAM(0)).unwrap();
            }
            LRESULT(0)
        }

        WM_DESTROY => {
            let data = get_menu_data_mut(window);
            if data.index == MAIN_MENU_INDEX {
                RemoveWindowSubclass(window, Some(menu_owner_subclass_proc), data.win_subclass_id.unwrap() as usize);
                CloseThemeData(data.htheme.unwrap()).unwrap();
            }
            DefWindowProcW(window, msg, wparam, lparam)
        }

        WM_ERASEBKGND => {
            let data = get_menu_data(window);
            paint_background(window, data);
            LRESULT(1)
        }

        WM_PAINT => {
            let data = get_menu_data(window);
            let theme = get_theme(window, data);
            on_paint(window, data, theme).unwrap();
            LRESULT(0)
        }

        WM_MOUSEMOVE => {
            let mut pt = POINT::default();
            let _ = unsafe { GetCursorPos(&mut pt) };
            let data = get_menu_data_mut(window);
            let should_show_submenu = on_mouse_move(data, window, pt);
            set_menu_data(window, data);

            if should_show_submenu {
                show_submenu(window);
            }

            if data.visible_submenu_index >= 0 {
                let hwnd = data.items[data.visible_submenu_index as usize].submenu.unwrap();
                let data = get_menu_data_mut(hwnd);
                on_mouse_move(data, hwnd, pt);
                set_menu_data(hwnd, data);
            }

            LRESULT(0)
        }

        WM_LBUTTONUP | WM_RBUTTONUP => {
            let hwnd_opt = get_hwnd_from_point(window, lparam);
            if hwnd_opt.is_none() {
                return LRESULT(0);
            }

            let hwnd = hwnd_opt.unwrap();
            let data = get_menu_data_mut(hwnd);
            let index = index_from_point(hwnd, to_screen_point(window, lparam), data);

            // toggle checkbox
            if data.items[index as usize].menu_type == RMT_CHECKBOX {
                let checked = (data.items[index as usize].state.0 & MENU_CHECKED.0) != 0;
                toggle_checked(&mut data.items[index as usize], !checked);
            }

            // toggle radio checkbox
            if data.items[index as usize].menu_type == RMT_RADIO {
                toggle_radio(data, index as usize);
            }

            set_menu_data(hwnd, data);
            init_menu_data(window);

            let menu_item = SelectedMenuItem::from(&data.items[index as usize]);
            PostMessageW(hwnd, WM_MENUSELECTED, WPARAM(0), LPARAM(Box::into_raw(Box::new(menu_item)) as _)).unwrap();

            LRESULT(0)
        }

        WM_LBUTTONDOWN | WM_RBUTTONDOWN => {
            // If mouse input occurs outside of menu, close menu
            if get_hwnd_from_point(window, lparam).is_none() {
                init_menu_data(window);
                PostMessageW(window, WM_CLOSEMENU, WPARAM(0), LPARAM(0)).unwrap();
                // If mouse input occurs on parent window, send mouse input
                send_mouse_input(window, msg);
                return LRESULT(0);
            }
            DefWindowProcW(window, msg, wparam, lparam)
        }

        _ => {
            DefWindowProcW(window, msg, wparam, lparam)
        }
    }

}

fn send_mouse_input(hwnd:HWND, msg: u32){

    let mut count = 0;
    let mut parent = unsafe { GetParent(hwnd) };
    let mut cursor_point = POINT::default();
    let _ = unsafe { GetCursorPos(&mut cursor_point) };
    while parent.0 != 0 {
        let mut rect = RECT::default();
        let _ = unsafe { GetWindowRect(parent, &mut rect) };
        if unsafe { PtInRect(&mut rect as *const _ as _, cursor_point) }.as_bool() {
            count += 1;
        }
        parent = unsafe { GetParent(parent) };
    }

    if count > 0 {
        let mut flags = MOUSEEVENTF_VIRTUALDESK | MOUSEEVENTF_ABSOLUTE;
        flags |= if msg == WM_LBUTTONDOWN { MOUSEEVENTF_LEFTDOWN } else { MOUSEEVENTF_RIGHTDOWN };

        let input = INPUT{
            r#type:INPUT_MOUSE,
            Anonymous:INPUT_0 {
                mi: MOUSEINPUT{
                    dx:cursor_point.x,
                    dy:cursor_point.y,
                    mouseData:0,
                    dwFlags: flags,
                    time:0,
                    dwExtraInfo:0
                    }
            }
        };
        unsafe { SendInput(&[input], size_of::<INPUT>() as i32) };
    }

}

unsafe extern "system" fn menu_owner_subclass_proc(
    window: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _uidsubclass: usize,
    _dwrefdata: usize,
) -> LRESULT {

    match msg {
        WM_THEMECHANGED => {
            let hwnd = transmute::<usize, &HWND>(_dwrefdata);
            on_theme_change(*hwnd, None);
            DefSubclassProc(window, msg, wparam, lparam)
        }

        _ => {
            DefSubclassProc(window, msg, wparam, lparam)
        }
    }
}

fn measure_item(hwnd:HWND, size:&MenuSize, item_data:&InnerMenuItem, theme:Theme) -> Result<(i32,i32), Error> {

    let mut width = 0;
    let height;

    match item_data.menu_type {

        RMT_SEPARATOR => {
            // separator - use half system height and zero width
            height = unsafe { (GetSystemMetrics(SM_CYMENU) as i32 + 4) / 2 };
        },

        _ => {

            let dc:HDC = unsafe { GetDC(hwnd) };
            let menu_font = get_font(theme, size)?;
            let font:HFONT = unsafe { CreateFontIndirectW(&menu_font) };
            let old_font:HGDIOBJ = unsafe { SelectObject(dc, font) };
            let mut text_rect = RECT::default();

            let mut raw_text = item_data.label.clone();
            if item_data.accelerator.is_some() {
                raw_text.extend(item_data.accelerator.as_ref().unwrap());
            }

            unsafe { DrawTextW(dc, raw_text.as_mut_slice(), &mut text_rect, DT_SINGLELINE | DT_LEFT | DT_VCENTER | DT_CALCRECT) };
            unsafe { SelectObject(dc, old_font) };

            let mut cx = text_rect.right - text_rect.left;

            let mut log_font = LOGFONTW::default();
            unsafe { GetObjectW(font, size_of::<LOGFONTW>() as i32, Some(&mut log_font as *mut _ as *mut c_void)) };

            let mut cy = log_font.lfHeight;
            if cy < 0 {
                cy = -cy;
            }
            cy += size.item_vertical_padding;

            height = cy;

            cx += size.item_horizontal_padding;
            cx += LR_BUTTON_SIZE * 2;
            // extra padding
            if item_data.accelerator.is_some() {
                cx += 30;
            }

            // Windows adds 1 to returned value
            cx -= unsafe { GetSystemMetrics(SM_CXMENUCHECK) - 1 };

            width = cx;

            unsafe { ReleaseDC(hwnd, dc) };
        }
    }

    Ok((width,height))
}

fn get_font(theme:Theme, size:&MenuSize) -> Result<LOGFONTW, Error> {

    let mut info:NONCLIENTMETRICSW = NONCLIENTMETRICSW::default();
    info.cbSize = size_of::<NONCLIENTMETRICSW>() as u32;
    unsafe { SystemParametersInfoW(SPI_GETNONCLIENTMETRICS, size_of::<NONCLIENTMETRICSW>() as u32, Some(&mut info as *mut _ as *mut c_void), SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0))? };

    let mut menu_font = info.lfMenuFont;

    if size.font_size.is_some() {
        menu_font.lfHeight = -size.font_size.unwrap();
    }

    if size.font_weight.is_some() {
        menu_font.lfWeight = size.font_weight.unwrap();
    } else {
        if theme == Theme::Dark {
            // bold font
            menu_font.lfWeight = 700;
        }
    }

    Ok(menu_font)
}

fn get_color_scheme(data:&MenuData) -> &ColorScheme {
    if data.theme == Theme::Dark { &data.color.dark } else { &data.color.light }
}

fn paint_background(hwnd:HWND, data:&MenuData) {
    unsafe {

        let dc = GetWindowDC(hwnd);

        if dc.0 == 0 {
            return;
        }

        let scheme = get_color_scheme(data);

        let mut client_rect = RECT::default();
        GetClientRect(hwnd, &mut client_rect).unwrap();

        let hbr = CreateSolidBrush(COLORREF(scheme.border));
        FillRect(dc, &mut client_rect, hbr);
        DeleteObject(hbr);

        let mut menu_rect = RECT {
            left: client_rect.left + data.size.border_width,
            top: client_rect.top +  data.size.border_width,
            right: client_rect.right - data.size.border_width,
            bottom: client_rect.bottom -  data.size.border_width,
        } ;

        let hbr = CreateSolidBrush(COLORREF(scheme.background_color));
        FillRect(dc, &mut menu_rect, hbr);
        DeleteObject(hbr);

        ReleaseDC(hwnd, dc);
    }
}

fn on_paint(hwnd:HWND, data:&MenuData, theme:HTHEME) -> Result<(), Error> {

    let mut paint_struct = PAINTSTRUCT::default();
    let dc = unsafe { BeginPaint(hwnd, &mut paint_struct) };

    if dc.0 == 0 {
        return Ok(());
    }

    let index = index_from_rect(data, paint_struct.rcPaint);

    if index.is_none() {
        paint(dc, data, &data.items, theme)?;
    } else {
        paint(dc, data, &vec![data.items[index.unwrap() as usize].clone()], theme)?;
    }

    unsafe { EndPaint(hwnd, &mut paint_struct) };

    Ok(())
}

fn paint(dc:HDC, data:&MenuData, items:&Vec<InnerMenuItem>, theme:HTHEME) -> Result<(), Error> {

    let scheme = get_color_scheme(data);
    let selected_color = unsafe { CreateSolidBrush(COLORREF(scheme.hover_background_color)) };
    let normal_color = unsafe { CreateSolidBrush(COLORREF(scheme.background_color)) };

    for item in items {

        let mut item_rect = get_item_rect(data, item);

        let disabled = (item.state.0 & MENU_DISABLED.0) != 0;
        let checked = (item.state.0 & MENU_CHECKED.0) != 0;

        if item.index == data.selected_index {
            unsafe { FillRect(dc, &mut item_rect, selected_color) };
        }else{
            unsafe { FillRect(dc, &mut item_rect, normal_color) };
        }

        match item.menu_type {

            RMT_SEPARATOR => {
                draw_separator(dc, scheme, item_rect)?;
            }

            _ => {

                if checked {
                    let mut rect = RECT{ left: item_rect.left, top:item_rect.top, right:item_rect.left + LR_BUTTON_SIZE, bottom:item_rect.top + LR_BUTTON_SIZE };
                    // center vertically
                    unsafe { OffsetRect(&mut rect, 0, ((item_rect.bottom - item_rect.top) - (rect.bottom - rect.top)) / 2) };
                    let mut check_rect = rect.clone();
                    unsafe { InflateRect(&mut check_rect as *mut _ as *mut RECT, -1, -1) };
                    unsafe { DrawThemeBackgroundEx(theme, dc, MENU_POPUPCHECK.0, MC_CHECKMARKNORMAL.0, &mut check_rect, None)? };
                }

                let mut text_rect = item_rect.clone();
                // Keep space for check mark and submenu mark
                text_rect.left += LR_BUTTON_SIZE;
                text_rect.right -= LR_BUTTON_SIZE;

                if item.menu_type == RMT_SUBMENU {
                    let mut arrow_rect  = item_rect.clone();
                    let arrow_size = unsafe { GetSystemMetrics(SM_CXHSCROLL) };
                    text_rect.right -= arrow_size;
                    arrow_rect.left = item_rect.right - arrow_size;

                    // center vertically
                    unsafe { OffsetRect(&mut arrow_rect as *mut _ as *mut RECT, 0, ((item_rect.bottom - item_rect.top) - (arrow_rect.bottom - arrow_rect.top)) / 2) };
                    unsafe { DrawThemeBackgroundEx(theme, dc, MENU_POPUPSUBMENU.0, MSM_NORMAL.0, &mut arrow_rect, None)? };

                }

                draw_menu_text(dc, scheme, &text_rect, item, data, disabled)?;
                unsafe { ExcludeClipRect(dc, item_rect.left, item_rect.top, item_rect.right, item_rect.bottom) };
            }
        }
    }

    unsafe { DeleteObject(selected_color) };
    unsafe { DeleteObject(normal_color); }

    Ok(())
}

fn draw_separator(dc:HDC, scheme:&ColorScheme, rect:RECT) -> Result<(), Error> {

    let mut separator_rect = rect.clone();

    separator_rect.top += (rect.bottom - rect.top) / 2;

    let pen:HPEN = unsafe { CreatePen(PS_SOLID, 1, COLORREF(scheme.border)) };
    let old_pen:HGDIOBJ = unsafe { SelectObject(dc,pen) };
    unsafe { MoveToEx(dc, separator_rect.left, separator_rect.top, None) };
    unsafe { LineTo(dc, separator_rect.right, separator_rect.top) };
    unsafe { SelectObject(dc,old_pen) };

    Ok(())
}

fn draw_menu_text(dc:HDC, scheme:&ColorScheme, rect:&RECT, item:&InnerMenuItem, data:&MenuData, disabled:bool) -> Result<(), Error> {

    let mut text_rect = rect.clone();

    unsafe { SetBkMode(dc, TRANSPARENT) };
    if disabled {
        unsafe { SetTextColor(dc, COLORREF(scheme.disabled)) };
    } else {
        unsafe { SetTextColor(dc, COLORREF(scheme.color)) };
    }

    let menu_font = get_font(data.theme.clone(), &data.size)?;
    let font:HFONT = unsafe { CreateFontIndirectW(&menu_font) };
    let old_font:HGDIOBJ = unsafe { SelectObject(dc,font) };

    unsafe { DrawTextW(dc, &mut item.label.clone(), &mut text_rect, DT_SINGLELINE | DT_LEFT | DT_VCENTER) };

    if item.accelerator.is_some() {
        unsafe { SetTextColor(dc, COLORREF(scheme.disabled)) };
        unsafe { DrawTextW(dc, &mut item.accelerator.as_ref().unwrap().clone(), &mut text_rect, DT_SINGLELINE | DT_RIGHT | DT_VCENTER) };
    }

    unsafe { SelectObject(dc,old_font) };

    Ok(())
}

fn show_submenu(hwnd:HWND){
    let proc: TIMERPROC = Some(delay_show_submenu);
    let mut show_delay:u32 = 0;
    let _ = unsafe { SystemParametersInfoW(SPI_GETMENUSHOWDELAY, 0, Some(&mut show_delay as *mut _ as *mut c_void), SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0)) };
    unsafe { SetTimer(hwnd, TIMER_ID, show_delay, proc) };
}

unsafe extern "system" fn delay_show_submenu(hwnd: HWND, _msg: u32, id: usize, _time: u32){

    KillTimer(hwnd, id).unwrap();

    let main_menu_data = get_menu_data(hwnd);

    if main_menu_data.visible_submenu_index >= 0 {
        let mut main_menu_rect = RECT::default();
        GetWindowRect(hwnd, &mut main_menu_rect).unwrap();
        let item = &main_menu_data.items[main_menu_data.visible_submenu_index as usize];
        let submenu_hwnd = item.submenu.unwrap();
        let submenu_data = get_menu_data(submenu_hwnd);

        let pt = get_display_point(submenu_hwnd, main_menu_rect.right, main_menu_rect.top + item.top, submenu_data.width, submenu_data.height);

        let x = if pt.rtl { main_menu_rect.left - submenu_data.width - SUBMENU_OFFSET} else { main_menu_rect.right + SUBMENU_OFFSET };
        let y = if pt.reverse {
            let mut reversed_point = POINT {x:0, y:item.bottom - submenu_data.height};
            ClientToScreen(hwnd, &mut reversed_point);
            // Add top + bottom margin
            reversed_point.y + main_menu_data.size.vertical_margin * 2
        } else {
            // Reduce top margin
            main_menu_rect.top + item.top - main_menu_data.size.vertical_margin
        };

        SetWindowPos(submenu_hwnd, HWND_TOP, x, y, submenu_data.width, submenu_data.height, SWP_NOACTIVATE | SWP_NOOWNERZORDER).unwrap();
        ShowWindow(submenu_hwnd, SW_SHOWNOACTIVATE);
    }
}

fn hide_submenu(hwnd:HWND){
    let data = get_menu_data_mut(hwnd);
    data.selected_index = -1;
    set_menu_data(hwnd, data);
    unsafe {ShowWindow(hwnd, SW_HIDE) };
}

fn toggle_submenu(data:&mut MenuData, selected_index:i32) -> bool {

    let mut should_show_submenu = false;

    if selected_index < 0 {
        return should_show_submenu;
    }

    if data.visible_submenu_index >= 0 && data.visible_submenu_index != selected_index {
        let hwnd = data.items[data.visible_submenu_index as usize].submenu.unwrap();
        hide_submenu(hwnd);
        data.visible_submenu_index = -1;
    }

    if data.visible_submenu_index < 0 && data.items[selected_index as usize].menu_type == RMT_SUBMENU {
        data.visible_submenu_index = selected_index;
        should_show_submenu = true;
    }

    should_show_submenu

}

fn get_display_point(hwnd:HWND, x:i32, y:i32, cx:i32, cy:i32) -> DisplayPoint {

    let mut rtl = false;
    let mut reverse = false;

    /*
    *  First get the dimensions of the monitor that contains (x, y).
    */
    let mut ppt = POINT::default();
    ppt.x = x;
    ppt.y = y;

    let mut hmon = unsafe { MonitorFromPoint(ppt, MONITOR_DEFAULTTONULL) };

    /*
    *  If (x, y) is not on any monitor, then use the monitor that
    *  the owner window is on.
    */
    if hmon.0 == 0 {
        hmon = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
    }

    let mut minf = MONITORINFO::default();
    minf.cbSize = size_of::<MONITORINFO>() as u32;
    unsafe { GetMonitorInfoW(hmon, &mut minf) };

    /*
    *  If too high, then slide down.
    */
    if ppt.y < minf.rcWork.top {
        ppt.y = minf.rcMonitor.top;
    }

    /*
    *  If too far left, then slide right.
    */
    if ppt.x < minf.rcWork.left {
        ppt.x = minf.rcMonitor.left;
    }

    /*
    *  If too low, then slide up.
    */
    if ppt.y + cy >= minf.rcWork.bottom {
        ppt.y -= cy;
        reverse = true;
    }

    /*
    *  If too far right, then flip left.
    */
    //if ppt.x > minf.rcWork.right - cx {
    if ppt.x + cx >= minf.rcWork.right{
        ppt.x -= cx;
        rtl = true;
    }

    DisplayPoint {
        x:ppt.x,
        y:ppt.y,
        rtl,
        reverse,
    }
}

fn on_mouse_move(data:&mut MenuData, hwnd:HWND, screen_point:POINT) -> bool {

    let selected_index = index_from_point(hwnd, screen_point, data);

    if data.visible_submenu_index >= 0 && selected_index < 0 {
        return false;
    }

    let mut should_show_submenu = false;

    if data.selected_index != selected_index {

        should_show_submenu = toggle_submenu(data, selected_index);

        if selected_index >= 0 {
            let item = &data.items[selected_index as usize];
            let mut rect = get_item_rect(data, item);
            unsafe { InvalidateRect(hwnd, Some(&mut rect), false) };
        }

        if data.selected_index >= 0 {
            let item = &data.items[data.selected_index as usize];
            let mut rect = get_item_rect(data, item);
            unsafe { InvalidateRect(hwnd, Some(&mut rect), false) };
        }

    };

    data.selected_index = selected_index;

    should_show_submenu
}

fn get_item_rect(data:&MenuData, item:&InnerMenuItem) -> RECT {
    let border_width = data.size.border_width;
    RECT{ left:border_width, top:item.top + border_width, right: data.width - border_width, bottom: item.bottom + border_width}
}

fn to_screen_point(hwnd:HWND, lparam:LPARAM) -> POINT {
    let mut pt = POINT::default();
    pt.x = LOWORD(lparam.0 as u32) as i32;
    pt.y = HIWORD(lparam.0 as u32) as i32;
    unsafe { ClientToScreen(hwnd, &mut pt) };
    pt
}

fn index_from_rect(data:&MenuData, rect:RECT) -> Option<i32> {

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

fn index_from_point(hwnd:HWND, screen_pt:POINT, data:&MenuData) -> i32 {
    let mut selected_index:i32 = -1;
    let mut pt = screen_pt.clone();
    unsafe { ScreenToClient(hwnd, &mut pt)};

    if pt.x >= 0 && pt.x < data.width && pt.y >= 0 && pt.y < data.height {
        for item in &data.items {
            if pt.y >= item.top && pt.y <= item.bottom {

                if item.menu_type != RMT_SEPARATOR {
                    selected_index = item.index as i32;
                    break;
                }

            }
        }
    }
    selected_index
}

fn get_hwnd_from_point(hwnd:HWND, lparam:LPARAM) -> Option<HWND> {
    let data = get_menu_data(hwnd);
    let submenu = if data.visible_submenu_index >= 0 { data.items[data.visible_submenu_index as usize].submenu.unwrap() } else { HWND(0) };

    let pt = to_screen_point(hwnd, lparam);

    let window = unsafe { WindowFromPoint(pt) };

    if submenu.0 != 0 && window == submenu {
        return Some(submenu);
    }

    if hwnd == window {
        return Some(hwnd);
    }

    None
}

fn init_menu_data(hwnd:HWND){
    let data = get_menu_data_mut(hwnd);
    data.selected_index = -1;

    if data.visible_submenu_index >= 0 {
        let submenu_hwnd = data.items[data.visible_submenu_index as usize].submenu.unwrap();
        hide_submenu(submenu_hwnd);
    }
    data.visible_submenu_index = -1;

    set_menu_data(hwnd, data);
}

fn get_theme(hwnd:HWND, data:&MenuData) -> HTHEME {
    if data.htheme.is_some() {
        return data.htheme.unwrap();
    }

    let parent = unsafe { GetParent(hwnd) };
    let parent_data = get_menu_data(parent);
    parent_data.htheme.unwrap()
}

fn toggle_checked(item:&mut InnerMenuItem, checked:bool){
    if checked {
        item.state.0 |= MENU_CHECKED.0
    }else{
        item.state.0 &= !MENU_CHECKED.0;
    }
}

fn toggle_radio(data:&mut MenuData, index:usize){

    toggle_checked(&mut data.items[index], true);

    for i in 0..data.items.len() {
        if data.items[i].menu_type == RMT_RADIO && data.items[i].name.as_ref().unwrap() == data.items[index].name.as_ref().unwrap() {
            if i != index {
                toggle_checked(&mut data.items[i], false);
            }
        }
    }
}

fn on_theme_change(hwnd:HWND, theme:Option<Theme>){

    let is_dark = if theme.is_some() { theme.unwrap() == Theme::Dark } else { should_apps_use_dark_mode() };
    allow_dark_mode_for_window(hwnd, is_dark);

    let data = get_menu_data_mut(hwnd);
    let old_htheme = data.htheme.unwrap();
    unsafe { CloseThemeData(old_htheme).unwrap() };
    let htheme = unsafe { OpenThemeDataEx(hwnd, w!("Menu"), OTD_NONCLIENT) };
    data.htheme = Some(htheme);

    data.theme = if is_dark { Theme::Dark } else { Theme::Light };
    set_menu_data(hwnd, data);
    unsafe { UpdateWindow(hwnd) };

    for item in &data.items {
        if item.menu_type == RMT_SUBMENU {
            let submenu_hwnd = item.submenu.unwrap();
            let data = get_menu_data_mut(submenu_hwnd);
            data.theme = if is_dark { Theme::Dark } else { Theme::Light };
            set_menu_data(submenu_hwnd, data);
            unsafe { UpdateWindow(submenu_hwnd) };
        }
    }

}

fn create_container_hwnd(parent: HWND, theme:Theme) -> Result<HWND, Error> {

    let class_name = w!("R_POPUPMENU");

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

    let hwnd = unsafe {
      CreateWindowExW(
        ex_style,
        PCWSTR::from_raw(class_name.as_ptr()),
        PCWSTR::null(),
        window_styles,
        0,
        0,
        0,
        0,
        parent,
        None,
        GetModuleHandleW(PCWSTR::null()).unwrap_or_default(),
        None,
      )
    };

    allow_dark_mode_for_window(hwnd, theme == Theme::Dark);

    Ok(hwnd)
}

fn allow_dark_mode_for_window(hwnd: HWND, is_dark: bool) {
    const UXTHEME_ALLOWDARKMODEFORWINDOW_ORDINAL: u16 = 133;
    type AllowDarkModeForWindow = unsafe extern "system" fn(HWND, bool) -> bool;
    static ALLOW_DARK_MODE_FOR_WINDOW: Lazy<Option<AllowDarkModeForWindow>> = Lazy::new(|| unsafe {
        if HUXTHEME.is_invalid() {
            return None;
        }

        GetProcAddress(
            *HUXTHEME,
            PCSTR::from_raw(UXTHEME_ALLOWDARKMODEFORWINDOW_ORDINAL as usize as *mut _),
        )
        .map(|handle| std::mem::transmute(handle))
    });

    if let Some(_allow_dark_mode_for_window) = *ALLOW_DARK_MODE_FOR_WINDOW {
        unsafe { _allow_dark_mode_for_window(hwnd, is_dark) };
    }

}

fn should_apps_use_dark_mode() -> bool {
    const UXTHEME_SHOULDAPPSUSEDARKMODE_ORDINAL: u16 = 132;
    type ShouldAppsUseDarkMode = unsafe extern "system" fn() -> bool;
    static SHOULD_APPS_USE_DARK_MODE: Lazy<Option<ShouldAppsUseDarkMode>> = Lazy::new(|| unsafe {
        if HUXTHEME.is_invalid() {
            return None;
        }

        GetProcAddress(
            *HUXTHEME,
            PCSTR::from_raw(UXTHEME_SHOULDAPPSUSEDARKMODE_ORDINAL as usize as *mut _),
        )
        .map(|handle| std::mem::transmute(handle))
    });

    SHOULD_APPS_USE_DARK_MODE.map(|should_apps_use_dark_mode| unsafe { (should_apps_use_dark_mode)() })
    .unwrap_or(false)
}