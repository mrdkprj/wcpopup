use std::sync::atomic::{AtomicU32, Ordering};

use crate::{create_state, get_menu_data, Config, Menu, MenuData, MenuItem, MenuItemType, MenuType, Theme};
use windows::core::{w, Error};
use windows::Win32::UI::Controls::OTD_NONCLIENT;
use windows::Win32::UI::WindowsAndMessaging::{SetWindowLongPtrW, GWL_USERDATA};
use windows::Win32::{Foundation::HWND, UI::Controls::OpenThemeDataEx};

static COUNTER: AtomicU32 = AtomicU32::new(400);
pub struct MenuBuilder {
    pub(crate) menu: Menu,
    items: Vec<MenuItem>,
    config: Config,
    menu_type: MenuType,
}

impl MenuBuilder {
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

    pub fn text(&mut self, id: &str, label: &str, disabled: Option<bool>) -> &Self {
        self.items.push(MenuItem::new(self.menu.hwnd, id, label, "", "", "", create_state(disabled, None), MenuItemType::Text, None));
        self
    }

    pub fn text_with_accelerator(&mut self, id: &str, label: &str, disabled: Option<bool>, accelerator: &str) -> &Self {
        self.items.push(MenuItem::new(self.menu.hwnd, id, label, "", accelerator, "", create_state(disabled, None), MenuItemType::Text, None));
        self
    }

    pub fn check(&mut self, id: &str, label: &str, value: &str, checked: bool, disabled: Option<bool>) -> &Self {
        self.items.push(MenuItem::new(self.menu.hwnd, id, label, value, "", "", create_state(disabled, Some(checked)), MenuItemType::Checkbox, None));
        self
    }

    pub fn check_with_accelerator(&mut self, id: &str, label: &str, value: &str, checked: bool, disabled: Option<bool>, accelerator: &str) -> &Self {
        self.items.push(MenuItem::new(self.menu.hwnd, id, label, value, accelerator, "", create_state(disabled, Some(checked)), MenuItemType::Checkbox, None));
        self
    }

    pub fn radio(&mut self, id: &str, label: &str, value: &str, name: &str, checked: bool, disabled: Option<bool>) -> &Self {
        self.items.push(MenuItem::new(self.menu.hwnd, id, label, value, "", name, create_state(disabled, Some(checked)), MenuItemType::Radio, None));
        self
    }

    pub fn radio_with_accelerator(&mut self, id: &str, label: &str, value: &str, name: &str, checked: bool, disabled: Option<bool>, accelerator: &str) -> &Self {
        self.items.push(MenuItem::new(self.menu.hwnd, id, label, value, accelerator, name, create_state(disabled, Some(checked)), MenuItemType::Radio, None));
        self
    }

    pub fn separator(&mut self) -> &Self {
        self.items.push(MenuItem::new(self.menu.hwnd, "", "", "", "", "", create_state(None, None), MenuItemType::Separator, None));
        self
    }

    pub fn submenu(&mut self, label: &str, disabled: Option<bool>) -> Self {
        let mut item = MenuItem::new(self.menu.hwnd, label, label, "", "", "", create_state(disabled, None), MenuItemType::Submenu, None);
        /* Create builder */
        let mut builder = Self::new_from_config(self.menu.hwnd, self.config.clone());
        builder.menu_type = MenuType::Submenu;

        item.submenu = Some(builder.menu.clone());
        self.items.push(item);

        builder
    }

    pub fn build(mut self) -> Result<Menu, Error> {
        let size = self.menu.calculate(&mut self.items, &self.config.size, self.config.theme)?;
        let is_main_menu = self.menu_type == MenuType::Main;

        let data = MenuData {
            //index: self.menu.index,
            menu_type: self.menu_type,
            items: self.items.clone(),
            htheme: if is_main_menu {
                Some(unsafe { OpenThemeDataEx(self.menu.hwnd, w!("Menu"), OTD_NONCLIENT) })
            } else {
                None
            },
            win_subclass_id: if is_main_menu {
                Some(COUNTER.fetch_add(1, Ordering::Relaxed))
            } else {
                None
            },
            height: size.height,
            width: size.width,
            selected_index: -1,
            visible_submenu_index: -1,
            theme: self.config.theme,
            size: self.config.size.clone(),
            color: self.config.color.clone(),
        };

        if is_main_menu {
            self.menu.attach_owner_subclass(data.win_subclass_id.unwrap() as usize);
        }

        unsafe { SetWindowLongPtrW(self.menu.hwnd, GWL_USERDATA, Box::into_raw(Box::new(data)) as _) };

        Ok(self.menu)
    }
}
