use super::{accelerator::setup_accel_group, create_gtk_menu_item, from_accel_group, from_menu, to_gtk_window, to_menu, Config, Menu, MenuItem, MenuType, Theme};
use crate::MenuItemType;
use gtk::{
    glib::{Error, IsA, ObjectExt},
    prelude::{GtkWindowExt, MenuShellExt},
};
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct MenuData {
    pub(crate) gtk_menu_handle: isize,
    pub(crate) config: Config,
    pub(crate) accel_group_handle: Option<isize>,
}

#[derive(Debug, Clone)]
/// Builder to create Menu.
pub struct MenuBuilder {
    pub(crate) menu: Menu,
    pub(crate) gtk_submenu: HashMap<u16, SubmenuData>,
    theme: Theme,
    config: Config,
    items: Vec<MenuItem>,
}

#[derive(Debug, Clone)]
pub(crate) struct SubmenuData {
    pub(crate) gtk_submenu: gtk::Menu,
    pub(crate) submenu: Menu,
}

impl MenuBuilder {
    /// Creates a new Menu for the specified window handle.
    pub fn new(window_handle: isize) -> Self {
        let gtk_window = to_gtk_window(window_handle);
        Self::new_builder(&gtk_window)
    }

    /// Creates a new Menu for the specified Window.
    pub fn new_for_window(window: &impl IsA<gtk::Window>) -> Self {
        Self::new_builder(window)
    }

    fn new_builder(window: &impl IsA<gtk::Window>) -> Self {
        let config = Config::default();
        let theme = config.theme;
        let (menu, _) = Menu::new(super::Container::Window(window.as_ref()), &config);
        Self {
            menu,
            config,
            theme,
            gtk_submenu: HashMap::new(),
            items: Vec::new(),
        }
    }

    /// Creates a new Menu with the specified Theme for the specified window handle.
    pub fn new_with_theme(window_handle: isize, theme: Theme) -> Self {
        let gtk_window = to_gtk_window(window_handle);
        Self::new_builder_with_theme(&gtk_window, theme)
    }

    /// Creates a new Menu with the specified Theme for the specified Window.
    pub fn new_for_window_with_theme(window: &impl IsA<gtk::Window>, theme: Theme) -> Self {
        Self::new_builder_with_theme(window, theme)
    }

    fn new_builder_with_theme(window: &impl IsA<gtk::Window>, theme: Theme) -> Self {
        let config = Config {
            theme,
            ..Default::default()
        };
        let theme = config.theme;
        let (menu, _) = Menu::new(super::Container::Window(window.as_ref()), &config);
        Self {
            menu,
            config,
            theme,
            gtk_submenu: HashMap::new(),
            items: Vec::new(),
        }
    }

    /// Creates a new Menu using the specified Config for the specified window handle.
    pub fn new_from_config(window_handle: isize, config: Config) -> Self {
        let gtk_window = to_gtk_window(window_handle);
        Self::new_builder_from_config(&gtk_window, config)
    }

    /// Creates a new Menu using the specified Config for the specified Window.
    pub fn new_for_window_from_config(window: &impl IsA<gtk::Window>, config: Config) -> Self {
        Self::new_builder_from_config(window, config)
    }

    fn new_builder_from_config(window: &impl IsA<gtk::Window>, config: Config) -> Self {
        let theme = config.theme;
        let (menu, _) = Menu::new(super::Container::Window(window.as_ref()), &config);
        Self {
            menu,
            config,
            theme,
            gtk_submenu: HashMap::new(),
            items: Vec::new(),
        }
    }

    /// Adds a text MenuItem to Menu.
    pub fn text(&mut self, id: &str, label: &str, disabled: Option<bool>) -> &Self {
        let item = MenuItem::new_text_item(id, label, None, disabled);
        self.items.push(item);
        self
    }

    pub fn text_with_accelerator(&mut self, id: &str, label: &str, disabled: Option<bool>, accelerator: &str) -> &Self {
        let item = MenuItem::new_text_item(id, label, Some(accelerator), disabled);
        self.items.push(item);
        self
    }

    /// Adds a check MenuItem to Menu.
    pub fn check(&mut self, id: &str, label: &str, checked: bool, disabled: Option<bool>) -> &Self {
        let item = MenuItem::new_check_item(id, label, None, checked, disabled);
        self.items.push(item);
        self
    }

