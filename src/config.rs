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
    pub font: MenuFont,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: Theme::Light,
            size: MenuSize::default(),
            color: ThemeColor::default(),
            corner: Corner::Round,
            font: MenuFont::default(),
        }
    }
}

/// Size settings of Menu.
///
/// ## Default.
///
///  ```no_run
///   border_size: 0,
///   vertical_padding: 0,
///   horizontal_padding: 0,
///   item_vertical_padding: 8,
///   item_horizontal_padding: 0,
///   submenu_offset: -3
///  ```
#[derive(Debug, Clone)]
pub struct MenuSize {
    /// Border width and height.
    pub border_size: i32,
    /// Top and bottom padding of Menu.
    pub vertical_padding: i32,
    /// Left and right padding of Menu.
    pub horizontal_padding: i32,
    /// Top and bottom padding of MenuItem.
    pub item_vertical_padding: i32,
    /// Left and right padding of MenuItem.
    pub item_horizontal_padding: i32,
    /// Submenu position relative to Menu.
    pub submenu_offset: i32,
}

impl Default for MenuSize {
    fn default() -> Self {
        Self {
            border_size: 0,
            vertical_padding: 0,
            horizontal_padding: 0,
            item_vertical_padding: 8,
            item_horizontal_padding: 0,
            submenu_offset: -3,
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
///   color: 0xe7e0e0,
///   accelerator: 0xc5c1c1,
///   border: 0x454545,
///   separator: 0x454545,
///   disabled: 0x565659,
///   background_color: 0x252526,
///   hover_background_color: 0x3b3a3a,
/// };
/// ```
///
/// ## Default colors for Light Theme.
/// ```no_run
/// const DEFAULT_LIGHT_COLOR_SCHEME: ColorScheme = ColorScheme {
///   color: 0x494747,
///   accelerator: 0x635e5e,
///   border: 0xe9e2e2,
///   separator: 0xe9e2e2,
///   disabled: 0xc5c1c1,
///   background_color: 0xFFFFFF,
///   hover_background_color: 0xefefef,
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
    color: 0xe7e0e0,
    accelerator: 0xc5c1c1,
    border: 0x454545,
    separator: 0x454545,
    disabled: 0x565659,
    background_color: 0x252526,
    hover_background_color: 0x3b3a3a,
};

pub const DEFAULT_LIGHT_COLOR_SCHEME: ColorScheme = ColorScheme {
    color: 0x494747,
    accelerator: 0x635e5e,
    border: 0xe9e2e2,
    separator: 0xe9e2e2,
    disabled: 0xc5c1c1,
    background_color: 0xFFFFFF,
    hover_background_color: 0xefefef,
};

#[derive(Debug, Clone)]
/// Font settings of Menu.
///
/// ## Default.
///
///  ```no_run
///   font_family: "Segoe UI",
///   dark_font_size: 12.0,
///   light_font_size: 12.0,
///   dark_font_weight: Normal,
///   light_font_weight: Normal,
///  ```
pub struct MenuFont {
    /// Font family.
    pub font_family: String,
    /// Font size for Dark theme.
    pub dark_font_size: f32,
    /// Font size for Light theme.
    pub light_font_size: f32,
    /// Font weight for Dark theme.
    pub dark_font_weight: FontWeight,
    /// Font weight for Light theme.
    pub light_font_weight: FontWeight,
}

impl Default for MenuFont {
    fn default() -> Self {
        Self {
            font_family: String::from("Segoe UI"),
            dark_font_size: 12.0,
            light_font_size: 12.0,
            dark_font_weight: FontWeight::Normal,
            light_font_weight: FontWeight::Normal,
        }
    }
}

/// Font weight.
///  ```no_run
///   Thin: 100,
///   Light: 300,
///   Normal: 400,
///   Medium: 500,
///   Bold: 700,
///  ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontWeight {
    Thin,
    Light,
    Normal,
    Medium,
    Bold,
}

/// Creates RGB color.
pub fn rgb(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) << 16 | (g as u32) << 8 | (b as u32)
}
