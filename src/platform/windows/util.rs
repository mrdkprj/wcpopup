use super::{direct2d::get_text_metrics, ColorScheme, Config, IconSpace, MenuData, MenuItem, MenuItemType, Theme};
use once_cell::sync::Lazy;
use std::{
    mem::{size_of, transmute},
    os::windows::ffi::OsStrExt,
};
use windows::{
    core::{w, Error, PCSTR, PCWSTR},
    Win32::{
        Foundation::{FreeLibrary, COLORREF, HMODULE},
        Globalization::lstrlenW,
        Graphics::{
            DirectWrite::IDWriteFactory,
            Dwm::{DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_COLOR_NONE},
        },
        System::LibraryLoader::{GetProcAddress, LoadLibraryW},
        UI::WindowsAndMessaging::{GetWindowLongPtrW, SetWindowLongPtrW, GWL_USERDATA},
    },
    UI::ViewManagement::{UIColorType, UISettings},
};

static HUXTHEME: Lazy<isize> = Lazy::new(|| unsafe { LoadLibraryW(w!("uxtheme.dll")).unwrap_or_default().0 as _ });

macro_rules! hw {
    ($expression:expr) => {
        windows::Win32::Foundation::HWND($expression as isize as *mut std::ffi::c_void)
    };
}
pub(crate) use hw;

macro_rules! vtoi {
    ($expression:expr) => {
        $expression as *mut std::ffi::c_void as isize
    };
}
pub(crate) use vtoi;

pub(crate) fn get_menu_data<'a>(window_handle: isize) -> &'a MenuData {
    let userdata = unsafe { GetWindowLongPtrW(hw!(window_handle), GWL_USERDATA) };
    let item_data_ptr = userdata as *const MenuData;
    unsafe { &*item_data_ptr }
}

pub(crate) fn get_menu_data_mut<'a>(window_handle: isize) -> &'a mut MenuData {
    let userdata = unsafe { GetWindowLongPtrW(hw!(window_handle), GWL_USERDATA) };
    let item_data_ptr = userdata as *mut MenuData;
    unsafe { &mut *item_data_ptr }
}

pub(crate) fn set_menu_data(window_handle: isize, data: &mut MenuData) {
    unsafe { SetWindowLongPtrW(hw!(window_handle), GWL_USERDATA, Box::into_raw(Box::new(data.clone())) as _) };
}

pub(crate) fn encode_wide(string: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
    string.as_ref().encode_wide().chain(std::iter::once(0)).collect()
}

pub(crate) fn to_pcwstr(string: impl AsRef<std::ffi::OsStr>) -> PCWSTR {
    PCWSTR::from_raw(encode_wide(string).as_ptr())
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

pub(crate) fn toggle_radio(data: &mut MenuData, index: usize) {
    data.items[index].checked = true;

    for i in 0..data.items.len() {
        if data.items[i].menu_item_type == MenuItemType::Radio && data.items[i].name == data.items[index].name && i != index {
            data.items[i].checked = false;
        }
    }
}

pub(crate) fn measure_item(factory: &IDWriteFactory, config: &Config, item_data: &MenuItem, theme: Theme, icon_space: IconSpace) -> Result<(i32, i32), Error> {
    let mut width = 0;
    let mut height = 0;

    match item_data.menu_item_type {
        MenuItemType::Separator => {
            height += config.size.separator_size;
            height += config.size.item_vertical_padding * 2;
        }

        _ => {
            let mut raw_text = encode_wide(&item_data.label);
            if !item_data.accelerator.is_empty() {
                raw_text.extend(encode_wide(&item_data.accelerator));
            }

            let metrics = get_text_metrics(factory, theme, config, &mut raw_text)?;

            height = metrics.height as i32;
            if height < 0 {
                height = -height;
            }
            let icon_height = std::cmp::max(icon_space.left.width, icon_space.right.width);
            if height < icon_height {
                height += icon_height - height;
            }
            height += config.size.item_vertical_padding * 2;

            width = metrics.width as i32;
            width += config.size.item_horizontal_padding * 2;
            width += icon_space.left.width + icon_space.left.lmargin + icon_space.left.rmargin;
            width += icon_space.mid.width + icon_space.mid.lmargin + icon_space.mid.rmargin;
            width += icon_space.right.width + icon_space.right.lmargin + icon_space.right.rmargin;
            /* extra padding for accelerator */
            if !item_data.accelerator.is_empty() {
                width += (config.font.dark_font_size.max(config.font.light_font_size) * 2.5) as i32;
            }
        }
    }

    Ok((width, height))
}

pub(crate) fn get_current_theme(theme: Theme) -> Theme {
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
    let is_dark = if data.current_theme == Theme::System {
        is_sys_dark_color()
    } else {
        data.current_theme == Theme::Dark
    };

    if is_dark {
        &data.config.color.dark
    } else {
        &data.config.color.light
    }
}

pub(crate) fn is_win11() -> bool {
    let version = windows_version::OsVersion::current();
    version.major == 10 && version.build >= 22000
}

pub(crate) fn free_library() {
    let _ = unsafe { FreeLibrary(HMODULE(*HUXTHEME as _)) };
}

pub(crate) fn set_window_border_color(window_handle: isize, data: &MenuData) -> Result<(), Error> {
    if is_win11() {
        let hwnd = hw!(window_handle);
        if data.config.size.border_size > 0 {
            unsafe { DwmSetWindowAttribute(hwnd, DWMWA_BORDER_COLOR, &COLORREF(get_color_scheme(data).border) as *const _ as *const _, size_of::<COLORREF>() as u32)? };
        } else {
            unsafe { DwmSetWindowAttribute(hwnd, DWMWA_BORDER_COLOR, &DWMWA_COLOR_NONE as *const _ as *const _, size_of::<COLORREF>() as u32)? };
        }
    }

    Ok(())
}

pub(crate) fn should_apps_use_dark_mode() -> bool {
    const UXTHEME_SHOULDAPPSUSEDARKMODE_ORDINAL: u16 = 132;
    type ShouldAppsUseDarkMode = unsafe extern "system" fn() -> bool;
    static SHOULD_APPS_USE_DARK_MODE: Lazy<Option<ShouldAppsUseDarkMode>> = Lazy::new(|| unsafe {
        if HMODULE(*HUXTHEME as _).is_invalid() {
            return None;
        }

        GetProcAddress(HMODULE(*HUXTHEME as _), PCSTR::from_raw(UXTHEME_SHOULDAPPSUSEDARKMODE_ORDINAL as usize as *mut _)).map(|handle| transmute(handle))
    });

    SHOULD_APPS_USE_DARK_MODE.map(|should_apps_use_dark_mode| unsafe { (should_apps_use_dark_mode)() }).unwrap_or(false)
}

pub(crate) fn is_sys_dark_color() -> bool {
    let settings = UISettings::new().unwrap();
    let clr = settings.GetColorValue(UIColorType::Background).unwrap();
    let sum: u32 = (5 * clr.G as u32) + (2 * clr.R as u32) + (clr.B as u32);
    sum <= (8 * 128)
}
