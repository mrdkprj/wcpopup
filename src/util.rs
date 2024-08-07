use crate::{ColorScheme, MenuData, MenuItem, MenuItemState, MenuItemType, MenuSize, Theme, LR_BUTTON_SIZE, MENU_CHECKED, MENU_DISABLED, MENU_NORMAL};
use once_cell::sync::Lazy;
use std::{
    ffi::c_void,
    mem::{size_of, transmute},
    os::windows::ffi::OsStrExt,
};
use windows::{
    core::{w, Error, PCSTR, PCWSTR},
    Win32::{
        Foundation::{FreeLibrary, COLORREF, HMODULE, HWND, RECT},
        Globalization::lstrlenW,
        Graphics::{
            Dwm::{DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_COLOR_NONE},
            Gdi::{CreateFontIndirectW, DeleteObject, DrawTextW, GetDC, GetObjectW, ReleaseDC, SelectObject, DT_CALCRECT, DT_LEFT, DT_SINGLELINE, DT_VCENTER, LOGFONTW},
        },
        System::LibraryLoader::{GetProcAddress, LoadLibraryW},
        UI::WindowsAndMessaging::{
            GetSystemMetrics, GetWindowLongPtrW, SetWindowLongPtrW, SystemParametersInfoW, GWL_USERDATA, NONCLIENTMETRICSW, SM_CXMENUCHECK, SM_CYMENU, SPI_GETNONCLIENTMETRICS,
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
        },
    },
    UI::ViewManagement::{UIColorType, UISettings},
};

static HUXTHEME: Lazy<HMODULE> = Lazy::new(|| unsafe { LoadLibraryW(w!("uxtheme.dll")).unwrap_or_default() });

pub(crate) fn get_menu_data<'a>(hwnd: HWND) -> &'a MenuData {
    let userdata = unsafe { GetWindowLongPtrW(hwnd, GWL_USERDATA) };
    let item_data_ptr = userdata as *const MenuData;
    unsafe { &*item_data_ptr }
}

pub(crate) fn get_menu_data_mut<'a>(hwnd: HWND) -> &'a mut MenuData {
    let userdata = unsafe { GetWindowLongPtrW(hwnd, GWL_USERDATA) };
    let item_data_ptr = userdata as *mut MenuData;
    unsafe { &mut *item_data_ptr }
}

pub(crate) fn set_menu_data(hwnd: HWND, data: &mut MenuData) {
    unsafe { SetWindowLongPtrW(hwnd, GWL_USERDATA, Box::into_raw(Box::new(data.clone())) as _) };
}

pub(crate) fn encode_wide(string: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
    string.as_ref().encode_wide().chain(std::iter::once(0)).collect()
}

#[allow(dead_code)]
pub(crate) fn decode_wide(wide: &[u16]) -> String {
    let len = unsafe { lstrlenW(PCWSTR::from_raw(wide.as_ptr())) } as usize;
    let w_str_slice = unsafe { std::slice::from_raw_parts(wide.as_ptr(), len) };
    String::from_utf16_lossy(w_str_slice)
}

#[allow(dead_code)]
#[allow(non_snake_case)]
pub(crate) fn LOWORD(dword: u32) -> u16 {
    (dword & 0xFFFF) as u16
}

#[allow(dead_code)]
#[allow(non_snake_case)]
pub(crate) fn HIWORD(dword: u32) -> u16 {
    ((dword & 0xFFFF_0000) >> 16) as u16
}

pub(crate) fn create_state(disabled: Option<bool>, checked: Option<bool>) -> MenuItemState {
    let mut state = MENU_NORMAL.0;
    if disabled.is_some() && disabled.unwrap() {
        state |= MENU_DISABLED.0;
    }

    if checked.is_some() && checked.unwrap() {
        state |= MENU_CHECKED.0;
    }

    MenuItemState(state)
}

pub(crate) fn toggle_checked(item: &mut MenuItem, checked: bool) {
    if checked {
        item.state.0 |= MENU_CHECKED.0
    } else {
        item.state.0 &= !MENU_CHECKED.0;
    }
}

pub(crate) fn toggle_radio(data: &mut MenuData, index: usize) {
    toggle_checked(&mut data.items[index], true);

    for i in 0..data.items.len() {
        if data.items[i].menu_item_type == MenuItemType::Radio && data.items[i].name == data.items[index].name && i != index {
            toggle_checked(&mut data.items[i], false);
        }
    }
}

