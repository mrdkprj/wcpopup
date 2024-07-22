#[cfg(feature = "accelerator")]
use std::collections::HashMap;
use std::mem::size_of;
use std::os::raw::c_void;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};

#[cfg(feature = "accelerator")]
use crate::accelerator::create_haccel;
use crate::{create_state, get_menu_data, Config, Corner, Menu, MenuData, MenuItem, MenuItemType, MenuType, Theme};
use windows::core::{w, Error};
use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND, DWM_WINDOW_CORNER_PREFERENCE};
use windows::Win32::UI::Controls::OTD_NONCLIENT;

use windows::Win32::UI::WindowsAndMessaging::{SetWindowLongPtrW, GWL_USERDATA};
use windows::Win32::{Foundation::HWND, UI::Controls::OpenThemeDataEx};

static COUNTER: AtomicU32 = AtomicU32::new(400);

/// Builder to create Menu.
pub struct MenuBuilder {
    pub(crate) menu: Menu,
    items: Vec<MenuItem>,
    config: Config,
    menu_type: MenuType,
}

impl MenuBuilder {
    /// Creates a new Menu for the specified window.
    pub fn new(parent: HWND) -> Self {
        let mut menu = Menu::default();
        menu.parent = parent;
        menu.hwnd = menu.create_window(parent, Theme::Light);
        Self {
            menu,
            items: Vec::new(),
            config: Config::default(),
            menu_type: MenuType::Main,
        }
    }

    /// Creates a new Menu with the specified Theme for the specified window.
    pub fn new_with_theme(parent: HWND, theme: Theme) -> Self {
        let mut menu = Menu::default();
        menu.parent = parent;
        menu.hwnd = menu.create_window(parent, theme);
        let mut config = Config::default();
        config.theme = theme;
        Self {
            menu,
            items: Vec::new(),
            config,
            menu_type: MenuType::Main,
        }
    }

    /// Creates a new Menu using the specified Config for the specified window.
    pub fn new_from_config(parent: HWND, config: Config) -> Self {
        let mut menu = Menu::default();
        menu.parent = parent;
        menu.hwnd = menu.create_window(parent, config.theme);

        Self {
            menu,
            items: Vec::new(),
            config,
            menu_type: MenuType::Main,
        }
    }

    pub(crate) fn new_from_menu(parent: &Menu) -> Self {
        let data = get_menu_data(parent.hwnd);
        let config = Config {
            theme: data.theme,
            size: data.size.clone(),
            color: data.color.clone(),
            corner: data.corner.clone(),
        };

        let mut menu = Menu::default();
        menu.parent = parent.hwnd;
        menu.hwnd = menu.create_window(parent.hwnd, config.theme);

        Self {
            menu,
            items: Vec::new(),
            config,
            menu_type: MenuType::Submenu,
        }
    }

    /// Adds a text MenuItem to Menu.
    pub fn text(&mut self, id: &str, label: &str, disabled: Option<bool>) -> &Self {
        let item = MenuItem::new(self.menu.hwnd, id, label, "", "", "", create_state(disabled, None), MenuItemType::Text, None);
        self.items.push(item);
        self
    }

    pub fn text_with_accelerator(&mut self, id: &str, label: &str, disabled: Option<bool>, accelerator: &str) -> &Self {
        let item = MenuItem::new(self.menu.hwnd, id, label, "", accelerator, "", create_state(disabled, None), MenuItemType::Text, None);
        self.items.push(item);
        self
    }

    /// Adds a check MenuItem to Menu.
    pub fn check(&mut self, id: &str, label: &str, value: &str, checked: bool, disabled: Option<bool>) -> &Self {
        let item = MenuItem::new(self.menu.hwnd, id, label, value, "", "", create_state(disabled, Some(checked)), MenuItemType::Checkbox, None);
        self.items.push(item);
        self
    }

    pub fn check_with_accelerator(&mut self, id: &str, label: &str, value: &str, checked: bool, disabled: Option<bool>, accelerator: &str) -> &Self {
        let item = MenuItem::new(self.menu.hwnd, id, label, value, accelerator, "", create_state(disabled, Some(checked)), MenuItemType::Checkbox, None);
        self.items.push(item);
        self
    }

    /// Adds a radio MenuItem to Menu.
    pub fn radio(&mut self, id: &str, label: &str, value: &str, name: &str, checked: bool, disabled: Option<bool>) -> &Self {
        let item = MenuItem::new(self.menu.hwnd, id, label, value, "", name, create_state(disabled, Some(checked)), MenuItemType::Radio, None);
        self.items.push(item);
        self
    }

