use std::sync::atomic::{AtomicU32, Ordering};

use crate::{Config, MenuData, MenuItem, MenuItemState, MenuItemType, RMenu, Theme, MENU_CHECKED, MENU_DISABLED, MENU_NORMAL};
use windows::core::{w, Error};
use windows::Win32::UI::Controls::OTD_NONCLIENT;
use windows::Win32::UI::WindowsAndMessaging::{SetWindowLongPtrW, GWL_USERDATA};
use windows::Win32::{Foundation::HWND, UI::Controls::OpenThemeDataEx};

static COUNTER: AtomicU32 = AtomicU32::new(400);

#[derive(PartialEq, Eq)]
enum MenuType {
    Main,
    Submenu,
}

pub struct RMenuBuilder {
    menu: RMenu,
    items: Vec<MenuItem>,
    config: Config,
    menu_type: MenuType,
}

impl RMenuBuilder {
    pub fn new(parent: HWND) -> Self {
        let mut menu = RMenu::default();
        menu.parent = parent;
        menu.hwnd = menu.create_window(parent, Theme::Light);
        Self { menu, items: Vec::new(), config: Config::default(), menu_type: MenuType::Main }
    }

    pub fn new_with_theme(parent: HWND, theme: Theme) -> Self {
        let mut menu = RMenu::default();
        menu.parent = parent;
        menu.hwnd = menu.create_window(parent, theme);
        let mut config = Config::default();
        config.theme = theme;
        Self { menu, items: Vec::new(), config, menu_type: MenuType::Main }
    }

    pub fn new_from_config(parent: HWND, config: Config) -> Self {
        let mut menu = RMenu::default();
        menu.parent = parent;
        menu.hwnd = menu.create_window(parent, config.theme);
        Self { menu, items: Vec::new(), config, menu_type: MenuType::Main }
    }

    fn create_state(disabled: Option<bool>, checked: Option<bool>) -> MenuItemState {
        let mut state = MENU_NORMAL.0;
        if disabled.is_some() && disabled.unwrap() {
            state |= MENU_DISABLED.0;
        }

        if checked.is_some() && checked.unwrap() {
            state |= MENU_CHECKED.0;
        }

        MenuItemState(state)
    }

    pub fn text(&mut self, id: &str, label: &str, disabled: Option<bool>) -> &Self {
        self.items
            .push(MenuItem::new_with_hwnd(self.menu.hwnd, id, label, "", "", "", Self::create_state(disabled, None), MenuItemType::Text, None));
        self
    }

    pub fn text_with_accelerator(&mut self, id: &str, label: &str, disabled: Option<bool>, accelerator: &str) -> &Self {
        self.items
            .push(MenuItem::new_with_hwnd(self.menu.hwnd, id, label, "", accelerator, "", Self::create_state(disabled, None), MenuItemType::Text, None));
        self
    }

    pub fn check(&mut self, id: &str, label: &str, value: &str, checked: bool, disabled: Option<bool>) -> &Self {
        self.items
            .push(MenuItem::new_with_hwnd(self.menu.hwnd, id, label, value, "", "", Self::create_state(disabled, Some(checked)), MenuItemType::Checkbox, None));
        self
    }

    pub fn check_with_accelerator(&mut self, id: &str, label: &str, value: &str, checked: bool, disabled: Option<bool>, accelerator: &str) -> &Self {
        self.items
            .push(MenuItem::new_with_hwnd(self.menu.hwnd, id, label, value, accelerator, "", Self::create_state(disabled, Some(checked)), MenuItemType::Checkbox, None));
        self
    }

    pub fn radio(&mut self, id: &str, label: &str, value: &str, name: &str, checked: bool, disabled: Option<bool>) -> &Self {
        self.items
            .push(MenuItem::new_with_hwnd(self.menu.hwnd, id, label, value, "", name, Self::create_state(disabled, Some(checked)), MenuItemType::Radio, None));
        self
    }

    pub fn radio_with_accelerator(&mut self, id: &str, label: &str, value: &str, name: &str, checked: bool, disabled: Option<bool>, accelerator: &str) -> &Self {
        self.items
            .push(MenuItem::new_with_hwnd(self.menu.hwnd, id, label, value, accelerator, name, Self::create_state(disabled, Some(checked)), MenuItemType::Radio, None));
        self
    }

    pub fn separator(&mut self) -> &Self {
        self.items
            .push(MenuItem::new_with_hwnd(self.menu.hwnd, "", "", "", "", "", MENU_NORMAL, MenuItemType::Separator, None));
        self
    }

    pub fn submenu(&mut self, label: &str) -> Self {
        let mut item = MenuItem::new_with_hwnd(self.menu.hwnd, label, label, "", "", "", MENU_NORMAL, MenuItemType::Submenu, None);
        /* Create builder */
        let mut builder = Self::new_from_config(self.menu.hwnd, self.config.clone());
        /* Set MenuItem index as RMenu index and Submenu type */
        builder.menu.index = item.index;
        builder.menu_type = MenuType::Submenu;

        item.submenu = Some(builder.menu.hwnd);
        item.sub = Some(builder.menu.clone());
        self.items.push(item);

        builder
    }

    pub fn build(mut self) -> Result<RMenu, Error> {
        let size = self
            .menu
            .calculate(&mut self.items, &self.config.size, self.config.theme)?;
        let is_main_menu = self.menu_type == MenuType::Main;

        let data = MenuData {
            index: self.menu.index,
            items: self.items.clone(),
            htheme: if is_main_menu { Some(unsafe { OpenThemeDataEx(self.menu.hwnd, w!("Menu"), OTD_NONCLIENT) }) } else { None },
            win_subclass_id: if is_main_menu { Some(COUNTER.fetch_add(1, Ordering::Relaxed)) } else { None },
            height: size.height,
            width: size.width,
            selected_index: -1,
            visible_submenu_index: -1,
            theme: self.config.theme,
            size: self.config.size.clone(),
            color: self.config.color.clone(),
        };

        if is_main_menu {
            self.menu
                .attach_owner_subclass(data.win_subclass_id.unwrap() as usize);
        }

        unsafe { SetWindowLongPtrW(self.menu.hwnd, GWL_USERDATA, Box::into_raw(Box::new(data)) as _) };

        Ok(self.menu)
    }
}
