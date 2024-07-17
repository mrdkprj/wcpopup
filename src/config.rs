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
///   font_size: None,
///   font_weight: None,
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
    /// Font size
    pub font_size: Option<i32>,
    /// Font weight
    pub font_weight: Option<i32>,
}

impl Default for MenuSize {
    fn default() -> Self {
        Self {
            border_size: 1,
            vertical_margin: 2,
            horizontal_margin: 0,
            item_vertical_padding: 12,
            item_horizontal_padding: 10,
            font_size: None,
            font_weight: None,
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
            dark: DARK_COLOR_SCHEME,
            light: LIGHT_COLOR_SCHEME,
        }
    }
}

/// Menu color settings for text, border and background.
///
/// ## Default colors for Dark Theme
///
/// ```no_run
/// const DARK_COLOR_SCHEME: ColorScheme = ColorScheme {
///   color: 0x0e7e0e0,
///   border: 0x0454545,
///   disabled: 0x00565659,
///   background_color: 0x0252526,
///   hover_background_color: 0x0454545,
/// };
/// ```
///
/// ## Default colors for Light Theme
/// ```no_run
/// const LIGHT_COLOR_SCHEME: ColorScheme = ColorScheme {
///   color: 0x00e0e0e,
///   border: 0x0e9e2e2,
///   disabled: 0x00565659,
///   background_color: 0x00f5f5f5,
///   hover_background_color: 0x0e9e2e2,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ColorScheme {
    /// MenuItem text color.
    pub color: u32,
    /// Menu border and separator color.
    pub border: u32,
    /// Disabled MenuItem color.
    pub disabled: u32,
    /// Menu background color.
    pub background_color: u32,
    /// MenuItem hover color.
    pub hover_background_color: u32,
}

const DARK_COLOR_SCHEME: ColorScheme = ColorScheme {
    color: 0x0e7e0e0,
    border: 0x0454545,
    disabled: 0x00565659,
    background_color: 0x0252526,
    hover_background_color: 0x0454545,
};

const LIGHT_COLOR_SCHEME: ColorScheme = ColorScheme {
    color: 0x00e0e0e,
    border: 0x0e9e2e2,
    disabled: 0x00565659,
    background_color: 0x00f5f5f5,
    hover_background_color: 0x0e9e2e2,
};

/// Creates RGB color.
pub fn rgb(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) << 16 | (g as u32) << 8 | (b as u32)
}
