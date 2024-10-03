use super::{
    recalculate,
    util::{get_menu_data_mut, set_menu_data, toggle_radio},
    Menu,
};
use crate::MenuItemType;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU16, Ordering};

static UUID: AtomicU16 = AtomicU16::new(0);

/// Menu item.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub(crate) items: Option<Vec<MenuItem>>,
    pub(crate) icon: Option<std::path::PathBuf>,
}

impl MenuItem {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        window_handle: isize,
        id: &str,
        label: &str,
        accelerator: &str,
        name: &str,
        checked: bool,
        disabled: Option<bool>,
        menu_item_type: MenuItemType,
        submenu: Option<Menu>,
        icon: Option<std::path::PathBuf>,
    ) -> Self {
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
            items: None,
            icon,
        }
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
        let data = get_menu_data_mut(self.menu_window_handle);
        data.items[self.index as usize].disabled = disabled;
        set_menu_data(self.menu_window_handle, data);
    }

    pub fn set_label(&mut self, label: &str) {
        self.label = label.to_string();
        let data = get_menu_data_mut(self.menu_window_handle);
        data.items[self.index as usize].label = label.to_string();
        recalculate(data);
        set_menu_data(self.menu_window_handle, data);
    }
}

impl MenuItem {
    pub fn new_text_item(id: &str, label: &str, accelerator: Option<&str>, disabled: Option<bool>, icon: Option<std::path::PathBuf>) -> Self {
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
            items: None,
            icon,
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
            items: None,
            icon: None,
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
            items: None,
            icon: None,
        }
    }

    pub fn set_checked(&mut self, checked: bool) {
        self.checked = checked;
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

impl MenuItem {
    pub fn new_submenu_item(id: &str, label: &str, disabled: Option<bool>, icon: Option<std::path::PathBuf>) -> Self {
        let mut item = MenuItem::new(0, id, label, "", "", false, disabled, MenuItemType::Submenu, None, icon);
        item.items = Some(Vec::new());
        item
    }
    pub fn add_menu_item(&mut self, item: MenuItem) -> &Self {
        if let Some(items) = self.items.as_mut() {
            items.push(item);
        }
        self
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
            items: None,
            icon: None,
        }
    }
}
