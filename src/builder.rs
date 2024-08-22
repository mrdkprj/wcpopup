#[cfg(feature = "accelerator")]
use crate::accelerator::create_haccel;
use crate::{
    direct2d::{create_check_svg, create_render_target, create_submenu_svg},
    get_menu_data, hw, is_win11, set_window_border_color, Button, ButtonSize, Config, Corner, Menu, MenuItem, MenuItemType, MenuType, Theme,
};
#[cfg(feature = "accelerator")]
use std::collections::HashMap;
#[cfg(feature = "accelerator")]
use std::rc::Rc;
use std::{
    mem::size_of,
    os::raw::c_void,
    sync::atomic::{AtomicU32, Ordering},
};
#[cfg(feature = "accelerator")]
use windows::Win32::UI::WindowsAndMessaging::HACCEL;
use windows::{
    core::Error,
    Win32::{
        Foundation::HWND,
        Graphics::{
            Direct2D::{ID2D1DCRenderTarget, ID2D1SvgDocument},
            Dwm::{DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND, DWM_WINDOW_CORNER_PREFERENCE},
        },
        UI::WindowsAndMessaging::{SetWindowLongPtrW, GWL_USERDATA},
    },
};

static COUNTER: AtomicU32 = AtomicU32::new(400);
const MIN_LR_BUTTON_WIDTH: i32 = 12;
const CHECK_BUTTON_MARGIN: i32 = 10;
const ARROW_BUTTON_MARGIN: i32 = 5;

#[derive(Debug, Clone)]
pub(crate) struct MenuData {
    pub(crate) menu_type: MenuType,
    pub(crate) items: Vec<MenuItem>,
    pub(crate) win_subclass_id: Option<u32>,
    pub(crate) selected_index: i32,
    pub(crate) width: i32,
    pub(crate) height: i32,
    pub(crate) button: Button,
    pub(crate) visible_submenu_index: i32,
    pub(crate) current_theme: Theme,
    pub(crate) config: Config,
    pub(crate) thread_id: u32,
    pub(crate) parent: isize,
    pub(crate) dc_render_target: ID2D1DCRenderTarget,
    pub(crate) check_svg: ID2D1SvgDocument,
    pub(crate) submenu_svg: ID2D1SvgDocument,
    #[cfg(feature = "accelerator")]
    pub(crate) haccel: Option<Rc<HACCEL>>,
    #[cfg(feature = "accelerator")]
    pub(crate) accelerators: HashMap<u16, String>,
}

/// Builder to create Menu.
pub struct MenuBuilder {
    pub(crate) menu: Menu,
    items: Vec<MenuItem>,
    theme: Theme,
    config: Config,
    menu_type: MenuType,
}

impl MenuBuilder {
    /// Creates a new Menu for the specified window handle.
    pub fn new(window_handle: isize) -> Self {
        Self::new_builder(window_handle)
    }

    /// Creates a new Menu for the specified HWND.
    pub fn new_for_hwnd(hwnd: HWND) -> Self {
        let window_handle = hwnd.0 as isize;
        Self::new_builder(window_handle)
    }

    fn new_builder(window_handle: isize) -> Self {
        let mut menu = Menu::default();
        menu.parent_window_handle = window_handle;
        menu.window_handle = menu.create_window(window_handle);
        let config = Config::default();
        let theme = config.theme;
        Self {
            menu,
            items: Vec::new(),
            config,
            theme,
            menu_type: MenuType::Main,
        }
    }

    /// Creates a new Menu with the specified Theme for the specified window handle.
    pub fn new_with_theme(window_handle: isize, theme: Theme) -> Self {
        Self::new_builder_with_theme(window_handle, theme)
    }

    /// Creates a new Menu with the specified Theme for the specified HWND.
    pub fn new_for_hwnd_with_theme(hwnd: HWND, theme: Theme) -> Self {
        let window_handle = hwnd.0 as isize;
        Self::new_builder_with_theme(window_handle, theme)
    }

    fn new_builder_with_theme(window_handle: isize, theme: Theme) -> Self {
        let mut menu = Menu::default();
        menu.parent_window_handle = window_handle;
        menu.window_handle = menu.create_window(window_handle);
        let config = Config {
            theme,
            ..Default::default()
        };
        Self {
            menu,
            items: Vec::new(),
            config,
            theme,
            menu_type: MenuType::Main,
        }
    }

    /// Creates a new Menu using the specified Config for the specified window handle.
    pub fn new_from_config(window_handle: isize, config: Config) -> Self {
        Self::new_builder_from_config(window_handle, config)
    }

