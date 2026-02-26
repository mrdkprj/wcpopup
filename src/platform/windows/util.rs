use super::{
    create_write_factory,
    direct2d::{get_icon_space, get_text_metrics},
    ColorScheme, Config, Corner, IconSpace, MenuData, MenuItem, MenuItemType, Size, Theme, CORNER_RADIUS, DEFAULT_ICON_MARGIN, MIN_BUTTON_WIDTH,
};
use crate::config::{hex_from_rgb, rgba_from_hex};
use std::{
    mem::{size_of, transmute},
    os::windows::ffi::OsStrExt,
    sync::LazyLock,
};
use windows::{
    core::{w, Error, PCSTR, PCWSTR},
    Win32::{
        Foundation::{FreeLibrary, COLORREF, HMODULE, HWND},
        Globalization::lstrlenW,
        Graphics::{
            DirectWrite::IDWriteFactory,
            Dwm::{DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_COLOR_NONE},
        },
        System::{
            Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED},
            LibraryLoader::{GetProcAddress, LoadLibraryW},
        },
        UI::WindowsAndMessaging::{GetWindowLongPtrW, SetWindowLongPtrW, GWL_USERDATA},
    },
    UI::ViewManagement::{UIColorType, UISettings},
};

static HUXTHEME: LazyLock<isize> = LazyLock::new(|| unsafe { LoadLibraryW(w!("uxtheme.dll")).unwrap_or_default().0 as _ });

macro_rules! hwnd {
    ($expression:expr) => {
        windows::Win32::Foundation::HWND($expression as isize as *mut std::ffi::c_void)
    };
}
pub(crate) use hwnd;

macro_rules! vtoi {
    ($expression:expr) => {
        $expression as *mut std::ffi::c_void as isize
    };
}
pub(crate) use vtoi;

pub(crate) fn free_library() {
    let _ = unsafe { FreeLibrary(HMODULE(*HUXTHEME as _)) };
}

pub(crate) fn clear_userdata(hwnd: HWND) {
    unsafe { SetWindowLongPtrW(hwnd, GWL_USERDATA, 0) };
}

pub(crate) fn is_userdata_avive(hwnd: HWND) -> bool {
    let userdata = unsafe { GetWindowLongPtrW(hwnd, GWL_USERDATA) };
    userdata == 0
}

pub(crate) fn get_menu_data<'a>(window_handle: isize) -> &'a MenuData {
    let userdata = unsafe { GetWindowLongPtrW(hwnd!(window_handle), GWL_USERDATA) };
    let item_data_ptr = userdata as *const MenuData;
    unsafe { &*item_data_ptr }
}

