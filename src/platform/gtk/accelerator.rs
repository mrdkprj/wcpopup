#[cfg(feature = "accelerator")]
use super::to_gtk_window;
use super::{get_menu_data, to_accel_group, to_gtk_menu_item, MenuItem};
use crate::MenuItemType;
#[cfg(feature = "accelerator")]
use gtk::{
    accelerator_name,
    glib::{Propagation, Quark},
    prelude::AccelGroupExt,
};
use gtk::{gdk::keys::Key, gdk::ModifierType, prelude::WidgetExt, AccelFlags, AccelGroup};
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

pub(crate) fn add_accelerator(gtk_menu_handle: isize, new_accelerators: &HashMap<isize, String>) {
    let data = get_menu_data(gtk_menu_handle);
    if let Some(accel_group_handle) = data.accel_group_handle {
        let accel_group = to_accel_group(accel_group_handle);
        for (menu_item_handle, accelerator) in new_accelerators {
            if let Some(accelerator_key) = get_accelerator_key(accelerator.as_str()) {
                let gtk_menu_item = to_gtk_menu_item(*menu_item_handle);
                gtk_menu_item.add_accelerator("activate", &accel_group, accelerator_key.key, accelerator_key.modifier_type, AccelFlags::VISIBLE);
            }
        }
    }
}

pub(crate) fn add_accelerators_from_menu_item(gtk_menu_handle: isize, item: &MenuItem) {
    let items = if item.menu_item_type == MenuItemType::Submenu {
        &item.submenu.as_ref().unwrap().items()
    } else {
        &vec![item.clone()]
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

    add_accelerator(gtk_menu_handle, &accelerators);
}

pub(crate) fn get_accelerator_key(accelerator: &str) -> Option<AcceleratorKey> {
    let upper = accelerator.to_uppercase();
    let upper_keys: Vec<&str> = upper.split('+').collect();

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

#[cfg(feature = "accelerator")]
pub(crate) fn connect_accelerator(gtk_menu: &gtk::Menu, gtk_menu_handle: isize, gtk_window_handle: isize) {
    gtk_menu.connect_key_press_event(move |memu, event| {
        let accel_key = event.keyval();
        let mut accel_mods = event.state();

        accel_mods &= !(ModifierType::MOD2_MASK | ModifierType::MOD3_MASK | ModifierType::MOD4_MASK | ModifierType::MOD5_MASK);

        let quark = accelerator_name(*accel_key, accel_mods);
        let accel_quark = Quark::from_str(quark.unwrap());

        let data = get_menu_data(gtk_menu_handle);
        let accel_group = to_accel_group(data.accel_group_handle.unwrap());
        let parent = to_gtk_window(gtk_window_handle);
        let result = accel_group.activate(accel_quark, &parent, *accel_key, accel_mods);

        if result {
            memu.hide();
            Propagation::Stop
        } else {
            Propagation::Proceed
        }
    });
}
