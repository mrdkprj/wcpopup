#[cfg(feature = "accelerator")]
use super::{accelerator::create_haccel, get_menu_data};
use super::{
    calculate,
    direct2d::{create_check_svg, create_menu_image, create_render_target, create_submenu_svg, create_svg_from_path, get_icon_space},
    hwnd, is_win11,
    menu_item::MenuItem,
    util::set_window_border_color,
    Config, Corner, IconSettings, IconSpace, Menu, PopupInfo, Size, Theme,
};
use crate::{MenuIcon, MenuItemType, MenuType};
#[cfg(feature = "accelerator")]
use std::rc::Rc;
use std::{
    collections::HashMap,
    mem::size_of,
    sync::atomic::{AtomicU32, Ordering},
};
#[cfg(feature = "accelerator")]
use windows::Win32::UI::WindowsAndMessaging::HACCEL;
use windows::{
    core::Error,
    Win32::{
        Foundation::HWND,
        Graphics::{
            Direct2D::{ID2D1Bitmap1, ID2D1DCRenderTarget, ID2D1SvgDocument},
            Dwm::{DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND, DWM_WINDOW_CORNER_PREFERENCE},
        },
        UI::WindowsAndMessaging::{SetWindowLongPtrW, GWL_USERDATA},
    },
};

static COUNTER: AtomicU32 = AtomicU32::new(400);

#[derive(Debug, Clone)]
pub(crate) enum MenuImageType {
    Bitmap(ID2D1Bitmap1),
    Svg(ID2D1SvgDocument),
}

