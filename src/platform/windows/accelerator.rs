#[cfg(feature = "accelerator")]
use super::{get_menu_data, MenuData};
#[cfg(feature = "accelerator")]
use crate::platform::platform_impl::vtoi;
#[cfg(feature = "accelerator")]
use std::collections::HashMap;
#[cfg(feature = "accelerator")]
use windows::Win32::{
    Foundation::HWND,
    UI::{
        Input::KeyboardAndMouse::{
            VIRTUAL_KEY, VK_0, VK_1, VK_2, VK_3, VK_4, VK_5, VK_6, VK_7, VK_8, VK_9, VK_A, VK_APPS, VK_B, VK_BACK, VK_BROWSER_HOME, VK_C, VK_CAPITAL, VK_CONVERT, VK_D, VK_DELETE, VK_DOWN, VK_E,
            VK_END, VK_ESCAPE, VK_F, VK_F1, VK_F10, VK_F11, VK_F12, VK_F13, VK_F14, VK_F15, VK_F16, VK_F17, VK_F18, VK_F19, VK_F2, VK_F20, VK_F21, VK_F22, VK_F23, VK_F24, VK_F3, VK_F4, VK_F5, VK_F6,
            VK_F7, VK_F8, VK_F9, VK_G, VK_H, VK_HELP, VK_HOME, VK_I, VK_INSERT, VK_J, VK_K, VK_KANA, VK_L, VK_LEFT, VK_M, VK_MEDIA_NEXT_TRACK, VK_MEDIA_PLAY_PAUSE, VK_MEDIA_PREV_TRACK, VK_MEDIA_STOP,
            VK_N, VK_NEXT, VK_NONCONVERT, VK_NUMLOCK, VK_O, VK_OEM_1, VK_OEM_2, VK_OEM_3, VK_OEM_4, VK_OEM_5, VK_OEM_6, VK_OEM_7, VK_OEM_COMMA, VK_OEM_MINUS, VK_OEM_PERIOD, VK_OEM_PLUS, VK_P,
            VK_PAUSE, VK_PRIOR, VK_Q, VK_R, VK_RETURN, VK_RIGHT, VK_S, VK_SCROLL, VK_SNAPSHOT, VK_SPACE, VK_T, VK_TAB, VK_U, VK_UP, VK_V, VK_VOLUME_DOWN, VK_VOLUME_MUTE, VK_VOLUME_UP, VK_W, VK_X,
            VK_Y, VK_Z,
        },
        WindowsAndMessaging::{CreateAcceleratorTableW, DestroyAcceleratorTable, TranslateAcceleratorW, ACCEL, FALT, FCONTROL, FSHIFT, FVIRTKEY, HACCEL, MSG},
    },
};

#[cfg(feature = "accelerator")]
const MODIFIERS: [&str; 3] = ["CTRL", "ALT", "SHIFT"];

#[cfg(feature = "accelerator")]
pub(crate) fn translate_accel(hwnd: HWND, msg: MSG) {
    let data = get_menu_data(vtoi!(hwnd.0));
    if let Some(accel) = &data.haccel {
        unsafe { TranslateAcceleratorW(hwnd, HACCEL(accel.0), &msg) };
    }
}

#[cfg(feature = "accelerator")]
pub(crate) fn destroy_haccel(data: &MenuData) {
    if let Some(haccel) = &data.haccel {
        let haccel = HACCEL(haccel.0);
        let _ = unsafe { DestroyAcceleratorTable(haccel) };
    }
}

#[cfg(feature = "accelerator")]
pub(crate) fn create_haccel(accelerators: &HashMap<u16, String>) -> Option<HACCEL> {
    let mut accels = Vec::new();

    for (cmd, accel_key) in accelerators {
        let upper_key = accel_key.to_uppercase();
        let upper_keys: Vec<&str> = upper_key.split('+').collect();

        if MODIFIERS.contains(&upper_keys[upper_keys.len() - 1]) {
            continue;
        }

        let keys: Vec<&str> = accel_key.split('+').collect();
        let key = keys[keys.len() - 1];

        let virtual_key = get_virtual_key(key);

        if virtual_key == VIRTUAL_KEY(0) {
            continue;
        }

        let mut virt = FVIRTKEY;

        if upper_keys.contains(&"CTRL") {
            virt |= FCONTROL;
        }

        if upper_keys.contains(&"ALT") {
            virt |= FALT;
        }

        if upper_keys.contains(&"SHIFT") {
            virt |= FSHIFT;
        }

        let accel = ACCEL {
            fVirt: virt,
            key: virtual_key.0,
            cmd: *cmd,
        };

        accels.push(accel);
    }

    if accels.is_empty() {
        None
    } else {
        let haccel = unsafe { CreateAcceleratorTableW(&accels).unwrap() };
        Some(haccel)
    }
}

