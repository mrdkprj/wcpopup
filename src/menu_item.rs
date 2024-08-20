use crate::{
    builder::MenuBuilder,
    util::{get_menu_data_mut, set_menu_data, toggle_radio},
    Menu,
};
use serde::Serialize;
use std::sync::atomic::{AtomicU16, Ordering};

static UUID: AtomicU16 = AtomicU16::new(0);

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum MenuItemType {
    Text,
    Checkbox,
    Radio,
    Submenu,
    Separator,
}

/// Menu item.
#[derive(Debug, Clone, Serialize)]
pub struct MenuItem {
    pub id: String,
    pub label: String,
    pub accelerator: String,
    pub name: String,
    pub menu_item_type: MenuItemType,
    pub submenu: Option<Menu>,
    pub checked: bool,
    pub disabled: bool,
    pub uuid: u16,
    pub index: i32,
    pub(crate) menu_window_handle: isize,
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) right: i32,
    pub(crate) bottom: i32,
}

impl MenuItem {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(window_handle: isize, id: &str, label: &str, accelerator: &str, name: &str, checked: bool, disabled: Option<bool>, menu_item_type: MenuItemType, submenu: Option<Menu>) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            accelerator: accelerator.to_string(),
            name: name.to_string(),
            menu_item_type,
            submenu,
            menu_window_handle: window_handle,
            uuid: UUID.fetch_add(1, Ordering::Relaxed),
            index: 0,
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
            checked,
            disabled: disabled.unwrap_or(false),
        }
    }

    pub fn set_disabled(&self, disabled: bool) {
        let data = get_menu_data_mut(self.menu_window_handle);
        data.items[self.index as usize].disabled = disabled;
        set_menu_data(self.menu_window_handle, data);
    }

    pub fn set_label(&self, label: &str) {
        let data = get_menu_data_mut(self.menu_window_handle);
        data.items[self.index as usize].label = label.to_string();
        set_menu_data(self.menu_window_handle, data);
    }
}

impl MenuItem {
    pub fn new_text_item(id: &str, label: &str, accelerator: Option<&str>, disabled: Option<bool>) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            accelerator: accelerator.unwrap_or("").to_string(),
            name: String::new(),
            menu_item_type: MenuItemType::Text,
            submenu: None,
            menu_window_handle: 0,
            uuid: UUID.fetch_add(1, Ordering::Relaxed),
            index: 0,
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
            checked: false,
            disabled: disabled.unwrap_or(false),
        }
    }
}

impl MenuItem {
    pub fn new_check_item(id: &str, label: &str, accelerator: Option<&str>, checked: bool, disabled: Option<bool>) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            accelerator: accelerator.unwrap_or("").to_string(),
            name: String::new(),
            menu_item_type: MenuItemType::Checkbox,
            submenu: None,
            menu_window_handle: 0,
            uuid: UUID.fetch_add(1, Ordering::Relaxed),
            index: 0,
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
            checked,
            disabled: disabled.unwrap_or(false),
        }
    }

    pub fn new_radio_item(id: &str, label: &str, name: &str, accelerator: Option<&str>, checked: bool, disabled: Option<bool>) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            accelerator: accelerator.unwrap_or("").to_string(),
            name: name.to_string(),
            menu_item_type: MenuItemType::Radio,
            submenu: None,
            menu_window_handle: 0,
            uuid: UUID.fetch_add(1, Ordering::Relaxed),
            index: 0,
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
            checked,
            disabled: disabled.unwrap_or(false),
        }
    }

    pub fn set_checked(&self, checked: bool) {
        let data = get_menu_data_mut(self.menu_window_handle);
        let index = self.index as usize;
        if data.items[index].menu_item_type == MenuItemType::Checkbox {
            data.items[index].checked = checked;
        }
        if data.items[index].menu_item_type == MenuItemType::Radio {
            toggle_radio(data, index);
        }
        set_menu_data(self.menu_window_handle, data);
    }
}

/// Builder to create Submenu Item.
pub struct SubmenuItemBuilder {
    pub item: MenuItem,
    pub builder: MenuBuilder,
}

impl MenuItem {
    pub fn new_submenu_item(menu: &Menu, id: &str, label: &str, disabled: Option<bool>) -> SubmenuItemBuilder {
        let mut item = MenuItem::new(menu.window_handle, id, label, "", "", false, disabled, MenuItemType::Submenu, None);
        // Create builder
        let builder = MenuBuilder::new_for_submenu(menu);
        // Set dummy menu to be replaced later
        item.submenu = Some(builder.menu.clone());

        SubmenuItemBuilder {
            item,
            builder,
        }
    }
}

impl MenuItem {
    pub fn new_separator() -> Self {
        Self {
            id: String::new(),
            label: String::new(),
            accelerator: String::new(),
            name: String::new(),
            menu_item_type: MenuItemType::Separator,
            submenu: None,
            menu_window_handle: 0,
            uuid: UUID.fetch_add(1, Ordering::Relaxed),
            index: 0,
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
            checked: false,
            disabled: false,
        }
    }
}
