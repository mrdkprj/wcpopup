use super::{
    from_menu_item, get_menu_item_data_mut, set_menu_item_data,
    style::{get_menu_item_css, get_widget_name},
    to_menu_item, Config, Menu, SubmenuData,
};
use crate::{InnerMenuEvent, MenuEvent, MenuItemType, MenuType};
use gtk::{
    ffi::{gtk_style_context_add_provider_for_screen, GtkStyleProvider},
    gdk::ffi::gdk_screen_get_default,
    glib::{translate::ToGlibPtr, Cast},
    prelude::{CheckMenuItemExt, ContainerExt, CssProviderExt, GtkMenuItemExt, RadioMenuItemExt, WidgetExt},
    CssProvider, StyleProvider, STYLE_PROVIDER_PRIORITY_APPLICATION,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::atomic::{AtomicU16, Ordering},
};

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
    pub(crate) gtk_menu_item: isize,
    pub(crate) items: Option<Vec<MenuItem>>,
    suppress_event: bool,
}

impl MenuItem {
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
        let gtk_menu_item = to_menu_item(self.gtk_menu_item);
        let menu_item = get_menu_item_data_mut(&gtk_menu_item);
        gtk_menu_item.set_sensitive(!disabled);
        menu_item.disabled = disabled;
        set_menu_item_data(&gtk_menu_item, menu_item);
    }

    pub fn set_label(&mut self, label: &str) {
        self.label = label.to_string();
        let gtk_menu_item = to_menu_item(self.gtk_menu_item);
        let menu_item = get_menu_item_data_mut(&gtk_menu_item);
        gtk_menu_item.set_label(label);
        menu_item.label = label.to_string();
        set_menu_item_data(&gtk_menu_item, menu_item);
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
            uuid: UUID.fetch_add(1, Ordering::Relaxed),
            gtk_menu_item: 0,
            checked: false,
            disabled: disabled.unwrap_or(false),
            items: None,
            suppress_event: false,
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
            uuid: UUID.fetch_add(1, Ordering::Relaxed),
            gtk_menu_item: 0,
            checked,
            disabled: disabled.unwrap_or(false),
            items: None,
            suppress_event: false,
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
            uuid: UUID.fetch_add(1, Ordering::Relaxed),
            gtk_menu_item: 0,
            checked,
            disabled: disabled.unwrap_or(false),
            items: None,
            suppress_event: false,
        }
    }

    pub fn set_checked(&mut self, checked: bool) {
        self.checked = checked;
        let gtk_menu_item = to_menu_item(self.gtk_menu_item);
        let menu_item = get_menu_item_data_mut(&gtk_menu_item);
        menu_item.checked = checked;
        menu_item.suppress_event = true;
        set_menu_item_data(&gtk_menu_item, menu_item);

        if self.menu_item_type == MenuItemType::Checkbox {
            gtk_menu_item.downcast_ref::<gtk::CheckMenuItem>().unwrap().set_active(checked);
        }
        if self.menu_item_type == MenuItemType::Radio {
            gtk_menu_item.downcast_ref::<gtk::RadioMenuItem>().unwrap().set_active(checked);
        }
    }
}

impl MenuItem {
    pub fn new_submenu_item(id: &str, label: &str, disabled: Option<bool>) -> Self {
        let menu = Menu {
            menu_type: MenuType::Submenu,
            ..Default::default()
        };
        Self {
            id: id.to_string(),
            label: label.to_string(),
            accelerator: String::new(),
            name: String::new(),
            menu_item_type: MenuItemType::Submenu,
            submenu: Some(menu),
            uuid: UUID.fetch_add(1, Ordering::Relaxed),
            gtk_menu_item: 0,
            checked: false,
            disabled: disabled.unwrap_or(false),
            items: Some(Vec::new()),
            suppress_event: false,
        }
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
            uuid: UUID.fetch_add(1, Ordering::Relaxed),
            gtk_menu_item: 0,
            checked: false,
            disabled: false,
            items: None,
            suppress_event: false,
        }
    }
}

pub(crate) fn radio_group_from_item(item: &MenuItem) -> HashMap<String, gtk::RadioMenuItem> {
    let gtk_radio_item = to_menu_item(item.gtk_menu_item).downcast::<gtk::RadioMenuItem>().unwrap();
    HashMap::from([(item.name.clone(), gtk_radio_item)])
}

