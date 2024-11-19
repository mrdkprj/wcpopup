use super::{
    collect_menu_items, from_gtk_menu_item, get_icon_menu_css, get_menu_data, get_menu_item_data_mut, set_menu_item_data,
    style::{get_menu_item_css, get_widget_name},
    to_gtk_menu_item, Config, Menu, SubmenuData,
};
use crate::{
    platform::platform_impl::{to_font_weight, to_font_weight_string},
    InnerMenuEvent, MenuEvent, MenuItemType,
};
use gtk::{
    ffi::{gtk_style_context_add_provider_for_screen, GtkStyleProvider},
    gdk::ffi::gdk_screen_get_default,
    glib::{translate::ToGlibPtr, Cast, IsA},
    pango::FontDescription,
    prelude::{AccelLabelExt, BoxExt, CheckMenuItemExt, ContainerExt, CssProviderExt, GtkMenuItemExt, RadioMenuItemExt, StyleContextExt, WidgetExt},
    AccelLabel, CssProvider, Orientation, StyleProvider, Widget, STYLE_PROVIDER_PRIORITY_APPLICATION,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
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
    pub icon: Option<PathBuf>,
    pub uuid: u16,
    pub(crate) gtk_menu_item_handle: isize,
    pub(crate) items: Option<Vec<MenuItem>>,
    gtk_menu_handle: isize,
    suppress_event: bool,
}

impl MenuItem {
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
        if self.gtk_menu_item_handle == 0 {
            return;
        }

        let gtk_menu_item = to_gtk_menu_item(self.gtk_menu_item_handle);
        let menu_item = get_menu_item_data_mut(&gtk_menu_item);
        gtk_menu_item.set_sensitive(!disabled);
        menu_item.disabled = disabled;
        set_menu_item_data(&gtk_menu_item, menu_item);
    }

    pub fn set_label(&mut self, label: &str) {
        self.label = label.to_string();
        if self.gtk_menu_item_handle == 0 {
            return;
        }

        let gtk_menu_item = to_gtk_menu_item(self.gtk_menu_item_handle);
        let menu_item = get_menu_item_data_mut(&gtk_menu_item);
        gtk_menu_item.set_label(label);
        menu_item.label = label.to_string();
        set_menu_item_data(&gtk_menu_item, menu_item);
    }

    pub fn set_icon(&mut self, icon: Option<PathBuf>) {
        if self.menu_item_type == MenuItemType::Separator {
            return;
        }

        self.icon = icon;
        if self.gtk_menu_item_handle == 0 {
            return;
        }

        let data = get_menu_data(self.gtk_menu_handle);
        let gtk_menu_item = to_gtk_menu_item(self.gtk_menu_item_handle);

        if let Some(icon) = &self.icon {
            let image_item = get_image_item(&gtk_menu_item);
            apply_image_css(&image_item, icon, &data.config);
        }

        let menu_item = get_menu_item_data_mut(&gtk_menu_item);
        menu_item.icon = self.icon.clone();
        set_menu_item_data(&gtk_menu_item, menu_item);

        toggle_icon(self.gtk_menu_handle);
    }
}

impl MenuItem {
    pub fn new_text_item(id: &str, label: &str, accelerator: Option<&str>, disabled: Option<bool>, icon: Option<PathBuf>) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            accelerator: accelerator.unwrap_or("").to_string(),
            name: String::new(),
            menu_item_type: MenuItemType::Text,
            submenu: None,
            uuid: UUID.fetch_add(1, Ordering::Relaxed),
            gtk_menu_item_handle: 0,
            gtk_menu_handle: 0,
            checked: false,
            disabled: disabled.unwrap_or(false),
            items: None,
            icon,
            suppress_event: false,
        }
    }
}