#[derive(Debug, Clone)]
pub(crate) struct MenuData {
    pub(crate) popup_info: Option<PopupInfo>,
    pub(crate) menu_type: MenuType,
    pub(crate) items: Vec<MenuItem>,
    pub(crate) win_subclass_id: Option<u32>,
    pub(crate) selected_index: i32,
    pub(crate) size: Size,
    pub(crate) icon_space: IconSpace,
    pub(crate) visible_submenu_index: i32,
    pub(crate) current_theme: Theme,
    pub(crate) config: Config,
    pub(crate) parent: isize,
    pub(crate) dc_render_target: ID2D1DCRenderTarget,
    pub(crate) check_svg: ID2D1SvgDocument,
    pub(crate) submenu_svg: ID2D1SvgDocument,
    pub(crate) icon_map: HashMap<u16, MenuImageType>,
    #[cfg(feature = "accelerator")]
    pub(crate) haccel: Option<Rc<HACCEL>>,
    #[cfg(feature = "accelerator")]
    pub(crate) accelerators: HashMap<u16, String>,
    pub(crate) icon_size: i32,
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
                icon: if let Some(icon) = config.icon {
                    Some(icon)
                } else {
                    Some(IconSettings::default())
                },
                ..config
            },
            theme,
            menu_type: MenuType::Main,
        }
    }

    /// Adds a text MenuItem to Menu.
    pub fn text(&mut self, id: &str, label: &str, disabled: bool) -> &Self {
        let mut item = MenuItem::new_text_item(id, label, None, disabled, None);
        item.menu_window_handle = self.menu.window_handle;
        self.items.push(item);
        self
    }

    pub fn text_with_accelerator(&mut self, id: &str, label: &str, disabled: bool, accelerator: &str) -> &Self {
        let mut item = MenuItem::new_text_item(id, label, Some(accelerator), disabled, None);
        item.menu_window_handle = self.menu.window_handle;
        self.items.push(item);
        self
    }

    pub fn text_with_icon(&mut self, id: &str, label: &str, disabled: bool, icon: MenuIcon) -> &Self {
        let mut item = MenuItem::new_text_item(id, label, None, disabled, Some(icon));
        item.menu_window_handle = self.menu.window_handle;
        self.items.push(item);
        self
    }

    pub fn text_with_accel_icon(&mut self, id: &str, label: &str, disabled: bool, accelerator: &str, icon: MenuIcon) -> &Self {
        let mut item = MenuItem::new_text_item(id, label, Some(accelerator), disabled, Some(icon));
        item.menu_window_handle = self.menu.window_handle;
        self.items.push(item);
        self
    }

    /// Adds a check MenuItem to Menu.
    pub fn check(&mut self, id: &str, label: &str, checked: bool, disabled: bool) -> &Self {
        let mut item = MenuItem::new_check_item(id, label, None, checked, disabled, None);
        item.menu_window_handle = self.menu.window_handle;
        self.items.push(item);
        self
    }

    pub fn check_with_accelerator(&mut self, id: &str, label: &str, checked: bool, disabled: bool, accelerator: &str) -> &Self {
        let mut item = MenuItem::new_check_item(id, label, Some(accelerator), checked, disabled, None);
        item.menu_window_handle = self.menu.window_handle;
        self.items.push(item);
        self
    }

    pub fn check_with_icon(&mut self, id: &str, label: &str, checked: bool, disabled: bool, icon: MenuIcon) -> &Self {
        let mut item = MenuItem::new_check_item(id, label, None, checked, disabled, Some(icon));
        item.menu_window_handle = self.menu.window_handle;
        self.items.push(item);
        self
    }

    pub fn check_with_accel_icon(&mut self, id: &str, label: &str, checked: bool, disabled: bool, accelerator: &str, icon: MenuIcon) -> &Self {
        let mut item = MenuItem::new_check_item(id, label, Some(accelerator), checked, disabled, Some(icon));
        item.menu_window_handle = self.menu.window_handle;
        self.items.push(item);
        self
    }

    /// Adds a radio MenuItem to Menu.
    pub fn radio(&mut self, id: &str, label: &str, name: &str, checked: bool, disabled: bool) -> &Self {
        let mut item = MenuItem::new_radio_item(id, label, name, None, checked, disabled, None);
        item.menu_window_handle = self.menu.window_handle;
        self.items.push(item);
        self
    }

    pub fn radio_with_accelerator(&mut self, id: &str, label: &str, name: &str, checked: bool, disabled: bool, accelerator: &str) -> &Self {
        let mut item = MenuItem::new_radio_item(id, label, name, Some(accelerator), checked, disabled, None);
        item.menu_window_handle = self.menu.window_handle;
        self.items.push(item);
        self
    }

    pub fn radio_with_icon(&mut self, id: &str, label: &str, name: &str, checked: bool, disabled: bool, icon: MenuIcon) -> &Self {
        let mut item = MenuItem::new_radio_item(id, label, name, None, checked, disabled, Some(icon));
        item.menu_window_handle = self.menu.window_handle;
        self.items.push(item);
        self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn radio_with_accel_icon(&mut self, id: &str, label: &str, name: &str, checked: bool, disabled: bool, accelerator: &str, icon: MenuIcon) -> &Self {
        let mut item = MenuItem::new_radio_item(id, label, name, Some(accelerator), checked, disabled, Some(icon));
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
    pub fn submenu(&mut self, id: &str, label: &str, disabled: bool) -> Self {
        let mut item = MenuItem::new(self.menu.window_handle, id, label, "", "", false, disabled, MenuItemType::Submenu, None, None);
        let mut builder = Self::new_from_config(self.menu.window_handle, self.config.clone());
        builder.menu_type = MenuType::Submenu;

        /* Set dummy menu to be replaced later */
        item.submenu = Some(builder.menu.clone());
        self.items.push(item);

        builder
    }

    pub fn submenu_with_icon(&mut self, id: &str, label: &str, disabled: bool, icon: MenuIcon) -> Self {
        let mut item = MenuItem::new(self.menu.window_handle, id, label, "", "", false, disabled, MenuItemType::Submenu, None, Some(icon));
        let mut builder = Self::new_from_config(self.menu.window_handle, self.config.clone());
        builder.menu_type = MenuType::Submenu;

        /* Set dummy menu to be replaced later */
        item.submenu = Some(builder.menu.clone());
        self.items.push(item);

        builder
    }

    pub(crate) fn new_for_submenu(parent: &Menu, config: &Config, current_theme: Theme, items: &mut [MenuItem]) -> Self {
        let config = Config {
            corner: if !is_win11() && config.corner == Corner::Round {
                Corner::DoNotRound
            } else {
                config.corner
            },
            ..config.clone()
        };

        let mut menu = Menu::default();
        menu.parent_window_handle = parent.window_handle;
        menu.window_handle = menu.create_window(parent.window_handle);

        let items: Vec<MenuItem> = items
            .iter_mut()
            .map(|item| {
                item.menu_window_handle = menu.window_handle;
                item.clone()
            })
            .collect();

        Self {
            menu,
            items,
            config,
            theme: current_theme,
            menu_type: MenuType::Submenu,
        }
    }

    /// Adds a MenuItem to MenuBuilder.
    pub fn append(&mut self, mut menu_item: MenuItem) -> &Self {
        if menu_item.menu_item_type == MenuItemType::Submenu && menu_item.menu_window_handle == 0 {
            let mut builder = MenuBuilder::new_for_submenu(&self.menu, &self.config, self.config.theme, menu_item.items.as_mut().unwrap());
            let submenu = builder.build().unwrap();
            menu_item.menu_window_handle = submenu.parent_window_handle;
            menu_item.submenu = Some(submenu);
        }
        menu_item.menu_window_handle = self.menu.window_handle;
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
    pub fn build(&mut self) -> Result<Menu, Error> {
        let is_main_menu = self.menu_type == MenuType::Main;

        #[cfg(feature = "accelerator")]
        let mut accelerators = HashMap::new();
        #[cfg(feature = "accelerator")]
        let mut haccel = None;
        #[cfg(feature = "accelerator")]
        if is_main_menu {
            collect_accelerators(&self.items, &mut accelerators);
            if !accelerators.is_empty() {
                match create_haccel(&accelerators) {
                    Some(accel) => haccel = Some(Rc::new(accel)),
                    None => haccel = None,
                }
            }
        }

        let dc_render_target = create_render_target()?;

        let check_svg_doc = if let Some(svg) = &self.config.icon.as_ref().unwrap().check_svg {
            create_svg_from_path(&dc_render_target, svg)?
        } else {
            create_check_svg(&dc_render_target, &self.config)?
        };

        let submenu_svg_doc = if let Some(svg) = &self.config.icon.as_ref().unwrap().arrow_svg {
            create_svg_from_path(&dc_render_target, svg)?
        } else {
            create_submenu_svg(&dc_render_target, &self.config)?
        };

        let icon_size = unsafe { check_svg_doc.GetViewportSize().width } as _;
        /* Safe to unwrap icon which is Some(IconSettings::default()) by default */
        let icon_space = get_icon_space(&self.items, self.config.icon.as_ref().unwrap(), &check_svg_doc, &submenu_svg_doc);
        let size = calculate(&mut self.items, &self.config, self.config.theme, icon_space);

        if is_main_menu {
            /* Calculate items in all submenus */
            self.rebuild_submenu(icon_space);
        }

        /* Save icon for reuse */
        let mut icon_map = HashMap::new();
        for item in &self.items {
            if let Some(icon) = &item.icon {
                let bitmap = create_menu_image(&dc_render_target, icon, icon_size)?;
                icon_map.insert(item.uuid, bitmap);
            }
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
            size,
            icon_space,
            selected_index: -1,
            visible_submenu_index: -1,
            current_theme: self.theme,
            config: self.config.clone(),
            parent: if is_main_menu {
                0
            } else {
                self.menu.parent_window_handle
            },
            dc_render_target,
            check_svg: check_svg_doc,
            submenu_svg: submenu_svg_doc,
            popup_info: None,
            icon_map,
            icon_size,
        };

        if is_main_menu {
            self.menu.attach_owner_subclass(data.win_subclass_id.unwrap() as usize);
        }

        let hwnd = hwnd!(self.menu.window_handle);

        if is_win11() {
            if self.config.corner == Corner::Round {
                unsafe { DwmSetWindowAttribute(hwnd, DWMWA_WINDOW_CORNER_PREFERENCE, &DWMWCP_ROUND as *const _ as *const _, size_of::<DWM_WINDOW_CORNER_PREFERENCE>() as u32)? };
            }

            set_window_border_color(self.menu.window_handle, &data).unwrap();
        }

        unsafe { SetWindowLongPtrW(hwnd, GWL_USERDATA, Box::into_raw(Box::new(data)) as _) };

        Ok(self.menu.clone())
    }

    fn rebuild_submenu(&mut self, icon_space: IconSpace) {
        for item in self.items.iter_mut() {
            if item.menu_item_type == MenuItemType::Submenu {
                let submenu = item.submenu.as_mut().unwrap();
                submenu.menu_type = MenuType::Submenu;
                let _ = calculate(&mut submenu.items(), &self.config, self.config.theme, icon_space);
            }
        }
    }
}

#[cfg(feature = "accelerator")]
fn collect_accelerators(items: &Vec<MenuItem>, accelerators: &mut HashMap<u16, String>) {
    for item in items {
        if item.menu_item_type == MenuItemType::Submenu {
            let submenu_window_handle = item.submenu.as_ref().unwrap().window_handle;
            let data = get_menu_data(submenu_window_handle);
            collect_accelerators(&data.items, accelerators);
        } else if !item.accelerator.is_empty() {
            accelerators.insert(item.uuid, item.accelerator.clone());
        }
    }
}