    /// Creates a new Menu using the specified Config for the specified HWND.
    pub fn new_for_hwnd_from_config(hwnd: HWND, config: Config) -> Self {
        let window_handle = hwnd.0 as isize;
        Self::new_builder_from_config(window_handle, config)
    }

    fn new_builder_from_config(window_handle: isize, config: Config) -> Self {
        let mut menu = Menu::default();
        menu.parent_window_handle = window_handle;
        menu.window_handle = menu.create_window(window_handle);
        let theme = config.theme;
        Self {
            menu,
            items: Vec::new(),
            config: Config {
                corner: if !is_win11() && config.corner == Corner::Round {
                    Corner::DoNotRound
                } else {
                    config.corner
                },
                ..config
            },
            theme,
            menu_type: MenuType::Main,
        }
    }

    pub(crate) fn new_for_submenu(parent: &Menu) -> Self {
        let data = get_menu_data(parent.window_handle);
        let config = Config {
            corner: if !is_win11() && data.config.corner == Corner::Round {
                Corner::DoNotRound
            } else {
                data.config.corner
            },
            ..data.config.clone()
        };

        let mut menu = Menu::default();
        menu.parent_window_handle = parent.window_handle;
        menu.window_handle = menu.create_window(parent.window_handle);

        Self {
            menu,
            items: Vec::new(),
            config,
            theme: data.current_theme,
            menu_type: MenuType::Submenu,
        }
    }

    /// Adds a text MenuItem to Menu.
    pub fn text(&mut self, id: &str, label: &str, disabled: Option<bool>) -> &Self {
        let mut item = MenuItem::new_text_item(id, label, None, disabled);
        item.menu_window_handle = self.menu.window_handle;
        self.items.push(item);
        self
    }

    pub fn text_with_accelerator(&mut self, id: &str, label: &str, disabled: Option<bool>, accelerator: &str) -> &Self {
        let mut item = MenuItem::new_text_item(id, label, Some(accelerator), disabled);
        item.menu_window_handle = self.menu.window_handle;
        self.items.push(item);
        self
    }

    /// Adds a check MenuItem to Menu.
    pub fn check(&mut self, id: &str, label: &str, checked: bool, disabled: Option<bool>) -> &Self {
        let mut item = MenuItem::new_check_item(id, label, None, checked, disabled);
        item.menu_window_handle = self.menu.window_handle;
        self.items.push(item);
        self
    }

    pub fn check_with_accelerator(&mut self, id: &str, label: &str, checked: bool, disabled: Option<bool>, accelerator: &str) -> &Self {
        let mut item = MenuItem::new_check_item(id, label, Some(accelerator), checked, disabled);
        item.menu_window_handle = self.menu.window_handle;
        self.items.push(item);
        self
    }

    /// Adds a radio MenuItem to Menu.
    pub fn radio(&mut self, id: &str, label: &str, name: &str, checked: bool, disabled: Option<bool>) -> &Self {
        let mut item = MenuItem::new_radio_item(id, label, name, None, checked, disabled);
        item.menu_window_handle = self.menu.window_handle;
        self.items.push(item);
        self
    }

    pub fn radio_with_accelerator(&mut self, id: &str, label: &str, name: &str, checked: bool, disabled: Option<bool>, accelerator: &str) -> &Self {
        let mut item = MenuItem::new_radio_item(id, label, name, Some(accelerator), checked, disabled);
        item.menu_window_handle = self.menu.window_handle;
        self.items.push(item);
        self
    }

    /// Adds a separator to Menu.
    pub fn separator(&mut self) -> &Self {
        let mut item = MenuItem::new_separator();
        item.menu_window_handle = self.menu.window_handle;
        self.items.push(item);
        self
    }

    /// Adds a submenu MenuItem to Menu.
    pub fn submenu(&mut self, id: &str, label: &str, disabled: Option<bool>) -> Self {
        let mut item = MenuItem::new(self.menu.window_handle, id, label, "", "", false, disabled, MenuItemType::Submenu, None);
        let mut builder = Self::new_from_config(self.menu.window_handle, self.config.clone());
        builder.menu_type = MenuType::Submenu;

        // Set dummy menu to be replaced later
        item.submenu = Some(builder.menu.clone());
        self.items.push(item);

        builder
    }

