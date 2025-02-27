use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Dark,
    Light,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Corner {
    Round,
    DoNotRound,
}

/// Menu configuration for Theme, Size and Color.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub theme: Theme,
    pub size: MenuSize,
    pub color: ThemeColor,
    /// On Windows, effective starting with Windows 11 Build 22000.
    pub corner: Corner,
    pub font: MenuFont,
    pub icon: Option<IconSettings>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: Theme::Light,
            size: MenuSize::default(),
            color: ThemeColor::default(),
            corner: Corner::Round,
            font: MenuFont::default(),
            icon: Some(IconSettings::default()),
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
///   separator_size: 1,
///  ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuSize {
    /// Border width and height.
    pub border_size: i32,
    /// Top and bottom paddings of Menu.
    pub vertical_padding: i32,
    /// Left and right paddings of Menu.
    pub horizontal_padding: i32,
    /// Top and bottom paddings of MenuItem.
    pub item_vertical_padding: i32,
    /// Left and right paddings of MenuItem.
    pub item_horizontal_padding: i32,
    /// Submenu position relative to Menu.
    pub submenu_offset: i32,
    /// Separator height(stroke width).
    pub separator_size: i32,
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
            separator_size: 1,
        }
    }
}

/// Color settings for Dark and Light Theme.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// ## Default colors for Dark Theme.
///
/// ```no_run
/// const DEFAULT_DARK_COLOR_SCHEME: ColorScheme = ColorScheme {
///   color: 0xe7e0e0,
///   accelerator: 0xe7e0e08c,
///   border: 0x454545,
///   separator: 0x454545,
///   disabled: 0x565659,
///   background_color: 0x252526,
///   hover_background_color: 0x3b3a3a,
/// };
pub const DEFAULT_DARK_COLOR_SCHEME: ColorScheme = ColorScheme {
    color: 0xe7e0e0,
    accelerator: 0xe7e0e08c,
    border: 0x454545,
    separator: 0x454545,
    disabled: 0x565659,
    background_color: 0x252526,
    hover_background_color: 0x3b3a3a,
};

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
pub const DEFAULT_LIGHT_COLOR_SCHEME: ColorScheme = ColorScheme {
    color: 0x494747,
    accelerator: 0x4947478c,
    border: 0xe9e2e2,
    separator: 0xe9e2e2,
    disabled: 0xc5c1c1,
    background_color: 0xFFFFFF,
    hover_background_color: 0xefefef,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FontWeight {
    Thin,
    Light,
    Normal,
    Medium,
    Bold,
}

/// Icon settings.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct IconSettings {
    /// SVG to override the default check-mark SVG for check/radio menu item.
    pub check_svg: Option<MenuSVG>,
    /// SVG to override the default arrow SVG for submenu item.
    pub arrow_svg: Option<MenuSVG>,
    /// Whether to reserve space for icons regardless of their actual presence.
    pub reserve_icon_size: bool,
    /// Left and right margins of the icons set to menu items.
    pub horizontal_margin: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuSVG {
    pub path: PathBuf,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug)]
pub struct RGBA {
    pub r: u32,
    pub g: u32,
    pub b: u32,
    pub a: f32,
}

#[cfg(target_os = "windows")]
pub(crate) fn to_hex_string(color: u32) -> String {
    if has_alpha(color) {
        format!("#{:08x}", color)
    } else {
        format!("#{:06x}", color & 0xFFFFFF)
    }
}

#[cfg(target_os = "linux")]
pub(crate) fn to_rgba_string(color: u32) -> String {
    let rgba = rgba_from_hex(color);
    format!("rgba({}, {}, {}, {:.2})", rgba.r, rgba.g, rgba.b, rgba.a)
}

/// RGBA from hex value.
pub fn rgba_from_hex(color: u32) -> RGBA {
    if has_alpha(color) {
        let r = (color >> 24) & 0xFF; /* Shift right by 24 bits, mask the last 8 bits */
        let g = (color >> 16) & 0xFF; /* Shift right by 16 bits, mask the last 8 bits */
        let b = (color >> 8) & 0xFF; /* Shift right by 8 bits, mask the last 8 bits */
        let a = (color & 0xFF) as f32 / 255.0; /* Extract alpha and normalize to [0.0, 1.0] */
        RGBA {
            r,
            g,
            b,
            a,
        }
    } else {
        let r = (color >> 16) & 0xFF; /* Shift right by 16 bits, mask the last 8 bits */
        let g = (color >> 8) & 0xFF; /* Shift right by 8 bits, mask the last 8 bits */
        let b = color & 0xFF;
        RGBA {
            r,
            g,
            b,
            a: 1.0,
        }
    }
}

fn has_alpha(value: u32) -> bool {
    /* If the value is larger than 24 bits, it contains alpha */
    value > 0xFFFFFF
}

/// Hex value from RGB.
pub fn hex_from_rgb(r: u32, g: u32, b: u32) -> u32 {
    r << 16 | g << 8 | b
}

/// Hex value from RGBA.
pub fn hex_from_rgba(r: u32, g: u32, b: u32, a: f32) -> u32 {
    let alpha = (a * 255.0).round() as u32;
    r << 24 | g << 16 | b << 8 | alpha
}