    pub fn check_with_accelerator(&mut self, id: &str, label: &str, checked: bool, disabled: Option<bool>, accelerator: &str) -> &Self {
        let item = MenuItem::new_check_item(id, label, Some(accelerator), checked, disabled);
        self.items.push(item);
        self
    }

    /// Adds a radio MenuItem to Menu.
    pub fn radio(&mut self, id: &str, label: &str, name: &str, checked: bool, disabled: Option<bool>) -> &Self {
        let item = MenuItem::new_radio_item(id, label, name, None, checked, disabled);
        self.items.push(item);
        self
    }

    pub fn radio_with_accelerator(&mut self, id: &str, label: &str, name: &str, checked: bool, disabled: Option<bool>, accelerator: &str) -> &Self {
        let item = MenuItem::new_radio_item(id, label, name, Some(accelerator), checked, disabled);
        self.items.push(item);
        self
    }

    /// Adds a separator to Menu.
    pub fn separator(&mut self) -> &Self {
        let item = MenuItem::new_separator();
        self.items.push(item);
        self
    }

    pub(crate) fn new_for_submenu(parent: &Menu, item: &MenuItem, config: &Config) -> Self {
        let (menu, gtk_menu) = Menu::new(super::Container::Menu(parent), config);

        let theme = config.theme;
        Self {
            menu: menu.clone(),
            theme,
            config: config.clone(),
            gtk_submenu: HashMap::from([(
                item.uuid,
                SubmenuData {
                    gtk_submenu: gtk_menu,
                    submenu: menu.clone(),
                },
            )]),
            items: item.clone().items.unwrap(),
        }
    }

    /// Adds a submenu MenuItem to Menu.
    pub fn submenu(&mut self, id: &str, label: &str, disabled: Option<bool>) -> Self {
        let (menu, gtk_menu) = Menu::new(super::Container::Menu(&self.menu), &self.config);
        let item = MenuItem::new_submenu_item(id, label, disabled);

        let submenu_data = SubmenuData {
            gtk_submenu: gtk_menu,
            submenu: menu.clone(),
        };
        self.gtk_submenu.insert(item.uuid, submenu_data);

        let builder = MenuBuilder {
            menu,
            theme: self.theme,
            config: self.config.clone(),
            gtk_submenu: HashMap::new(),
            items: Vec::new(),
        };

        self.items.push(item);
        builder
    }

    /// Build Menu to make it ready to become visible.
    /// Must call this function before showing Menu, otherwise nothing shows up.
    pub fn build(&mut self) -> Result<Menu, Error> {
        let gtk_menu = to_menu(self.menu.gtk_menu_handle);

        let is_main_menu = self.menu.menu_type == MenuType::Main;

        let mut radio_groups: HashMap<String, gtk::RadioMenuItem> = HashMap::new();

        for item in self.items.iter_mut() {
            let gtk_menu_item = create_gtk_menu_item(item, Some(&self.gtk_submenu), Some(&mut radio_groups), &self.config);
            gtk_menu.append(&gtk_menu_item);
        }

        let mut accel_group_handle = None;
        let mut accelerators = HashMap::new();
        if is_main_menu {
            collect_accelerators(&self.items, &mut accelerators);
            if !accelerators.is_empty() {
                let gtk_window = to_gtk_window(self.menu.gtk_window_handle);
                let accel_group = setup_accel_group(&accelerators);
                gtk_window.add_accel_group(&accel_group);
                accel_group_handle = Some(from_accel_group(&accel_group));
            }
        }

        let data = MenuData {
            gtk_menu_handle: from_menu(&gtk_menu),
            config: self.config.clone(),
            accel_group_handle,
        };

        unsafe { gtk_menu.set_data("data", data) };

        Ok(self.menu.clone())
    }
}

fn collect_accelerators(items: &Vec<MenuItem>, accelerators: &mut HashMap<isize, String>) {
    for item in items {
        if item.menu_item_type == MenuItemType::Submenu {
            let submenu = item.submenu.as_ref().unwrap();
            collect_accelerators(&submenu.items(), accelerators);
        } else if !item.accelerator.is_empty() {
            accelerators.insert(item.gtk_menu_item, item.accelerator.clone());
        }
    }
}