pub(crate) fn measure_item(hwnd: HWND, size: &MenuSize, item_data: &MenuItem, theme: Theme) -> Result<(i32, i32), Error> {
    let mut width = 0;
    let height;

    match item_data.menu_item_type {
        MenuItemType::Separator => {
            // separator - use half system height and zero width
            height = unsafe { (GetSystemMetrics(SM_CYMENU) + 4) / 2 };
        }

        _ => {
            let dc = unsafe { GetDC(hwnd) };
            let menu_font = get_font(theme, size, true)?;
            let font = unsafe { CreateFontIndirectW(&menu_font) };
            let old_font = unsafe { SelectObject(dc, font) };
            let mut text_rect = RECT::default();

            let mut raw_text = encode_wide(&item_data.label);
            if !item_data.accelerator.is_empty() {
                raw_text.extend(encode_wide(&item_data.accelerator));
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
            if !item_data.accelerator.is_empty() {
                cx += 30;
            }

            // Windows adds 1 to returned value
            cx -= unsafe { GetSystemMetrics(SM_CXMENUCHECK) - 1 };

            width = cx;

            let _ = unsafe { DeleteObject(font) };

            unsafe { ReleaseDC(hwnd, dc) };
        }
    }

    Ok((width, height))
}

fn get_current_theme(theme: Theme) -> Theme {
    let is_dark = if theme == Theme::System {
        is_sys_dark_color()
    } else {
        theme == Theme::Dark
    };

    if is_dark {
        Theme::Dark
    } else {
        Theme::Light
    }
}

pub(crate) fn get_color_scheme(data: &MenuData) -> &ColorScheme {
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

pub(crate) fn get_non_client_metrics() -> Result<NONCLIENTMETRICSW, Error> {
    let mut info = NONCLIENTMETRICSW {
        cbSize: size_of::<NONCLIENTMETRICSW>() as u32,
        ..Default::default()
    };

    unsafe { SystemParametersInfoW(SPI_GETNONCLIENTMETRICS, size_of::<NONCLIENTMETRICSW>() as u32, Some(&mut info as *mut _ as *mut c_void), SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0))? };

    Ok(info)
}

pub(crate) fn get_font(theme: Theme, size: &MenuSize, for_measure: bool) -> Result<LOGFONTW, Error> {
    let info = get_non_client_metrics()?;
    let mut menu_font = info.lfMenuFont;

    let current_theme = get_current_theme(theme);

    let (font_size, font_weight) = match current_theme {
        Theme::Dark => (size.dark_font_size, size.dark_font_weight),
        Theme::Light => (size.light_font_size, size.light_font_weight),
        Theme::System => (None, None),
    };

    if font_size.is_some() {
        menu_font.lfHeight = -font_size.unwrap();
    }

    if font_weight.is_some() {
        menu_font.lfWeight = font_weight.unwrap();
    }

    // When font is used for measurement, always use bold font
    if for_measure {
        menu_font.lfWeight = 700;
    }

    Ok(menu_font)
}

pub(crate) fn is_win11() -> bool {
    let version = windows_version::OsVersion::current();
    version.major == 10 && version.build >= 22000
}

pub(crate) fn free_library() {
    let _ = unsafe { FreeLibrary(*HUXTHEME) };
}

pub(crate) fn set_window_border_color(hwnd: HWND, data: &MenuData) -> Result<(), Error> {
    if is_win11() {
        if data.size.border_size > 0 {
            unsafe { DwmSetWindowAttribute(hwnd, DWMWA_BORDER_COLOR, &COLORREF(get_color_scheme(data).border) as *const _ as *const c_void, size_of::<COLORREF>() as u32)? };
        } else {
            unsafe { DwmSetWindowAttribute(hwnd, DWMWA_BORDER_COLOR, &DWMWA_COLOR_NONE as *const _ as *const c_void, size_of::<COLORREF>() as u32)? };
        }
    }

    Ok(())
}

pub(crate) fn allow_dark_mode_for_window(hwnd: HWND, is_dark: bool) {
    const UXTHEME_ALLOWDARKMODEFORWINDOW_ORDINAL: u16 = 133;
    type AllowDarkModeForWindow = unsafe extern "system" fn(HWND, bool) -> bool;
    static ALLOW_DARK_MODE_FOR_WINDOW: Lazy<Option<AllowDarkModeForWindow>> = Lazy::new(|| unsafe {
        if HUXTHEME.is_invalid() {
            return None;
        }

        GetProcAddress(*HUXTHEME, PCSTR::from_raw(UXTHEME_ALLOWDARKMODEFORWINDOW_ORDINAL as usize as *mut _)).map(|handle| transmute(handle))
    });

    if let Some(_allow_dark_mode_for_window) = *ALLOW_DARK_MODE_FOR_WINDOW {
        unsafe { _allow_dark_mode_for_window(hwnd, is_dark) };
    }
}

pub(crate) fn set_preferred_app_mode(theme: Theme) {
    #[allow(dead_code)]
    #[repr(C)]
    enum PreferredAppMode {
        Default,
        AllowDark,
        ForceDark,
        ForceLight,
        Max,
    }

    let preferred_theme = match theme {
        Theme::Dark => PreferredAppMode::ForceDark,
        Theme::Light => PreferredAppMode::ForceLight,
        Theme::System => PreferredAppMode::AllowDark,
    };

    const UXTHEME_SETPREFERREDAPPMODE_ORDINAL: u16 = 135;
    type SetPreferredAppMode = unsafe extern "system" fn(PreferredAppMode) -> PreferredAppMode;
    static SET_PREFERRED_APP_MODE: Lazy<Option<SetPreferredAppMode>> = Lazy::new(|| unsafe {
        if HUXTHEME.is_invalid() {
            return None;
        }

        GetProcAddress(*HUXTHEME, PCSTR::from_raw(UXTHEME_SETPREFERREDAPPMODE_ORDINAL as usize as *mut _)).map(|handle| transmute(handle))
    });

    if let Some(_set_preferred_app_mode) = *SET_PREFERRED_APP_MODE {
        unsafe { _set_preferred_app_mode(preferred_theme) };
    }
    refresh_immersive_color_policy_state();
}

fn refresh_immersive_color_policy_state() {
    const UXTHEME_REFRESHIMMERSIVECOLORPOLICYSTATE_ORDINAL: u16 = 104;
    type RefreshImmersiveColorPolicyState = unsafe extern "system" fn();
    static REFRESH_IMMERSIVE_COLOR_POLICY_STATE: Lazy<Option<RefreshImmersiveColorPolicyState>> = Lazy::new(|| unsafe {
        if HUXTHEME.is_invalid() {
            return None;
        }

        GetProcAddress(*HUXTHEME, PCSTR::from_raw(UXTHEME_REFRESHIMMERSIVECOLORPOLICYSTATE_ORDINAL as usize as *mut _)).map(|handle| std::mem::transmute(handle))
    });

    if let Some(_refresh_immersive_color_policy_state) = *REFRESH_IMMERSIVE_COLOR_POLICY_STATE {
        unsafe { _refresh_immersive_color_policy_state() }
    }
}

pub(crate) fn should_apps_use_dark_mode() -> bool {
    const UXTHEME_SHOULDAPPSUSEDARKMODE_ORDINAL: u16 = 132;
    type ShouldAppsUseDarkMode = unsafe extern "system" fn() -> bool;
    static SHOULD_APPS_USE_DARK_MODE: Lazy<Option<ShouldAppsUseDarkMode>> = Lazy::new(|| unsafe {
        if HUXTHEME.is_invalid() {
            return None;
        }

        GetProcAddress(*HUXTHEME, PCSTR::from_raw(UXTHEME_SHOULDAPPSUSEDARKMODE_ORDINAL as usize as *mut _)).map(|handle| transmute(handle))
    });

    SHOULD_APPS_USE_DARK_MODE.map(|should_apps_use_dark_mode| unsafe { (should_apps_use_dark_mode)() }).unwrap_or(false)
}

pub(crate) fn is_sys_dark_color() -> bool {
    let settings = UISettings::new().unwrap();
    let clr = settings.GetColorValue(UIColorType::Background).unwrap();
    let sum: u32 = (5 * clr.G as u32) + (2 * clr.R as u32) + (clr.B as u32);
    sum <= (8 * 128)
}
