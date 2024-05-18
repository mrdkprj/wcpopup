use crate::MenuData;
use std::{mem::transmute, os::windows::ffi::OsStrExt};
use serde::Serialize;
use windows_core::PCWSTR;
use windows::Win32::{Foundation::HWND, Globalization::lstrlenW, UI::WindowsAndMessaging::{GetWindowLongPtrW, SetWindowLongPtrW, GWL_USERDATA}};

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RMENU_TYPE(pub i32);
pub const RMT_TEXT:RMENU_TYPE = RMENU_TYPE(0);
pub const RMT_CHECKBOX:RMENU_TYPE = RMENU_TYPE(1);
pub const RMT_RADIO:RMENU_TYPE = RMENU_TYPE(2);
pub const RMT_SUBMENU:RMENU_TYPE = RMENU_TYPE(3);
pub const RMT_SEPARATOR:RMENU_TYPE = RMENU_TYPE(4);

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MenuItemState(pub i32);
pub const MENU_NORMAL:MenuItemState = MenuItemState(1);
pub const MENU_CHECKED:MenuItemState = MenuItemState(2);
pub const MENU_DISABLED:MenuItemState = MenuItemState(4);

pub(crate) fn get_menu_data<'a>(hwnd:HWND) -> &'a MenuData {
  let userdata = unsafe { GetWindowLongPtrW(hwnd, GWL_USERDATA) };
  unsafe { transmute::<isize, &MenuData>(userdata) }
}

pub(crate) fn get_menu_data_mut<'a>(hwnd:HWND) -> &'a mut MenuData {
  let userdata = unsafe { GetWindowLongPtrW(hwnd, GWL_USERDATA) };
  unsafe { transmute::<isize, &mut MenuData>(userdata) }
}

pub(crate) fn set_menu_data(hwnd:HWND, data:&mut MenuData){
  unsafe { SetWindowLongPtrW(hwnd, GWL_USERDATA, transmute::<&mut MenuData, isize>(data)) };
}

pub(crate) fn encode_wide(string: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
    string.as_ref().encode_wide().chain(std::iter::once(0)).collect()
}

pub(crate) fn decode_wide(wide: &Vec<u16>) -> String {
    let len = unsafe { lstrlenW(PCWSTR::from_raw(wide.as_ptr())) } as usize;
    let w_str_slice = unsafe { std::slice::from_raw_parts(wide.as_ptr(), len) };
    String::from_utf16_lossy(w_str_slice)
}

#[allow(non_snake_case)]
pub(crate) fn LOWORD(dword: u32) -> u16 {
  (dword & 0xFFFF) as u16
}

#[allow(non_snake_case)]
pub(crate) fn HIWORD(dword: u32) -> u16 {
  ((dword & 0xFFFF_0000) >> 16) as u16
}
