use super::to_gtk_menu_item;
#[cfg(feature = "accelerator")]
use super::to_gtk_window;
use super::{util::get_accel_group, MenuItem, MenuItemType};
#[cfg(feature = "accelerator")]
use gtk::{
    accelerator_name,
    glib::{Propagation, Quark},
    prelude::AccelGroupExt,
};
use gtk::{
    gdk::{keys::Key, ModifierType},
    prelude::WidgetExt,
    traits::GtkWindowExt,
    AccelFlags, AccelGroup,
};
use std::collections::HashMap;

pub(crate) struct AcceleratorKey {
    pub(crate) key: u32,
    pub(crate) modifier_type: ModifierType,
}

const MODIFIERS: [&str; 3] = ["CTRL", "ALT", "SHIFT"];

pub(crate) fn setup_accel_group(accelerators: &HashMap<isize, String>) -> AccelGroup {
    let accel_group = AccelGroup::new();
    for (menu_item_handle, accelerator) in accelerators {
        if let Some(accelerator_key) = get_accelerator_key(accelerator.as_str()) {
            let gtk_menu_item = to_gtk_menu_item(*menu_item_handle);
            gtk_menu_item.add_accelerator("activate", &accel_group, accelerator_key.key, accelerator_key.modifier_type, AccelFlags::VISIBLE);
        }
    }
    accel_group
}

pub(crate) fn add_accelerator(main_gtk_menu_handle: isize, new_accelerators: &HashMap<isize, String>) {
    let accel_group = get_accel_group(main_gtk_menu_handle);

    for (menu_item_handle, accelerator) in new_accelerators {
        if let Some(accelerator_key) = get_accelerator_key(accelerator.as_str()) {
            let gtk_menu_item = to_gtk_menu_item(*menu_item_handle);
            gtk_menu_item.add_accelerator("activate", accel_group, accelerator_key.key, accelerator_key.modifier_type, AccelFlags::VISIBLE);
        }
    }
}

pub(crate) fn add_accelerators_from_menu_item(main_gtk_menu_handle: isize, item: &MenuItem) {
    let items = if item.menu_item_type == MenuItemType::Submenu {
        item.submenu.as_ref().unwrap().items()
    } else {
        vec![item.clone()]
    };

    let mut items_with_accel = Vec::new();
    for item in items {
        if !item.accelerator.is_empty() {
            items_with_accel.push(item);
        }
    }

    if items_with_accel.is_empty() {
        return;
    }

    let mut accelerators = HashMap::new();

    for item in items_with_accel {
        accelerators.insert(item.gtk_menu_item_handle, item.accelerator.clone());
    }

    add_accelerator(main_gtk_menu_handle, &accelerators);
}

pub(crate) fn get_accelerator_key(accelerator: &str) -> Option<AcceleratorKey> {
    let upper_key = accelerator.to_uppercase();
    let upper_keys: Vec<&str> = upper_key.split('+').collect();

    if MODIFIERS.contains(&upper_keys[upper_keys.len() - 1]) {
        return None;
    }

    let keys: Vec<&str> = accelerator.split('+').collect();
    let key_str = keys[keys.len() - 1];
    let key = Key::from_name(key_str);

    let mut modifier_type = ModifierType::empty();

    if upper_keys.contains(&"CTRL") {
        modifier_type.set(ModifierType::CONTROL_MASK, true);
    }

    if upper_keys.contains(&"ALT") {
        modifier_type.set(ModifierType::MOD1_MASK, true);
    }

    if upper_keys.contains(&"SHIFT") {
        modifier_type.set(ModifierType::SHIFT_MASK, true);
    }

    Some(AcceleratorKey {
        key: *key,
        modifier_type,
    })
}

pub(crate) fn add_accel_group(gtk_window: &gtk::Window, gtk_menu_handle: isize) {
    let accel_group = get_accel_group(gtk_menu_handle);
    gtk_window.add_accel_group(accel_group);
}

pub(crate) fn remove_accel_group(gtk_window: &gtk::Window, gtk_menu_handle: isize) {
    let accel_group = get_accel_group(gtk_menu_handle);
    gtk_window.remove_accel_group(accel_group);
}

#[cfg(feature = "accelerator")]
pub(crate) fn connect_accelerator(gtk_menu: &gtk::Menu, gtk_menu_handle: isize, gtk_window_handle: isize) {
    gtk_menu.connect_key_press_event(move |memu, event| {
        let key_val = event.keyval();
        let mut modifiers = event.state();

        modifiers &= !(ModifierType::MOD2_MASK | ModifierType::MOD3_MASK | ModifierType::MOD4_MASK | ModifierType::MOD5_MASK);

        if let Some(quark) = accelerator_name(*key_val, modifiers) {
            let accel_quark = Quark::from_str(quark);
            let accel_group = get_accel_group(gtk_menu_handle);
            let gtk_window = to_gtk_window(gtk_window_handle);
            let result = accel_group.activate(accel_quark, &gtk_window, *key_val, modifiers);

            if result {
                memu.hide();
                return Propagation::Stop;
            } else {
                return Propagation::Proceed;
            }
        }

        Propagation::Proceed
    });
}
