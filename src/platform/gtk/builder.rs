use super::{accelerator::setup_accel_group, create_gtk_menu_item, from_accel_group, from_gtk_menu, to_gtk_menu, to_gtk_window, toggle_menu_item_icons, Container};
use crate::{
    config::{Config, IconSettings, Theme},
    Menu, MenuIcon, MenuIconKind, MenuItem, MenuItemType, MenuType,
};
use gtk::{
    glib::{Error, IsA, ObjectExt},
    prelude::{GtkWindowExt, MenuShellExt},
    traits::{ContainerExt, WidgetExt},
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub(crate) struct MenuData {
    pub(crate) config: Config,
    pub(crate) accel_group_handle: Option<isize>,
    pub(crate) visible: bool,
    pub(crate) parent_gtk_menu_handle: isize,
    pub(crate) has_custom_check_image: bool,
}

#[derive(Debug)]
/// Builder to create Menu.
pub struct MenuBuilder {
    menu: Menu,
    gtk_menu: gtk::Menu,
    items: Vec<MenuItem>,
    theme: Theme,
    config: Config,
    radio_groups: HashMap<String, gtk::RadioMenuItem>,
}

#[derive(Debug)]
pub(crate) struct SubmenuData {
    pub(crate) gtk_submenu: isize,
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
        let (menu, gtk_menu) = Menu::new(Container::Window(window.as_ref()), &config);

        Self {
            menu,
            config,
            theme,
            items: Vec::new(),
            gtk_menu,
            radio_groups: HashMap::new(),
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
        let (menu, gtk_menu) = Menu::new(super::Container::Window(window.as_ref()), &config);
        Self {
            menu,
            config,
            theme,
            items: Vec::new(),
            gtk_menu,
            radio_groups: HashMap::new(),
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
        let (menu, gtk_menu) = Menu::new(Container::Window(window.as_ref()), &config);
        Self {
            menu,
            config: Config {
                icon: if let Some(icon) = config.icon {
                    Some(icon)
                } else {
                    Some(IconSettings::default())
                },
                ..config
            },
            theme,
            items: Vec::new(),
            gtk_menu,
            radio_groups: HashMap::new(),
        }
    }

    /// Adds a text MenuItem to Menu.
    pub fn text(&mut self, id: &str, label: &str, disabled: bool) -> &Self {
        let mut item = MenuItem::new_text_item(id, label, None, disabled, None);
        self.create_item(&mut item);
        self.items.push(item);
        self
    }

    pub fn text_with_accelerator(&mut self, id: &str, label: &str, disabled: bool, accelerator: &str) -> &Self {
        let mut item = MenuItem::new_text_item(id, label, Some(accelerator), disabled, None);
        self.create_item(&mut item);
        self.items.push(item);
        self
    }

    pub fn text_with_icon(&mut self, id: &str, label: &str, disabled: bool, icon: MenuIcon) -> &Self {
        let mut item = MenuItem::new_text_item(id, label, None, disabled, Some(icon));
        self.create_item(&mut item);
        self.items.push(item);
        self
    }

    pub fn text_with_accel_icon(&mut self, id: &str, label: &str, disabled: bool, accelerator: &str, icon: MenuIcon) -> &Self {
        let mut item = MenuItem::new_text_item(id, label, Some(accelerator), disabled, Some(icon));
        self.create_item(&mut item);
        self.items.push(item);
        self
    }

    /// Adds a check MenuItem to Menu.
    pub fn check(&mut self, id: &str, label: &str, checked: bool, disabled: bool) -> &Self {
        let mut item = MenuItem::new_check_item(id, label, None, checked, disabled, None);
        self.create_item(&mut item);
        self.items.push(item);
        self
    }

    pub fn check_with_accelerator(&mut self, id: &str, label: &str, checked: bool, disabled: bool, accelerator: &str) -> &Self {
        let mut item = MenuItem::new_check_item(id, label, Some(accelerator), checked, disabled, None);
        self.create_item(&mut item);
        self.items.push(item);
        self
    }

    pub fn check_with_icon(&mut self, id: &str, label: &str, checked: bool, disabled: bool, icon: MenuIcon) -> &Self {
        let mut item = MenuItem::new_check_item(id, label, None, checked, disabled, Some(icon));
        self.create_item(&mut item);
        self.items.push(item);
        self
    }

    pub fn check_with_accel_icon(&mut self, id: &str, label: &str, checked: bool, disabled: bool, accelerator: &str, icon: MenuIcon) -> &Self {
        let mut item = MenuItem::new_check_item(id, label, Some(accelerator), checked, disabled, Some(icon));
        self.create_item(&mut item);
        self.items.push(item);
        self
    }

    /// Adds a radio MenuItem to Menu.
    pub fn radio(&mut self, id: &str, label: &str, name: &str, checked: bool, disabled: bool) -> &Self {
        let mut item = MenuItem::new_radio_item(id, label, name, None, checked, disabled, None);
        self.create_item(&mut item);
        self.items.push(item);
        self
    }

    pub fn radio_with_accelerator(&mut self, id: &str, label: &str, name: &str, checked: bool, disabled: bool, accelerator: &str) -> &Self {
        let mut item = MenuItem::new_radio_item(id, label, name, Some(accelerator), checked, disabled, None);
        self.create_item(&mut item);
        self.items.push(item);
        self
    }

    pub fn radio_with_icon(&mut self, id: &str, label: &str, name: &str, checked: bool, disabled: bool, icon: MenuIcon) -> &Self {
        let mut item = MenuItem::new_radio_item(id, label, name, None, checked, disabled, Some(icon));
        self.create_item(&mut item);
        self.items.push(item);
        self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn radio_with_accel_icon(&mut self, id: &str, label: &str, name: &str, checked: bool, disabled: bool, accelerator: &str, icon: MenuIcon) -> &Self {
        let mut item = MenuItem::new_radio_item(id, label, name, Some(accelerator), checked, disabled, Some(icon));
        self.create_item(&mut item);
        self.items.push(item);
        self
    }

    /// Adds a separator to Menu.
    pub fn separator(&mut self) -> &Self {
        let mut item = MenuItem::new_separator();
        self.create_item(&mut item);
        self.items.push(item);
        self
    }

    pub fn separator_with_id(&mut self, id: &str) -> &Self {
        let mut item = MenuItem::new_separator_with_id(id);
        self.create_item(&mut item);
        self.items.push(item);
        self
    }

    fn create_item(&mut self, item: &mut MenuItem) {
        let gtk_menu_item = create_gtk_menu_item(self.menu.gtk_menu_handle, item, None, Some(&mut self.radio_groups), &self.config);
        self.gtk_menu.append(&gtk_menu_item);
    }

    fn create_submenu_item(&mut self, item: &mut MenuItem, submenu_data: SubmenuData) {
        let gtk_menu_item = create_gtk_menu_item(self.menu.gtk_menu_handle, item, Some(submenu_data), Some(&mut self.radio_groups), &self.config);
        self.gtk_menu.append(&gtk_menu_item);
    }

    /// Adds a submenu MenuItem to Menu.
    pub fn submenu(&mut self, id: &str, label: &str, disabled: bool) -> Self {
        let (menu, gtk_menu) = Menu::new(Container::Menu(&self.menu), &self.config);
        let mut item = MenuItem::new_submenu_item(id, label, disabled, None);

        let submenu_data = SubmenuData {
            gtk_submenu: from_gtk_menu(&gtk_menu),
            submenu: menu.clone(),
        };
        self.create_submenu_item(&mut item, submenu_data);

        let builder = MenuBuilder {
            menu,
            theme: self.theme,
            config: self.config.clone(),
            items: Vec::new(),
            gtk_menu,
            radio_groups: HashMap::new(),
        };

        self.items.push(item);
        builder
    }

    pub fn submenu_with_icon(&mut self, id: &str, label: &str, disabled: bool, icon: MenuIcon) -> Self {
        let (menu, gtk_menu) = Menu::new(Container::Menu(&self.menu), &self.config);
        let mut item = MenuItem::new_submenu_item(id, label, disabled, Some(icon));

        let submenu_data = SubmenuData {
            gtk_submenu: from_gtk_menu(&gtk_menu),
            submenu: menu.clone(),
        };
        self.create_submenu_item(&mut item, submenu_data);

        let builder = MenuBuilder {
            menu,
            theme: self.theme,
            config: self.config.clone(),
            items: Vec::new(),
            gtk_menu,
            radio_groups: HashMap::new(),
        };

        self.items.push(item);
        builder
    }

    pub(crate) fn new_submenu_with_items(parent: &Menu, item: &mut MenuItem, config: &Config) -> gtk::MenuItem {
        let (menu, gtk_menu) = Menu::new(Container::Menu(parent), config);
        /* First create submenu item */
        let submedata = SubmenuData {
            gtk_submenu: from_gtk_menu(&gtk_menu),
            submenu: menu.clone(),
        };
        /* Don't append */
        let gtk_submenu_item = create_gtk_menu_item(parent.gtk_menu_handle, item, Some(submedata), None, config);

        /* Then append items to the submenu */
        let mut radio_groups = HashMap::new();
        for menu_item in item.items.as_mut().unwrap().iter_mut() {
            let gtk_menu_item = create_gtk_menu_item(menu.gtk_menu_handle, menu_item, None, Some(&mut radio_groups), config);
            gtk_menu.append(&gtk_menu_item);
        }

        /* Build for data setup */
        let builder = MenuBuilder {
            menu,
            theme: config.theme,
            config: config.clone(),
            items: Vec::new(),
            gtk_menu,
            radio_groups,
        };
        /* Safe to unwrap, because Result is for compatibility with Windows */
        builder.build().unwrap();

        gtk_submenu_item
    }

    /// Adds a MenuItem to MenuBuilder.
    pub fn append(&mut self, mut menu_item: MenuItem) -> &Self {
        if menu_item.menu_item_type == MenuItemType::Submenu {
            let gtk_menu_item = Self::new_submenu_with_items(&self.menu, &mut menu_item, &self.config);
            self.gtk_menu.append(&gtk_menu_item);
        } else {
            self.create_item(&mut menu_item);
        }
        self.items.push(menu_item);
        self
    }

    /// Adds MenuItems to MenuBuilder.
    pub fn append_all(&mut self, menu_items: Vec<MenuItem>) -> &Self {
        for menu_item in menu_items {
            self.append(menu_item);
        }
        self
    }

    /// Build Menu to make it ready to become visible.
    /// Must call this function before showing Menu, otherwise nothing shows up.
    pub fn build(self) -> Result<Menu, Error> {
        let gtk_menu = to_gtk_menu(self.menu.gtk_menu_handle);

        let is_main_menu = self.menu.menu_type == MenuType::Main;

        if !is_main_menu && gtk_menu.children().is_empty() {
            gtk_menu.set_sensitive(false);
        }

        /* Add accel_group after gtk::MenuItem is created */
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

        /* Path icon does not require gtk::Image for check */
        let has_custom_check_image = if let Some(check) = &self.config.icon.as_ref().unwrap().check {
            !matches!(&check.icon, MenuIconKind::Path(_))
        } else {
            false
        };

        let data = MenuData {
            config: self.config,
            accel_group_handle,
            visible: false,
            parent_gtk_menu_handle: self.menu.parent_gtk_menu_handle,
            has_custom_check_image,
        };

        unsafe { gtk_menu.set_data("data", data) };

        toggle_menu_item_icons(self.menu.gtk_menu_handle);

        Ok(self.menu)
    }
}

fn collect_accelerators(items: &Vec<MenuItem>, accelerators: &mut HashMap<isize, String>) {
    for item in items {
        if item.menu_item_type == MenuItemType::Submenu {
            let submenu = item.submenu.as_ref().unwrap();
            collect_accelerators(&submenu.items(), accelerators);
        } else if !item.accelerator.is_empty() {
            accelerators.insert(item.gtk_menu_item_handle, item.accelerator.clone());
        }
    }
}
