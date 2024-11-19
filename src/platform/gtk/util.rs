use super::{FontWeight, MenuData, MenuItem};
use gtk::{
    ffi::{GtkAccelGroup, GtkMenu, GtkMenuItem, GtkWindow},
    glib::{
        translate::{FromGlibPtrNone, ToGlibPtr},
        IsA, ObjectExt,
    },
    prelude::GtkSettingsExt,
    AccelGroup, Widget,
};

pub(crate) fn get_menu_data<'a>(gtk_menu_handle: isize) -> &'a MenuData {
    let menu = to_gtk_menu(gtk_menu_handle);
    unsafe { menu.data::<MenuData>("data").unwrap().as_ref() }
}

pub(crate) fn get_menu_data_mut<'a>(gtk_menu_handle: isize) -> &'a mut MenuData {
    let menu = to_gtk_menu(gtk_menu_handle);
    unsafe { menu.data::<MenuData>("data").unwrap().as_mut() }
}

pub(crate) fn set_menu_data(gtk_menu_handle: isize, data: &mut MenuData) {
    let menu = to_gtk_menu(gtk_menu_handle);
    unsafe { menu.set_data("data", data.clone()) };
}

pub(crate) fn get_menu_item_data<'a>(gtk_menu_item: &impl IsA<Widget>) -> &'a MenuItem {
    unsafe { gtk_menu_item.data::<MenuItem>("data").unwrap().as_mut() }
}

pub(crate) fn get_menu_item_data_mut<'a>(gtk_menu_item: &impl IsA<Widget>) -> &'a mut MenuItem {
    unsafe { gtk_menu_item.data::<MenuItem>("data").unwrap().as_mut() }
}

pub(crate) fn set_menu_item_data(gtk_menu_item: &impl IsA<Widget>, item: &mut MenuItem) {
    unsafe { gtk_menu_item.set_data("data", item.clone()) };
}

pub(crate) fn to_gtk_window(gtk_window_handle: isize) -> gtk::Window {
    let window: gtk::Window = unsafe { gtk::Window::from_glib_none(gtk_window_handle as *mut GtkWindow) };
    window
}

pub(crate) fn from_gtk_window(gtk_window: &gtk::Window) -> isize {
    let ptr: *mut GtkWindow = gtk_window.to_glib_none().0;
    ptr as isize
}

pub(crate) fn to_gtk_menu(gtk_menu_handle: isize) -> gtk::Menu {
    let menu: gtk::Menu = unsafe { gtk::Menu::from_glib_none(gtk_menu_handle as *mut GtkMenu) };
    menu
}

pub(crate) fn from_gtk_menu(gtk_menu: &gtk::Menu) -> isize {
    let ptr: *mut GtkMenu = gtk_menu.to_glib_none().0;
    ptr as isize
}

pub(crate) fn to_gtk_menu_item(gtk_menu_item_handle: isize) -> gtk::MenuItem {
    let menu_item: gtk::MenuItem = unsafe { gtk::MenuItem::from_glib_none(gtk_menu_item_handle as *mut GtkMenuItem) };
    menu_item
}

pub(crate) fn from_gtk_menu_item(gtk_menu_item: &gtk::MenuItem) -> isize {
    let ptr: *mut GtkMenuItem = gtk_menu_item.to_glib_none().0;
    ptr as isize
}

pub(crate) fn to_accel_group(accel_group_handle: isize) -> AccelGroup {
    let accel_group: AccelGroup = unsafe { AccelGroup::from_glib_none(accel_group_handle as *mut GtkAccelGroup) };
    accel_group
}

pub(crate) fn from_accel_group(accel_group: &AccelGroup) -> isize {
    let ptr: *mut GtkAccelGroup = accel_group.to_glib_none().0;
    ptr as isize
}

pub(crate) fn to_font_weight(weight: FontWeight) -> u32 {
    match weight {
        FontWeight::Thin => 100,
        FontWeight::Light => 300,
        FontWeight::Normal => 400,
        FontWeight::Medium => 500,
        FontWeight::Bold => 700,
    }
}

pub(crate) fn to_font_weight_string<'a>(weight: u32) -> &'a str {
    match weight {
        100 => "Thin",
        300 => "Light",
        400 => "Regular",
        500 => "Medium",
        700 => "Bold",
        _ => "Regular",
    }
}

pub(crate) fn is_sys_dark() -> bool {
    if let Some(settings) = gtk::Settings::default() {
        if let Some(theme_name) = settings.gtk_theme_name() {
            return theme_name.as_str().to_lowercase().contains("dark");
        }
    }
    false
}