    pub fn radio_with_accelerator(&mut self, id: &str, label: &str, value: &str, name: &str, checked: bool, disabled: Option<bool>, accelerator: &str) -> &Self {
        let item = MenuItem::new(self.menu.hwnd, id, label, value, accelerator, name, create_state(disabled, Some(checked)), MenuItemType::Radio, None);
        self.items.push(item);
        self
    }

    /// Adds a separator to Menu.
    pub fn separator(&mut self) -> &Self {
        self.items.push(MenuItem::new(self.menu.hwnd, "", "", "", "", "", create_state(None, None), MenuItemType::Separator, None));
        self
    }

    /// Adds a submenu MenuItem to Menu.
    pub fn submenu(&mut self, label: &str, disabled: Option<bool>) -> Self {
        let mut item = MenuItem::new(self.menu.hwnd, label, label, "", "", "", create_state(disabled, None), MenuItemType::Submenu, None);
        /* Create builder */
        let mut builder = Self::new_from_config(self.menu.hwnd, self.config.clone());
        builder.menu_type = MenuType::Submenu;

        item.submenu = Some(builder.menu.clone());
        self.items.push(item);

        builder
    }

    /// Build Menu to make it ready to become visible.
    /// Must call this function before showing Menu, otherwise nothing shows up.
    pub fn build(mut self) -> Result<Menu, Error> {
        let size = self.menu.calculate(&mut self.items, &self.config.size, self.config.theme, self.config.corner)?;
        let is_main_menu = self.menu_type == MenuType::Main;

        #[cfg(feature = "accelerator")]
        let mut accelerators = HashMap::new();
        #[cfg(feature = "accelerator")]
        let mut haccel = None;
        #[cfg(feature = "accelerator")]
        if is_main_menu {
            Self::collect_accelerators(&self.items, &mut accelerators);
            if !accelerators.is_empty() {
                match create_haccel(&accelerators) {
                    Some(accel) => haccel = Some(Rc::new(accel)),
                    None => haccel = None,
                }
            }
        }

        let data = MenuData {
            menu_type: self.menu_type,
            items: self.items.clone(),
            h_theme: if is_main_menu {
                Some(unsafe { Rc::new(OpenThemeDataEx(self.menu.hwnd, w!("Menu"), OTD_NONCLIENT)) })
            } else {
                None
            },
            win_subclass_id: if is_main_menu {
                Some(COUNTER.fetch_add(1, Ordering::Relaxed))
            } else {
                None
            },
            #[cfg(feature = "accelerator")]
            haccel,
            #[cfg(feature = "accelerator")]
            accelerators,
            height: size.height,
            width: size.width,
            selected_index: -1,
            visible_submenu_index: -1,
            theme: self.config.theme,
            size: self.config.size.clone(),
            color: self.config.color.clone(),
            corner: self.config.corner.clone(),
            thread_id: 0,
            parent: if is_main_menu {
                HWND(0)
            } else {
                self.menu.parent
            },
        };

        if is_main_menu {
            self.menu.attach_owner_subclass(data.win_subclass_id.unwrap() as usize);
        }

        if Self::is_win11() && self.config.corner == Corner::Round {
            unsafe { DwmSetWindowAttribute(self.menu.hwnd, DWMWA_WINDOW_CORNER_PREFERENCE, &DWMWCP_ROUND as *const _ as *const c_void, size_of::<DWM_WINDOW_CORNER_PREFERENCE>() as u32).unwrap() };
        }

        unsafe { SetWindowLongPtrW(self.menu.hwnd, GWL_USERDATA, Box::into_raw(Box::new(data)) as _) };

        Ok(self.menu)
    }

    fn is_win11() -> bool {
        let version = windows_version::OsVersion::current();
        if version.major == 10 && version.build >= 22000 {
            true
        } else {
            false
        }
    }

    #[cfg(feature = "accelerator")]
    fn collect_accelerators(items: &Vec<MenuItem>, accelerators: &mut HashMap<u16, String>) {
        for item in items {
            if item.menu_item_type == MenuItemType::Submenu {
                let submenu_hwnd = item.submenu.as_ref().unwrap().hwnd;
                let data = get_menu_data(submenu_hwnd);
                Self::collect_accelerators(&data.items, accelerators);
            } else {
                if !item.accelerator.is_empty() {
                    accelerators.insert(item.uuid, item.accelerator.clone());
                }
            }
        }
    }
}