impl MenuItem {
    pub fn new_check_item(id: &str, label: &str, accelerator: Option<&str>, checked: bool, disabled: Option<bool>, icon: Option<PathBuf>) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            accelerator: accelerator.unwrap_or("").to_string(),
            name: String::new(),
            menu_item_type: MenuItemType::Checkbox,
            submenu: None,
            uuid: UUID.fetch_add(1, Ordering::Relaxed),
            gtk_menu_item_handle: 0,
            gtk_menu_handle: 0,
            checked,
            disabled: disabled.unwrap_or(false),
            items: None,
            icon,
            suppress_event: false,
        }
    }

    pub fn new_radio_item(id: &str, label: &str, name: &str, accelerator: Option<&str>, checked: bool, disabled: Option<bool>, icon: Option<PathBuf>) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            accelerator: accelerator.unwrap_or("").to_string(),
            name: name.to_string(),
            menu_item_type: MenuItemType::Radio,
            submenu: None,
            uuid: UUID.fetch_add(1, Ordering::Relaxed),
            gtk_menu_item_handle: 0,
            gtk_menu_handle: 0,
            checked,
            disabled: disabled.unwrap_or(false),
            items: None,
            icon,
            suppress_event: false,
        }
    }

    pub fn set_checked(&mut self, checked: bool) {
        self.checked = checked;
        if self.gtk_menu_item_handle == 0 {
            return;
        }

        let gtk_menu_item = to_gtk_menu_item(self.gtk_menu_item_handle);
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
    pub fn new_submenu_item(id: &str, label: &str, disabled: Option<bool>, icon: Option<PathBuf>) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            accelerator: String::new(),
            name: String::new(),
            menu_item_type: MenuItemType::Submenu,
            submenu: None,
            uuid: UUID.fetch_add(1, Ordering::Relaxed),
            gtk_menu_item_handle: 0,
            gtk_menu_handle: 0,
            checked: false,
            disabled: disabled.unwrap_or(false),
            items: Some(Vec::new()),
            icon,
            suppress_event: false,
        }
    }

    /// Adds a menu item to submenu.
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
            gtk_menu_item_handle: 0,
            gtk_menu_handle: 0,
            checked: false,
            disabled: false,
            items: None,
            icon: None,
            suppress_event: false,
        }
    }
}

pub(crate) fn radio_group_from_item(item: &MenuItem) -> HashMap<String, gtk::RadioMenuItem> {
    let gtk_radio_item = to_gtk_menu_item(item.gtk_menu_item_handle).downcast::<gtk::RadioMenuItem>().unwrap();
    HashMap::from([(item.name.clone(), gtk_radio_item)])
}

pub(crate) fn toggle_icon(gtk_menu_handle: isize) {
    let data = get_menu_data(gtk_menu_handle);
    let menu_items = collect_menu_items(gtk_menu_handle);
    let has_check = menu_items.iter().any(|item| item.menu_item_type == MenuItemType::Checkbox || item.menu_item_type == MenuItemType::Radio);
    let has_icon = menu_items.iter().any(|item| item.icon.is_some());
    for menu_item in menu_items {
        if menu_item.menu_item_type != MenuItemType::Separator {
            let gtk_menu_item = to_gtk_menu_item(menu_item.gtk_menu_item_handle);
            let image_item = get_image_item(&gtk_menu_item);

            /*
                Use icon margin if
                - Menu has no check item but has any icon item
                - Menu has any icon item and reserve_icon_size is true
                - This item has icon
            */
            if !has_icon {
                image_item.hide();
            } else if (!has_check && has_icon) || data.config.icon.as_ref().unwrap().reserve_icon_size || menu_item.icon.is_some() {
                image_item.show();
            }
        }
    }
}

fn get_image_item(gtk_menu_item: &gtk::MenuItem) -> Widget {
    let children = gtk_menu_item.children();
    let box_container: &gtk::Box = children.first().unwrap().downcast_ref().unwrap();
    box_container.children().first().unwrap().clone()
}