#[cfg(feature = "accelerator")]
fn get_virtual_key(key_string: &str) -> VIRTUAL_KEY {
    match key_string {
        "A" => VK_A,
        "B" => VK_B,
        "C" => VK_C,
        "D" => VK_D,
        "E" => VK_E,
        "F" => VK_F,
        "G" => VK_G,
        "H" => VK_H,
        "I" => VK_I,
        "J" => VK_J,
        "K" => VK_K,
        "L" => VK_L,
        "M" => VK_M,
        "N" => VK_N,
        "O" => VK_O,
        "P" => VK_P,
        "Q" => VK_Q,
        "R" => VK_R,
        "S" => VK_S,
        "T" => VK_T,
        "U" => VK_U,
        "V" => VK_V,
        "W" => VK_W,
        "X" => VK_X,
        "Y" => VK_Y,
        "Z" => VK_Z,
        "0" => VK_0,
        "1" => VK_1,
        "2" => VK_2,
        "3" => VK_3,
        "4" => VK_4,
        "5" => VK_5,
        "6" => VK_6,
        "7" => VK_7,
        "8" => VK_8,
        "9" => VK_9,
        "F1" => VK_F1,
        "F2" => VK_F2,
        "F3" => VK_F3,
        "F4" => VK_F4,
        "F5" => VK_F5,
        "F6" => VK_F6,
        "F7" => VK_F7,
        "F8" => VK_F8,
        "F9" => VK_F9,
        "F10" => VK_F10,
        "F11" => VK_F11,
        "F12" => VK_F12,
        "F13" => VK_F13,
        "F14" => VK_F14,
        "F15" => VK_F15,
        "F16" => VK_F16,
        "F17" => VK_F17,
        "F18" => VK_F18,
        "F19" => VK_F19,
        "F20" => VK_F20,
        "F21" => VK_F21,
        "F22" => VK_F22,
        "F23" => VK_F23,
        "F24" => VK_F24,
        "Plus" => VK_OEM_PLUS,
        "Space" => VK_SPACE,
        "Tab" => VK_TAB,
        "CapsLock" => VK_CAPITAL,
        "NumLock" => VK_NUMLOCK,
        "ScrollLock" => VK_SCROLL,
        "Backspace" => VK_BACK,
        "Delete" => VK_DELETE,
        "Insert" => VK_INSERT,
        "Enter" => VK_RETURN,
        "ArrowLeft" => VK_LEFT,
        "ArrowUp" => VK_UP,
        "ArrowRight" => VK_RIGHT,
        "ArrowDown" => VK_DOWN,
        "End" => VK_END,
        "Home" => VK_HOME,
        "PageUp" => VK_PRIOR,
        "PageDown" => VK_NEXT,
        "Escape" => VK_ESCAPE,
        "BrowserHome" => VK_BROWSER_HOME,
        "AudioVolumeMute" => VK_VOLUME_MUTE,
        "AudioVolumeDown" => VK_VOLUME_DOWN,
        "AudioVolumeUp" => VK_VOLUME_UP,
        "MediaTrackNext" => VK_MEDIA_NEXT_TRACK,
        "MediaTrackPrevious" => VK_MEDIA_PREV_TRACK,
        "MediaStop" => VK_MEDIA_STOP,
        "MediaPlayPause" => VK_MEDIA_PLAY_PAUSE,
        "PrintScreen" => VK_SNAPSHOT,
        "Comma" => VK_OEM_COMMA,
        "Minus" => VK_OEM_MINUS,
        "Period" => VK_OEM_PERIOD,
        "Semicolon" => VK_OEM_1,
        "Slash" => VK_OEM_2,
        "Backquote" => VK_OEM_3,
        "BracketLeft" => VK_OEM_4,
        "Backslash" => VK_OEM_5,
        "BracketRight" => VK_OEM_6,
        "Quote" => VK_OEM_7,
        "Pause" => VK_PAUSE,
        "KanaMode" => VK_KANA,
        "NonConvert" => VK_NONCONVERT,
        "Help" => VK_HELP,
        "ContextMenu" => VK_APPS,
        "Convert" => VK_CONVERT,
        _ => {
            println!("Invalid key:{}", key_string);
            VIRTUAL_KEY(0)
        }
    }
}