pub(crate) fn get_menu_data_mut<'a>(window_handle: isize) -> &'a mut MenuData {
    let userdata = unsafe { GetWindowLongPtrW(hwnd!(window_handle), GWL_USERDATA) };
    let item_data_ptr = userdata as *mut MenuData;
    unsafe { &mut *item_data_ptr }
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

pub(crate) fn toggle_radio(data: &mut MenuData, index: usize) {
    data.items[index].checked = true;

    for i in 0..data.items.len() {
        if data.items[i].menu_item_type == MenuItemType::Radio && data.items[i].name == data.items[index].name && i != index {
            data.items[i].checked = false;
        }
    }
}

pub(crate) fn calculate(items: &mut [MenuItem], config: &Config, theme: Theme, icon_space: IconSpace) -> Size {
    let mut width = 0;
    let mut height = 0;

    /* Add padding */
    height += config.size.vertical_padding;
    /* Add border size */
    height += config.size.border_size;

    if config.corner == Corner::Round {
        height += CORNER_RADIUS;
    }

    let factory = create_write_factory().unwrap();

    /* Find the widest accelerator string */
    let mut widest_accel = (0.0, "");
    let mut cloned_items = Vec::new();

    items.clone_into(&mut cloned_items);
    let accels = cloned_items.iter().map(|i| i.accelerator.as_str());
    for accel in accels {
        if !accel.is_empty() {
            let mut raw_text = encode_wide(accel);
            let metrics = get_text_metrics(&factory, theme, config, &mut raw_text).unwrap();
            if metrics.width >= widest_accel.0 {
                widest_accel = (metrics.width, accel);
            }
        }
    }

    /* Calculate item top, left, bottom and menu size */
    for (index, item) in items.iter_mut().enumerate() {
        item.index = index as u32;

        /* Don't measure invisible MenuItem */
        if !item.visible {
            item.top = -1;
            item.left = -1;
            item.bottom = -1;
            item.right = -1;
            continue;
        }

        item.top = height;
        item.left = config.size.border_size + config.size.horizontal_padding;
        let (item_width, item_height) = measure_item(&factory, config, item, theme, icon_space, widest_accel.1).unwrap();
        item.bottom = item.top + item_height;

        width = std::cmp::max(width, item_width);
        height += item_height;
    }

    /* Calculate item right */
    for item in items {
        if item.visible {
            item.right = item.left + width;
        }
    }

    /* Add padding */
    width += config.size.horizontal_padding * 2;
    height += config.size.vertical_padding;

    if config.corner == Corner::Round {
        height += CORNER_RADIUS;
    }

    /* Add border size */
    width += config.size.border_size * 2;
    height += config.size.border_size;

    Size {
        width,
        height,
    }
}

pub(crate) fn recalculate(data: &mut MenuData) {
    data.icon_space = get_icon_space(&data.items, data.config.icon.as_ref().unwrap(), &data.check_svg, &data.submenu_svg);
    data.size = calculate(&mut data.items, &data.config, data.current_theme, data.icon_space);
}

pub(crate) fn measure_item(factory: &IDWriteFactory, config: &Config, menu_item: &MenuItem, theme: Theme, icon_space: IconSpace, widest_accel: &str) -> Result<(i32, i32), Error> {
    let mut width = 0;
    let mut height = 0;

    match menu_item.menu_item_type {
        MenuItemType::Separator => {
            height += config.size.separator_size;
            height += config.size.item_vertical_padding * 2;
        }

        _ => {
            let mut raw_text = encode_wide(&menu_item.label);
            /* Set widest accelerator string */
            if !widest_accel.is_empty() {
                raw_text.extend(encode_wide(widest_accel));
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

            /* Add space for icon only when icon is set or reserve_icon_size is true */
            if menu_item.icon.is_some() || config.icon.as_ref().unwrap().reserve_icon_size {
                width += icon_space.mid.width + icon_space.mid.lmargin + icon_space.mid.rmargin;
            }

            width += icon_space.right.width + icon_space.right.lmargin + icon_space.right.rmargin;

            /* Add space for accelerator */
            if !menu_item.accelerator.is_empty() {
                width += MIN_BUTTON_WIDTH + DEFAULT_ICON_MARGIN;
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

pub(crate) fn set_window_border_color(window_handle: isize, data: &MenuData) -> Result<(), Error> {
    if is_win11() {
        let hwnd = hwnd!(window_handle);
        if data.config.size.border_size > 0 {
            let color = get_color_scheme(data).border;
            let rgb = rgba_from_hex(color);
            /* COLORREF red is last byte */
            let hex = hex_from_rgb(rgb.b, rgb.g, rgb.r);
            unsafe { DwmSetWindowAttribute(hwnd, DWMWA_BORDER_COLOR, &COLORREF(hex) as *const _ as *const _, size_of::<COLORREF>() as u32)? };
        } else {
            unsafe { DwmSetWindowAttribute(hwnd, DWMWA_BORDER_COLOR, &DWMWA_COLOR_NONE as *const _ as *const _, size_of::<COLORREF>() as u32)? };
        }
    }

    Ok(())
}

pub(crate) fn should_apps_use_dark_mode() -> bool {
    const UXTHEME_SHOULDAPPSUSEDARKMODE_ORDINAL: u16 = 132;
    type ShouldAppsUseDarkMode = unsafe extern "system" fn() -> bool;
    static SHOULD_APPS_USE_DARK_MODE: LazyLock<Option<ShouldAppsUseDarkMode>> = LazyLock::new(|| unsafe {
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

pub(crate) struct ComGuard;

impl ComGuard {
    pub fn new() -> Self {
        let _ = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        Self
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}