fn create_icon_label(label: &str, accelerator: &str, icon: &Option<PathBuf>, config: &Config, accel_widget: Option<&impl IsA<Widget>>) -> gtk::Box {
    let box_container = gtk::Box::new(Orientation::Horizontal, 6);
    let accel_label = AccelLabel::builder().label(label).xalign(0.0).build();
    accel_label.set_accel_widget(accel_widget);

    if !accelerator.is_empty() {
        /* When Image + Label + Accelerator, prevent Label width from being too small */
        let layout = accel_label.create_pango_layout(Some(label));
        let weight = to_font_weight_string(std::cmp::max(to_font_weight(config.font.dark_font_weight), to_font_weight(config.font.light_font_weight)));
        let size = config.font.dark_font_size.max(config.font.light_font_size);
        let font = format!(r#""{}" Normal {} {}"#, config.font.font_family, weight, size);
        let desc = FontDescription::from_string(&font);
        layout.set_font_description(Some(&desc));
        let (width, _) = layout.pixel_size();
        accel_label.set_width_request(width);
    }
    accel_label.show();

    /* Initially hide Image */
    let image = if let Some(icon) = icon {
        let image = gtk::Image::new();
        apply_image_css(&image, icon, config);
        image
    } else {
        gtk::Image::new()
    };

    box_container.pack_start(&image, false, false, 0);
    box_container.pack_start(&accel_label, true, true, 0);
    box_container.show();

    box_container
}

fn apply_image_css(image: &impl IsA<Widget>, icon: &Path, config: &Config) {
    let css_provider = CssProvider::new();
    let css = get_icon_menu_css(icon, config);
    css_provider.load_from_data(css.as_bytes()).unwrap();
    image.style_context().add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
}

pub(crate) fn create_gtk_menu_item(
    gtk_menu_handle: isize,
    item: &mut MenuItem,
    submenudata_map: Option<&HashMap<u16, SubmenuData>>,
    radio_groups: Option<&mut HashMap<String, gtk::RadioMenuItem>>,
    config: &Config,
) -> gtk::MenuItem {
    let gtk_menu_item = match item.menu_item_type {
        MenuItemType::Text => {
            let gtk_menu_item = gtk::MenuItem::builder().sensitive(!item.disabled).build();
            let box_container = create_icon_label(&item.label, &item.accelerator, &item.icon, config, Some(&gtk_menu_item));
            gtk_menu_item.add(&box_container);

            item.gtk_menu_item_handle = from_gtk_menu_item(&gtk_menu_item);
            gtk_menu_item
        }
        MenuItemType::Checkbox => {
            let check_menu_item = gtk::CheckMenuItem::builder().sensitive(!item.disabled).active(item.checked).build();
            let gtk_menu_item = check_menu_item.upcast::<gtk::MenuItem>();
            let box_container = create_icon_label(&item.label, &item.accelerator, &item.icon, config, Some(&gtk_menu_item));
            gtk_menu_item.add(&box_container);

            item.gtk_menu_item_handle = from_gtk_menu_item(&gtk_menu_item);
            gtk_menu_item
        }
        MenuItemType::Radio => {
            let radio_menu_item = gtk::RadioMenuItem::builder().draw_as_radio(false).sensitive(!item.disabled).active(item.checked).build();
            if let Some(radio_groups) = radio_groups {
                if radio_groups.contains_key(&item.name) {
                    let radio_group = radio_groups.get(&item.name).unwrap();
                    radio_menu_item.join_group(Some(radio_group));
                } else {
                    radio_groups.insert(item.name.clone(), radio_menu_item.clone());
                }
            }
            let gtk_menu_item = radio_menu_item.upcast::<gtk::MenuItem>();
            let box_container = create_icon_label(&item.label, &item.accelerator, &item.icon, config, Some(&gtk_menu_item));
            gtk_menu_item.add(&box_container);

            item.gtk_menu_item_handle = from_gtk_menu_item(&gtk_menu_item);
            gtk_menu_item
        }
        MenuItemType::Separator => {
            let gtk_menu_item = gtk::SeparatorMenuItem::new().upcast::<gtk::MenuItem>();
            item.gtk_menu_item_handle = from_gtk_menu_item(&gtk_menu_item);
            gtk_menu_item
        }
        MenuItemType::Submenu => {
            let submenudata_map = submenudata_map.unwrap();
            let submedata = submenudata_map.get(&item.uuid).unwrap();
            if submedata.gtk_submenu.children().is_empty() {
                submedata.gtk_submenu.set_sensitive(false);
            }

            let gtk_menu_item = gtk::MenuItem::builder().sensitive(!item.disabled).build();
            let box_container = create_icon_label(&item.label, &item.accelerator, &item.icon, config, Some(&gtk_menu_item));
            gtk_menu_item.add(&box_container);

            gtk_menu_item.set_submenu(Some(&submedata.gtk_submenu));
            item.gtk_menu_item_handle = from_gtk_menu_item(&gtk_menu_item);
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

    item.gtk_menu_handle = gtk_menu_handle;
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
