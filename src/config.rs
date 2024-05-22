#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Dark,
    Light,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub theme: Theme,
    pub size: MenuSize,
    pub color: ThemeColor,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: Theme::Light,
            size: MenuSize::default(),
            color: ThemeColor::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MenuSize {
    pub border_width: i32,
    pub vertical_margin: i32,
    pub horizontal_margin: i32,
    pub item_vertical_padding: i32,
    pub item_horizontal_padding: i32,
    pub font_size: Option<i32>,
    pub font_weight: Option<i32>,
}

impl Default for MenuSize {
    fn default() -> Self {
        Self {
            border_width: 1,
            vertical_margin: 2,
            horizontal_margin: 0,
            item_vertical_padding: 12,
            item_horizontal_padding: 10,
            font_size: None,
            font_weight: None,
        }
    }
}

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

#[derive(Debug, Clone)]
pub struct ColorScheme {
    pub color: u32,
    pub border: u32,
    pub disabled: u32,
    pub background_color: u32,
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