pub(crate) fn create_gtk_menu_item(
    item: &mut MenuItem,
    submenudata_map: Option<&HashMap<u16, SubmenuData>>,
    radio_groups: Option<&mut HashMap<String, gtk::RadioMenuItem>>,
    config: &Config,
) -> gtk::MenuItem {
    let gtk_menu_item = match item.menu_item_type {
        MenuItemType::Text => {
            let gtk_menu_item = gtk::MenuItem::builder().label(item.label.as_str()).sensitive(!item.disabled).build();
            item.gtk_menu_item = from_menu_item(&gtk_menu_item);
            gtk_menu_item
        }
        MenuItemType::Checkbox => {
            let check_menu_item = gtk::CheckMenuItem::builder().label(item.label.as_str()).sensitive(!item.disabled).active(item.checked).build();
            let gtk_menu_item = check_menu_item.upcast::<gtk::MenuItem>();
            item.gtk_menu_item = from_menu_item(&gtk_menu_item);
            gtk_menu_item
        }
        MenuItemType::Radio => {
            let radio_menu_item = gtk::RadioMenuItem::builder().label(item.label.as_str()).draw_as_radio(false).sensitive(!item.disabled).active(item.checked).build();
            if let Some(radio_groups) = radio_groups {
                if radio_groups.contains_key(&item.name) {
                    let radio_group = radio_groups.get(&item.name).unwrap();
                    radio_menu_item.join_group(Some(radio_group));
                } else {
                    radio_groups.insert(item.name.clone(), radio_menu_item.clone());
                }
            }
            let gtk_menu_item = radio_menu_item.upcast::<gtk::MenuItem>();
            item.gtk_menu_item = from_menu_item(&gtk_menu_item);
            gtk_menu_item
        }
        MenuItemType::Separator => {
            let gtk_menu_item = gtk::SeparatorMenuItem::new().upcast::<gtk::MenuItem>();
            item.gtk_menu_item = from_menu_item(&gtk_menu_item);
            gtk_menu_item
        }
        MenuItemType::Submenu => {
            let submenudata_map = submenudata_map.unwrap();
            let submedata = submenudata_map.get(&item.uuid).unwrap();
            if submedata.gtk_submenu.children().is_empty() {
                submedata.gtk_submenu.set_sensitive(false);
            }
            let gtk_menu_item = gtk::MenuItem::builder().label(&item.label).submenu(&submedata.gtk_submenu).sensitive(!item.disabled).build();
            item.gtk_menu_item = from_menu_item(&gtk_menu_item);
            item.submenu = Some(submedata.submenu.clone());

            gtk_menu_item
        }
    };

    gtk_menu_item.connect_activate(move |selected_gtk_menu_item| {
        let menu_item = get_menu_item_data_mut(selected_gtk_menu_item);

        if should_send(selected_gtk_menu_item, menu_item) && selected_gtk_menu_item.get_sensitive() {
            MenuEvent::send(MenuEvent {
                item: menu_item.clone(),
            });
            MenuEvent::send_inner(InnerMenuEvent {
                item: Some(menu_item.clone()),
            });
            set_menu_item_data(selected_gtk_menu_item, menu_item);
        }
    });

    let widget_name = get_widget_name(config.theme);
    gtk_menu_item.set_widget_name(widget_name);

    let css = get_menu_item_css(config);
    let css_provider = CssProvider::new();
    css_provider.load_from_data(css.as_bytes()).unwrap();
    let provider = css_provider.dynamic_cast::<StyleProvider>().unwrap();
    let provider_ptr: *mut GtkStyleProvider = provider.to_glib_none().0;
    unsafe { gtk_style_context_add_provider_for_screen(gdk_screen_get_default(), provider_ptr, STYLE_PROVIDER_PRIORITY_APPLICATION) };

    set_menu_item_data(&gtk_menu_item, item);

    gtk_menu_item.show();

    gtk_menu_item
}

fn should_send(gtk_menu_item: &gtk::MenuItem, item: &mut MenuItem) -> bool {
    match item.menu_item_type {
        MenuItemType::Checkbox => {
            item.checked = gtk_menu_item.downcast_ref::<gtk::CheckMenuItem>().unwrap().is_active();
            if item.suppress_event {
                item.suppress_event = false;
                false
            } else {
                true
            }
        }
        MenuItemType::Radio => {
            item.checked = gtk_menu_item.downcast_ref::<gtk::RadioMenuItem>().unwrap().is_active();
            if item.suppress_event {
                item.suppress_event = false;
                false
            } else {
                item.checked
            }
        }
        MenuItemType::Submenu => false,
        _ => true,
    }
}
