use super::{
    direct2d::create_menu_image,
    recalculate,
    util::{get_menu_data_mut, toggle_radio},
    Menu,
};
use crate::{MenuIcon, MenuItemType};
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
    pub visible: bool,
    pub icon: Option<MenuIcon>,
    pub uuid: u16,
    pub index: u32,
    pub(crate) menu_window_handle: isize,
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) right: i32,
    pub(crate) bottom: i32,
    pub(crate) items: Option<Vec<MenuItem>>,
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
        disabled: bool,
        menu_item_type: MenuItemType,
        submenu: Option<Menu>,
        icon: Option<MenuIcon>,
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
            disabled,
            visible: true,
            items: None,
            icon,
        }
    }

    pub fn set_label(&mut self, label: &str) {
        self.label = label.to_string();

        /* Exit if window is not created */
        if self.menu_window_handle == 0 {
            return;
        }
        let data = get_menu_data_mut(self.menu_window_handle);
        data.items[self.index as usize].label = label.to_string();
        recalculate(data);
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;

        /* Exit if window is not created */
        if self.menu_window_handle == 0 {
            return;
        }
        let data = get_menu_data_mut(self.menu_window_handle);
        data.items[self.index as usize].disabled = disabled;
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;

        /* Exit if window is not created */
        if self.menu_window_handle == 0 {
            return;
        }

        let data = get_menu_data_mut(self.menu_window_handle);
        data.items[self.index as usize].visible = visible;
        recalculate(data);
    }

    pub fn set_icon(&mut self, icon: Option<MenuIcon>) {
        if self.menu_item_type == MenuItemType::Separator {
            return;
        }

        self.icon = icon;

        /* Exit if window is not created */
        if self.menu_window_handle == 0 {
            return;
        }

        let data = get_menu_data_mut(self.menu_window_handle);

        let _ = data.icon_map.remove(&self.uuid);

        if let Some(icon) = &self.icon {
            let bitmap = create_menu_image(&data.dc_render_target, icon, data.icon_size).unwrap();
            data.icon_map.insert(self.uuid, bitmap);
        }

        data.items[self.index as usize].icon.clone_from(&self.icon);

        recalculate(data);
    }
}

impl MenuItem {
    pub fn new_text_item(id: &str, label: &str, accelerator: Option<&str>, disabled: bool, icon: Option<MenuIcon>) -> Self {
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
            disabled,
            visible: true,
            items: None,
            icon,
        }
    }
}

impl MenuItem {
    pub fn new_check_item(id: &str, label: &str, accelerator: Option<&str>, checked: bool, disabled: bool, icon: Option<MenuIcon>) -> Self {
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
            disabled,
            visible: true,
            items: None,
            icon,
        }
    }

    pub fn new_radio_item(id: &str, label: &str, name: &str, accelerator: Option<&str>, checked: bool, disabled: bool, icon: Option<MenuIcon>) -> Self {
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
            disabled,
            visible: true,
            items: None,
            icon,
        }
    }

    pub fn set_checked(&mut self, checked: bool) {
        self.checked = checked;

        /* Exit if window is not created */
        if self.menu_window_handle == 0 {
            return;
        }

        let data = get_menu_data_mut(self.menu_window_handle);
        let index = self.index as usize;
        if data.items[index].menu_item_type == MenuItemType::Checkbox {
            data.items[index].checked = checked;
        }
        if data.items[index].menu_item_type == MenuItemType::Radio {
            toggle_radio(data, index);
        }
    }
}

impl MenuItem {
    pub fn new_submenu_item(id: &str, label: &str, disabled: bool, icon: Option<MenuIcon>) -> Self {
        let mut item = MenuItem::new(0, id, label, "", "", false, disabled, MenuItemType::Submenu, None, icon);
        item.items = Some(Vec::new());
        item
    }

    /// Adds a MenuItem to submenu.
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
            visible: true,
            items: None,
            icon: None,
        }
    }

    pub fn new_separator_with_id(id: &str) -> Self {
        Self {
            id: id.to_string(),
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
            visible: true,
            items: None,
            icon: None,
        }
    }
}

impl MenuItem {
    pub fn builder(menu_item_type: MenuItemType) -> MenuItemBuilder {
        /* window handle is later set in append */
        MenuItemBuilder {
            menu_item: MenuItem::new(0, "", "", "", "", false, false, menu_item_type, None, None),
        }
    }
}

pub struct MenuItemBuilder {
    menu_item: MenuItem,
}

impl MenuItemBuilder {
    pub fn id(mut self, id: &str) -> Self {
        self.menu_item.id = id.to_string();
        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.menu_item.label = label.to_string();
        self
    }

    pub fn accelerator(mut self, accelerator: &str) -> Self {
        self.menu_item.accelerator = accelerator.to_string();
        self
    }

    pub fn name(mut self, name: &str) -> Self {
        self.menu_item.name = name.to_string();
        self
    }

    pub fn submenu(mut self, items: Vec<MenuItem>) -> Self {
        if self.menu_item.menu_item_type == MenuItemType::Submenu {
            self.menu_item.items = Some(items);
        }
        self
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.menu_item.checked = checked;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.menu_item.disabled = disabled;
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.menu_item.visible = visible;
        self
    }

    pub fn icon(mut self, icon: MenuIcon) -> Self {
        self.menu_item.icon = Some(icon);
        self
    }

    /// Build the [`MenuItem`].
    pub fn build(self) -> MenuItem {
        self.menu_item
    }
}
