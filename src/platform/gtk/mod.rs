use async_std::future::timeout;
use gtk::{
    gdk,
    glib::{monotonic_time, translate::ToGlibPtr, Cast, ObjectExt},
    prelude::{ContainerExt, CssProviderExt, GtkMenuExt, GtkMenuItemExt, GtkSettingsExt, MenuShellExt, SeatExt, StyleContextExt, WidgetExt},
    CssProvider, Widget, STYLE_PROVIDER_PRIORITY_APPLICATION,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
mod accelerator;
mod builder;
mod menu_item;
mod style;
mod util;
use crate::{config::*, MenuEvent};
use crate::{InnerMenuEvent, MenuType};
use crate::{MenuItemType, ThemeChangeFactor};
use accelerator::*;
pub use builder::*;
pub use menu_item::*;
use style::*;
use util::*;

pub(crate) enum Container<'a> {
    Window(&'a gtk::Window),
    Menu(&'a Menu),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Menu {
    pub gtk_menu_handle: isize,
    pub menu_type: MenuType,
    parent_gtk_menu_handle: isize,
    gtk_window_handle: isize,
}

impl Default for Menu {
    fn default() -> Self {
        Self {
            gtk_menu_handle: 0,
            parent_gtk_menu_handle: 0,
            gtk_window_handle: 0,
            menu_type: MenuType::Main,
        }
    }
}

impl Menu {
    pub(crate) fn new(parent: Container, config: &Config) -> (Self, gtk::Menu) {
        let widget_name = get_widget_name(config.theme);
        let gtk_menu = gtk::Menu::new();

        if let Some(menu_conainer_widget) = gtk_menu.parent() {
            // Make window borderless
            let gtk_window = menu_conainer_widget.dynamic_cast::<gtk::Window>().unwrap();
            gtk_window.set_widget_name(widget_name);
            let provider = CssProvider::new();
            let css = get_window_css(config);
            provider.load_from_data(css.as_bytes()).unwrap();
            gtk_window.style_context().add_provider(&provider, STYLE_PROVIDER_PRIORITY_APPLICATION);
        }

        let (parent_gtk_menu_handle, gtk_window_handle, menu_type) = match parent {
            Container::Window(gtk_window) => {
                gtk_menu.set_attach_widget(Some(gtk_window));
                let gtk_window_handle = from_gtk_window(gtk_window);
                let gtk_menu_handle = from_menu(&gtk_menu);

                if let Some(settings) = gtk_window.settings() {
                    settings.connect_gtk_application_prefer_dark_theme_notify(move |changed_settings| {
                        let theme = if changed_settings.is_gtk_application_prefer_dark_theme() {
                            Theme::Dark
                        } else {
                            Theme::Light
                        };
                        on_theme_change(MenuType::Main, gtk_menu_handle, Some(theme), ThemeChangeFactor::App);
                    });
                    settings.connect_gtk_theme_name_notify(move |_| {
                        on_theme_change(MenuType::Main, gtk_menu_handle, None, ThemeChangeFactor::App);
                    });
                }
                (0, gtk_window_handle, MenuType::Main)
            }
            Container::Menu(menu) => (menu.gtk_menu_handle, menu.gtk_window_handle, MenuType::Submenu),
        };

        gtk_menu.set_border_width(config.size.border_size as u32);

        gtk_menu.set_widget_name(widget_name);

        let css = get_menu_css(config);
        let provider = CssProvider::new();
        provider.load_from_data(css.as_bytes()).unwrap();
        gtk_menu.style_context().add_provider(&provider, STYLE_PROVIDER_PRIORITY_APPLICATION);

        gtk_menu.show();

        let menu = Self {
            gtk_menu_handle: from_menu(&gtk_menu),
            parent_gtk_menu_handle,
            gtk_window_handle,
            menu_type,
        };

        (menu, gtk_menu)
    }

    pub fn config(&self) -> Config {
        get_menu_data(self.gtk_menu_handle).config.clone()
    }

    pub fn theme(&self) -> Theme {
        let data = get_menu_data(self.gtk_menu_handle);
        data.config.theme
    }

    /// Sets the theme for Menu.
    pub fn set_theme(&self, theme: Theme) {
        on_theme_change(self.menu_type, self.gtk_menu_handle, Some(theme), ThemeChangeFactor::User);
    }

    /// Gets all MenuItems of Menu.
    pub fn items(&self) -> Vec<MenuItem> {
        let gtk_menu = to_menu(self.gtk_menu_handle);
        gtk_menu.children().iter().map(|item| get_menu_item_data(item).clone()).collect()
    }

    /// Gets the MenuItem with the specified id.
    pub fn get_menu_item_by_id(&self, id: &str) -> Option<MenuItem> {
        let gtk_menu = to_menu(self.gtk_menu_handle);
        find_by_id(&gtk_menu.children(), id)
    }

    /// Adds a MenuItem to the end of MenuItems.
    pub fn append(&mut self, item: MenuItem) {
        let mut item = item.clone();

        let gtk_menu = to_menu(self.gtk_menu_handle);
        let config = &get_menu_data(self.gtk_menu_handle).config;

        let gtk_menu_item = self.create_gtk_menu(&mut item, config);
        gtk_menu.append(&gtk_menu_item);

        Self::reset_haccel(self, &item);

        if !gtk_menu.children().is_empty() {
            gtk_menu.set_sensitive(true);
        }
    }

    /// Adds a MenuItem at the specified index.
    pub fn insert(&mut self, item: MenuItem, index: u32) {
        let mut item = item.clone();

        let gtk_menu = to_menu(self.gtk_menu_handle);
        let config = &get_menu_data(self.gtk_menu_handle).config;

        let gtk_menu_item = self.create_gtk_menu(&mut item, config);
        gtk_menu.insert(&gtk_menu_item, index as i32);

        Self::reset_haccel(self, &item);

        if !gtk_menu.children().is_empty() {
            gtk_menu.set_sensitive(true);
        }
    }

    fn create_gtk_menu(&mut self, item: &mut MenuItem, config: &Config) -> gtk::MenuItem {
        match item.menu_item_type {
            MenuItemType::Submenu => self.create_submenu(item, config),
            MenuItemType::Radio => {
                if let Some(radio) = self.find_first_radio(&item.name) {
                    let mut group = radio_group_from_item(&radio);
                    create_gtk_menu_item(item, None, Some(&mut group), config)
                } else {
                    create_gtk_menu_item(item, None, None, config)
                }
            }
            _ => create_gtk_menu_item(item, None, None, config),
        }
    }

    fn create_submenu(&mut self, item: &mut MenuItem, config: &Config) -> gtk::MenuItem {
        let mut builder = MenuBuilder::new_for_submenu(self, item, config);
        let submenu = builder.build().unwrap();
        let gtk_menu = create_gtk_menu_item(item, Some(&builder.gtk_submenu), None, config);
        item.submenu = Some(submenu);
        gtk_menu
    }

    fn find_first_radio(&self, name: &str) -> Option<MenuItem> {
        self.items().iter().find(|item| item.name == *name).cloned()
    }

    /// Removes the MenuItem at the specified index.
    pub fn remove_at(&mut self, index: u32) {
        let gtk_menu = to_menu(self.gtk_menu_handle);
        if let Some(remove_item) = gtk_menu.children().get(index as usize) {
            gtk_menu.remove(remove_item);

            if gtk_menu.children().is_empty() {
                gtk_menu.set_sensitive(false);
            }
        }
    }

    /// Removes the MenuItem.
    pub fn remove(&mut self, item: &MenuItem) {
        let gtk_menu = to_menu(self.gtk_menu_handle);
        let maybe_index = index_of_item(&gtk_menu.children(), item.uuid);

        if let Some(index) = maybe_index {
            self.remove_at(index as u32);
        }
    }

    fn reset_haccel(&self, item: &MenuItem) {
        let gtk_menu_handle = if self.menu_type == MenuType::Main {
            self.gtk_menu_handle
        } else {
            self.parent_gtk_menu_handle
        };

        add_accelerators_from_menu_item(gtk_menu_handle, item);
    }

    /// Shows Menu at the specified point.
    pub fn popup_at(&self, x: i32, y: i32) {
        let gtk_window = to_gtk_window(self.gtk_window_handle);
        let gtk_menu = to_menu(self.gtk_menu_handle);

        let mut event = gdk::Event::new(gdk::EventType::ButtonPress);
        event.set_device(gtk_window.display().default_seat().and_then(|d| d.pointer()).as_ref());

        let window = gtk_window.window().unwrap();

        let event_ffi: *mut gdk::ffi::GdkEvent = event.to_glib_none().0;
        if !event_ffi.is_null() {
            let time = monotonic_time() / 1000;
            unsafe {
                (*event_ffi).button.time = time as _;
            }
        }

        #[cfg(feature = "accelerator")]
        connect_accelerator(&gtk_menu, self.gtk_menu_handle, self.gtk_window_handle);

        gtk_menu.popup_at_rect(&window, &gdk::Rectangle::new(x, y, 0, 0), gdk::Gravity::NorthWest, gdk::Gravity::NorthWest, Some(&event));
    }

    /// Shows Menu asynchronously at the specified point and returns the selected MenuItem if any.
    pub async fn popup_at_async(&self, x: i32, y: i32) -> Option<MenuItem> {
        let gtk_window = to_gtk_window(self.gtk_window_handle);
        let gtk_menu = to_menu(self.gtk_menu_handle);

        let mut event = gdk::Event::new(gdk::EventType::ButtonPress);
        event.set_device(gtk_window.display().default_seat().and_then(|d| d.pointer()).as_ref());

        let window = gtk_window.window().unwrap();

        let event_ffi: *mut gdk::ffi::GdkEvent = event.to_glib_none().0;
        if !event_ffi.is_null() {
            let time = monotonic_time() / 1000;
            unsafe {
                (*event_ffi).button.time = time as _;
            }
        }

        #[cfg(feature = "accelerator")]
        connect_accelerator(&gtk_menu, self.gtk_menu_handle, self.gtk_window_handle);

        gtk_menu.popup_at_rect(&window, &gdk::Rectangle::new(x, y, 0, 0), gdk::Gravity::NorthWest, gdk::Gravity::NorthWest, Some(&event));

        let mut item = None;

        let signal = gtk_menu.connect_hide(move |_| {
            MenuEvent::send_inner(InnerMenuEvent {
                item: None,
            });
        });

        if let Ok(event) = MenuEvent::innner_receiver().recv().await {
            item = event.item;
        }

        gtk_menu.disconnect(signal);

        // Wait 50 ms for "activate" event.
        // Click "activate" occurs after automatic menu "hide", so event can have menu item.
        // Keypress "activate" occurs before menu "hide", so event is none that should be dismissed.
        if let Ok(Ok(event)) = timeout(Duration::from_millis(50), MenuEvent::innner_receiver().recv()).await {
            if event.item.is_some() {
                item = event.item;
            }
        }

        item
    }
}

fn find_by_id(items: &Vec<Widget>, id: &str) -> Option<MenuItem> {
    let item_id = id.to_string();
    for item in items {
        let menu_item = get_menu_item_data(item);
        if menu_item.id == item_id {
            return Some(menu_item.clone());
        }

        if menu_item.menu_item_type == MenuItemType::Submenu {
            let submenu = to_menu(menu_item.submenu.as_ref().unwrap().gtk_menu_handle);
            if let Some(menu_item) = find_by_id(&submenu.children(), id) {
                return Some(menu_item);
            }
        }
    }
    None
}

fn index_of_item(items: &[Widget], uuid: u16) -> Option<usize> {
    for (index, item) in items.iter().enumerate() {
        let menu_item = get_menu_item_data(item);
        if menu_item.uuid == uuid {
            return Some(index);
        }

        if menu_item.menu_item_type == MenuItemType::Submenu {
            let submenu = to_menu(menu_item.submenu.as_ref().unwrap().gtk_menu_handle);
            if let Some(index) = index_of_item(&submenu.children(), uuid) {
                return Some(index);
            }
        }
    }
    None
}

fn on_theme_change(menu_type: MenuType, gtk_menu_handle: isize, maybe_preferred_theme: Option<Theme>, factor: ThemeChangeFactor) {
    let data = get_menu_data_mut(gtk_menu_handle);

    if menu_type == MenuType::Submenu {
        return;
    }

    let current_them = data.config.theme;

    // Don't respont to setting change event unless theme is System
    if current_them != Theme::System && factor == ThemeChangeFactor::SystemSetting {
        return;
    }

    let should_be_dark = match factor {
        ThemeChangeFactor::User => {
            let preferred_theme = maybe_preferred_theme.unwrap();
            if preferred_theme == Theme::System {
                is_sys_dark()
            } else {
                preferred_theme == Theme::Dark
            }
        }
        ThemeChangeFactor::App => {
            if let Some(preferred_theme) = maybe_preferred_theme {
                preferred_theme == Theme::Dark
            } else {
                is_sys_dark()
            }
        }
        ThemeChangeFactor::SystemSetting => false,
    };

    let new_theme = match maybe_preferred_theme {
        Some(preferred_theme) => preferred_theme,
        None => {
            if current_them == Theme::System {
                current_them
            } else if should_be_dark {
                Theme::Dark
            } else {
                Theme::Light
            }
        }
    };

    data.config.theme = new_theme;

    let widget_name = get_widget_name(new_theme);

    let gtk_menu = to_menu(gtk_menu_handle);
    if let Some(menu_conainer_widget) = gtk_menu.parent() {
        let gtk_window = menu_conainer_widget.dynamic_cast::<gtk::Window>().unwrap();
        gtk_window.set_widget_name(widget_name);
    }
    gtk_menu.set_widget_name(widget_name);

    change_style(&gtk_menu.children(), new_theme, widget_name);
    set_menu_data(gtk_menu_handle, data);
}

fn change_style(items: &Vec<Widget>, new_theme: Theme, widget_name: &str) {
    for gtk_menu_item in items {
        gtk_menu_item.set_widget_name(widget_name);

        if let Some(submenu_widget) = gtk_menu_item.downcast_ref::<gtk::MenuItem>().unwrap().submenu() {
            let submenu = submenu_widget.downcast::<gtk::Menu>().unwrap();
            let submenu_handle = from_menu(&submenu);
            let submenu_data = get_menu_data_mut(submenu_handle);
            submenu_data.config.theme = new_theme;
            if let Some(menu_conainer_widget) = submenu.parent() {
                let gtk_window = menu_conainer_widget.dynamic_cast::<gtk::Window>().unwrap();
                gtk_window.set_widget_name(widget_name);
            }
            submenu.set_widget_name(widget_name);
            change_style(&submenu.children(), new_theme, widget_name);
            set_menu_data(submenu_handle, submenu_data);
        }
    }
}
