#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Dark,
    Light,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Corner {
    Round,
    DoNotRound,
}

/// Menu configuration for Theme, Size and Color.
#[derive(Debug, Clone)]
pub struct Config {
    pub theme: Theme,
    pub size: MenuSize,
    pub color: ThemeColor,
    /// Effective starting with Windows 11 Build 22000
    pub corner: Corner,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: Theme::Light,
            size: MenuSize::default(),
            color: ThemeColor::default(),
            corner: Corner::Round,
        }
    }
}

/// Size settings of Menu.
///
/// ## Default.
///
///  ```no_run
///   border_size: 1,
///   vertical_margin: 2,
///   horizontal_margin: 0,
///   item_vertical_padding: 12,
///   item_horizontal_padding: 10,
///   submenu_offset: -3
///   dark_font_size: None,
///   light_font_size: None,
///   dark_font_weight: Some(700),
///   light_font_weight: None,
///  ```
#[derive(Debug, Clone)]
pub struct MenuSize {
    /// Border width and height.
    pub border_size: i32,
    /// Top and bottom margin of a Menu.
    pub vertical_margin: i32,
    /// Left and right margin of a Menu.
    pub horizontal_margin: i32,
    /// Top and bottom padding of a MenuItem.
    pub item_vertical_padding: i32,
    /// Left and right padding of a MenuItem.
    pub item_horizontal_padding: i32,
    /// Submenu position relative to the menu.
    pub submenu_offset: i32,
    /// Font size for Dark theme.
    pub dark_font_size: Option<i32>,
    /// Font size for Light theme.
    pub light_font_size: Option<i32>,
    /// Font weight for Dark theme. Default is 700(bold).
    pub dark_font_weight: Option<i32>,
    /// Font weight for Light theme.
    pub light_font_weight: Option<i32>,
}

impl Default for MenuSize {
    fn default() -> Self {
        Self {
            border_size: 1,
            vertical_margin: 2,
            horizontal_margin: 0,
            item_vertical_padding: 12,
            item_horizontal_padding: 10,
            submenu_offset: -3,
            dark_font_size: None,
            light_font_size: None,
            dark_font_weight: Some(700),
            light_font_weight: None,
        }
    }
}

/// Color settings for Dark and Light Theme.
#[derive(Debug, Clone)]
pub struct ThemeColor {
    pub dark: ColorScheme,
    pub light: ColorScheme,
}

impl Default for ThemeColor {
    fn default() -> Self {
        Self {
            dark: DEFAULT_DARK_COLOR_SCHEME,
            light: DEFAULT_LIGHT_COLOR_SCHEME,
        }
    }
}

/// Menu color settings for text, accelerator, border and background.
///
/// ## Default colors for Dark Theme.
///
/// ```no_run
/// const DEFAULT_DARK_COLOR_SCHEME: ColorScheme = ColorScheme {
///   color: 0x00e7e0e0,
///   accelerator: 0x00c5c1c1,
///   border: 0x00454545,
///   separator: 0x00454545,
///   disabled: 0x00565659,
///   background_color: 0x00252526,
///   hover_background_color: 0x003b3a3a,
/// };
/// ```
///
/// ## Default colors for Light Theme.
/// ```no_run
/// const DEFAULT_LIGHT_COLOR_SCHEME: ColorScheme = ColorScheme {
///   color: 0x00e0e0e,
///   accelerator: 0x00635e5e,
///   border: 0x00e9e2e2,
///   separator: 0x00e9e2e2,
///   disabled: 0x00c5c1c1,
///   background_color: 0x00f5f5f5,
///   hover_background_color: 0x00e9e2e2,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ColorScheme {
    /// MenuItem text color.
    pub color: u32,
    /// MenuItem accelerator text color.
    pub accelerator: u32,
    /// Menu border color.
    pub border: u32,
    /// Menu separator color.
    pub separator: u32,
    /// Disabled MenuItem color.
    pub disabled: u32,
    /// Menu background color.
    pub background_color: u32,
    /// MenuItem hover color.
    pub hover_background_color: u32,
}

pub const DEFAULT_DARK_COLOR_SCHEME: ColorScheme = ColorScheme {
    color: 0x00e7e0e0,
    accelerator: 0x00c5c1c1,
    border: 0x00454545,
    separator: 0x00454545,
    disabled: 0x000565659,
    background_color: 0x00252526,
    hover_background_color: 0x003b3a3a,
};

pub const DEFAULT_LIGHT_COLOR_SCHEME: ColorScheme = ColorScheme {
    color: 0x000e0e0e,
    accelerator: 0x00635e5e,
    border: 0x00e9e2e2,
    separator: 0x00e9e2e2,
    disabled: 0x00c5c1c1,
    background_color: 0x00f5f5f5,
    hover_background_color: 0x00e9e2e2,
};

/// Creates RGB color.
pub fn rgb(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) << 16 | (g as u32) << 8 | (b as u32)
}