    /// Build Menu to make it ready to become visible.
    /// Must call this function before showing Menu, otherwise nothing shows up.
    pub fn build(mut self) -> Result<Menu, Error> {
        let is_main_menu = self.menu_type == MenuType::Main;

        #[cfg(feature = "accelerator")]
        let mut accelerators = HashMap::new();
        #[cfg(feature = "accelerator")]
        let mut haccel = None;
        #[cfg(feature = "accelerator")]
        if is_main_menu {
            Self::collect_accelerators(&self.items, &mut accelerators);
            if !accelerators.is_empty() {
                match create_haccel(&accelerators) {
                    Some(accel) => haccel = Some(Rc::new(accel)),
                    None => haccel = None,
                }
            }
        }

        let dc_render_target = create_render_target();
        let check_svg_doc = create_check_svg(&dc_render_target, &self.config.font);
        let submenu_svg_doc = create_submenu_svg(&dc_render_target, &self.config.font);

        let has_checkbox = self.items.iter().any(|item| item.menu_item_type == MenuItemType::Checkbox || item.menu_item_type == MenuItemType::Radio);
        let has_submenu = self.items.iter().any(|item| item.menu_item_type == MenuItemType::Submenu);
        let default_button_size = ButtonSize {
            width: MIN_LR_BUTTON_WIDTH,
            margins: MIN_LR_BUTTON_WIDTH,
        };
        let button = Button {
            left: if has_checkbox {
                ButtonSize {
                    width: check_svg_doc.size as i32,
                    margins: MIN_LR_BUTTON_WIDTH + CHECK_BUTTON_MARGIN * 2,
                }
            } else {
                default_button_size
            },
            right: if has_submenu {
                ButtonSize {
                    width: submenu_svg_doc.size as i32,
                    margins: MIN_LR_BUTTON_WIDTH + ARROW_BUTTON_MARGIN * 2,
                }
            } else {
                default_button_size
            },
        };

        let size = self.menu.calculate(&mut self.items, &self.config, self.config.theme, button)?;

        if is_main_menu {
            Self::rebuild_submenu(&mut self, button);
        }

        let data = MenuData {
            menu_type: self.menu_type,
            items: self.items.clone(),
            win_subclass_id: if is_main_menu {
                Some(COUNTER.fetch_add(1, Ordering::Relaxed))
            } else {
                None
            },
            #[cfg(feature = "accelerator")]
            haccel,
            #[cfg(feature = "accelerator")]
            accelerators,
            height: size.height,
            width: size.width,
            button,
            selected_index: -1,
            visible_submenu_index: -1,
            current_theme: self.theme,
            config: self.config.clone(),
            thread_id: 0,
            parent: if is_main_menu {
                0
            } else {
                self.menu.parent_window_handle
            },
            dc_render_target,
            check_svg: check_svg_doc.document,
            submenu_svg: submenu_svg_doc.document,
        };

        if is_main_menu {
            self.menu.attach_owner_subclass(data.win_subclass_id.unwrap() as usize);
        }

        let hwnd = hw!(self.menu.window_handle);
        if is_win11() {
            if self.config.corner == Corner::Round {
                unsafe { DwmSetWindowAttribute(hwnd, DWMWA_WINDOW_CORNER_PREFERENCE, &DWMWCP_ROUND as *const _ as *const c_void, size_of::<DWM_WINDOW_CORNER_PREFERENCE>() as u32)? };
            }

            set_window_border_color(self.menu.window_handle, &data)?;
        }

        unsafe { SetWindowLongPtrW(hwnd, GWL_USERDATA, Box::into_raw(Box::new(data)) as _) };

        Ok(self.menu)
    }

    fn rebuild_submenu(&mut self, buttom: Button) {
        for item in self.items.iter_mut() {
            if item.menu_item_type == MenuItemType::Submenu {
                let mut submenu = item.submenu.as_ref().unwrap().clone();
                submenu.menu_type = MenuType::Submenu;
                let _ = submenu.calculate(&mut submenu.items(), &self.config, self.config.theme, buttom);
                item.submenu = Some(submenu);
            }
        }
    }

    #[cfg(feature = "accelerator")]
    fn collect_accelerators(items: &Vec<MenuItem>, accelerators: &mut HashMap<u16, String>) {
        for item in items {
            if item.menu_item_type == MenuItemType::Submenu {
                let submenu_window_handle = item.submenu.as_ref().unwrap().window_handle;
                let data = get_menu_data(submenu_window_handle);
                Self::collect_accelerators(&data.items, accelerators);
            } else if !item.accelerator.is_empty() {
                accelerators.insert(item.uuid, item.accelerator.clone());
            }
        }
    }
}
