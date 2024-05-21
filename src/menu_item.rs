use crate::{
    util::{get_menu_data, get_menu_data_mut, set_menu_data, toggle_checked, toggle_radio},
    RMenu,
};
use serde::Serialize;
use windows::Win32::Foundation::HWND;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuItemType {
    Text,
    Checkbox,
    Radio,
    Submenu,
    Separator,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MenuItemState(pub i32);
pub const MENU_NORMAL: MenuItemState = MenuItemState(1);
pub const MENU_CHECKED: MenuItemState = MenuItemState(2);
pub const MENU_DISABLED: MenuItemState = MenuItemState(4);

#[derive(Debug, Clone)]
pub struct MenuItem {
    pub id: String,
    pub label: String,
    pub value: String,
    pub accelerator: String,
    pub name: String,
    pub state: MenuItemState,
    pub menu_item_type: MenuItemType,
    pub sub: Option<RMenu>,
    pub(crate) submenu: Option<HWND>,
    pub(crate) hwnd: HWND,
    pub(crate) index: i32,
    pub(crate) top: i32,
    pub(crate) bottom: i32,
}

impl MenuItem {
    pub fn new(id: &str, label: &str, value: &str, accelerator: &str, name: &str, state: MenuItemState, menu_item_type: MenuItemType, submenu: Option<HWND>) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            value: value.to_string(),
            accelerator: accelerator.to_string(),
            name: name.to_string(),
            state,
            menu_item_type,
            submenu,
            hwnd: HWND(0),
            index: 0,
            top: 0,
            bottom: 0,
            sub: None,
        }
    }

    pub(crate) fn new_with_hwnd(hwnd: HWND, id: &str, label: &str, value: &str, accelerator: &str, name: &str, state: MenuItemState, menu_type: MenuItemType, submenu: Option<HWND>) -> Self {
        let mut item = Self::new(id, label, value, accelerator, name, state, menu_type, submenu);
        item.hwnd = hwnd;
        item
    }

    pub fn checked(&self) -> bool {
        let data = get_menu_data(self.hwnd);
        (data.items[self.index as usize].state.0 & MENU_CHECKED.0) != 0
    }

    pub fn set_checked(&self, checked: bool) {
        let data = get_menu_data_mut(self.hwnd);
        let index = self.index as usize;
        if data.items[index].menu_item_type == MenuItemType::Checkbox {
            toggle_checked(&mut data.items[index], checked);
        }
        if data.items[index].menu_item_type == MenuItemType::Radio {
            toggle_radio(data, index);
        }
        set_menu_data(self.hwnd, data);
    }

    pub fn disabled(&self) -> bool {
        let data = get_menu_data(self.hwnd);
        (data.items[self.index as usize].state.0 & MENU_DISABLED.0) != 0
    }

    pub fn set_disabled(&self, disabled: bool) {
        let data = get_menu_data_mut(self.hwnd);
        if disabled {
            data.items[self.index as usize].state.0 |= MENU_DISABLED.0;
        } else {
            data.items[self.index as usize].state.0 &= !MENU_DISABLED.0;
        }
        set_menu_data(self.hwnd, data);
    }

    pub fn set_label(&self, label: &str) {
        let data = get_menu_data_mut(self.hwnd);
        data.items[self.index as usize].label = label.to_string();
        set_menu_data(self.hwnd, data);
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SelectedMenuItem {
    pub id: String,
    pub label: String,
    pub value: String,
    pub name: String,
    pub state: MenuItemState,
}

impl SelectedMenuItem {
    pub(crate) fn from(item: &MenuItem) -> Self {
        Self {
            id: item.id.clone(),
            label: item.label.clone(),
            value: item.value.clone(),
            name: item.name.clone(),
            state: item.state.clone(),
        }
    }
}
