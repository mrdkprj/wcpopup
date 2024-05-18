use crate::{util::{decode_wide, encode_wide, get_menu_data, get_menu_data_mut, set_menu_data, MenuItemState, RMENU_TYPE}, MENU_CHECKED, MENU_DISABLED};
use serde::Serialize;
use windows::Win32::Foundation::HWND;


#[derive(Debug, Clone)]
pub(crate) struct InnerMenuItem {
    pub(crate) id:Vec<u16>,
    pub(crate) label:Vec<u16>,
    pub(crate) value:Vec<u16>,
    pub(crate) accelerator:Option<Vec<u16>>,
    pub(crate) name:Option<Vec<u16>>,
    pub(crate) state:MenuItemState,
    pub(crate) menu_type:RMENU_TYPE,
    pub(crate) index:i32,
    pub(crate) top:i32,
    pub(crate) bottom:i32,
    pub(crate) submenu:Option<HWND>,
}

#[derive(Debug, Clone)]
pub struct MenuItem {
    pub id:String,
    pub label:String,
    pub value:String,
    pub accelerator:String,
    pub name:String,
    pub state:MenuItemState,
    pub menu_type:RMENU_TYPE,
    index:usize,
    hwnd:HWND,
}

#[derive(Debug, Clone, Serialize)]
pub struct SelectedMenuItem {
    pub id:String,
    pub label:String,
    pub value:String,
    pub name:String,
    pub state:MenuItemState,
}

impl SelectedMenuItem {
    pub(crate) fn from(item: &InnerMenuItem) -> Self {
        Self {
            id: decode_wide(&item.id),
            label: decode_wide(&item.label),
            value: decode_wide(&item.value),
            name: if item.name.is_none() { String::new() } else { decode_wide(item.name.as_ref().unwrap()) },
            state: item.state.clone(),
        }
    }
}

impl MenuItem {

    pub(crate) fn new(hwnd:HWND, item:&InnerMenuItem) -> Self {
        Self {
            index: item.index as usize,
            hwnd,
            id: decode_wide(&item.id),
            label:decode_wide(&item.label),
            value: decode_wide(&item.value),
            accelerator: if item.accelerator.is_none() { String::new() } else { decode_wide(item.accelerator.as_ref().unwrap()) },
            name: if item.name.is_none() { String::new() } else { decode_wide(item.name.as_ref().unwrap()) },
            state: item.state.clone(),
            menu_type: item.menu_type.clone(),
        }
    }

    pub fn checked(&self) -> bool {
        let data = get_menu_data(self.hwnd);
        (data.items[self.index as usize].state.0 & MENU_CHECKED.0) != 0
    }

    pub fn set_checked(&self, checked:bool){
        let data = get_menu_data_mut(self.hwnd);
        if checked {
            data.items[self.index as usize].state.0 |= MENU_CHECKED.0;
        } else {
            data.items[self.index as usize].state.0 &= !MENU_CHECKED.0;
        }
        set_menu_data(self.hwnd, data);
    }

    pub fn disabled(&self) -> bool {
        let data = get_menu_data(self.hwnd);
        (data.items[self.index as usize].state.0 & MENU_DISABLED.0) != 0
    }

    pub fn set_disabled(&self, disabled:bool){
        let data = get_menu_data_mut(self.hwnd);
        if disabled {
            data.items[self.index as usize].state.0 |= MENU_DISABLED.0;
        } else {
            data.items[self.index as usize].state.0 &= !MENU_DISABLED.0;
        }
        set_menu_data(self.hwnd, data);
    }

    pub fn set_label(&self, label:&str){
        let data = get_menu_data_mut(self.hwnd);
        data.items[self.index as usize].label = encode_wide(label);
        set_menu_data(self.hwnd, data);
    }
}

impl InnerMenuItem {

    pub(crate) fn new(id:&str, label:&str, value:&str, accelerator:Option<&str>, name:Option<&str>, state:MenuItemState, menu_type:RMENU_TYPE) -> Self {
        Self {
            id: encode_wide(id),
            label:encode_wide(label),
            value: encode_wide(value),
            accelerator: if accelerator.is_some() { Some(encode_wide(accelerator.unwrap())) } else { None },
            name: if name.is_some() { Some(encode_wide(name.unwrap())) } else { None },
            state,
            menu_type,
            index:0,
            top:0,
            bottom:0,
            submenu:None,
        }
    }
}