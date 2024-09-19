use crate::{config::to_rgba_string, platform::platform_impl::to_font_weight, Corner};

use super::{Config, Theme};

const CORNER_RADIUS: i32 = 8;
const SEPARATOR_MARGIN: i32 = 5;

const WIDGET_NAME: &str = "wcpopup";
const DARK_WIDGET_NAME: &str = "wcpopup-dark";
const LIGHT_WIDGET_NAME: &str = "wcpopup-light";

pub(crate) fn get_widget_name<'a>(theme: Theme) -> &'a str {
    match theme {
        Theme::Dark => DARK_WIDGET_NAME,
        Theme::Light => LIGHT_WIDGET_NAME,
        Theme::System => WIDGET_NAME,
    }
}

pub(crate) fn get_window_css(config: &Config) -> String {
    format!(
        r#"
            * {{
                border-radius: {}px;
            }}

            *#{DARK_WIDGET_NAME} {{
                background: {};
            }}

            *{LIGHT_WIDGET_NAME} {{
                background: {};
            }}

            decoration {{
                box-shadow: 5px 5px 5px 5px rgba(0, 0, 0, 0.2);
            }}
        "#,
        if config.corner == Corner::Round {
            CORNER_RADIUS
        } else {
            0
        },
        to_rgba_string(config.color.dark.background_color),
        to_rgba_string(config.color.light.background_color),
    )
}

pub(crate) fn get_menu_css(config: &Config) -> String {
    let dark_weight = to_font_weight(config.font.dark_font_weight);
    let light_weight = to_font_weight(config.font.light_font_weight);

    let vertical_padding = if config.corner == Corner::Round && config.size.vertical_padding < CORNER_RADIUS {
        CORNER_RADIUS
    } else {
        config.size.vertical_padding
    };

    format!(
        r#"
            menu#{WIDGET_NAME},
            menu#{DARK_WIDGET_NAME},
            menu#{LIGHT_WIDGET_NAME} {{
                padding-left: {}px;
                padding-right: {}px;
                padding-top: {}px;
                padding-bottom: {}px;
                border: {}px solid;
                border-radius: {}px;
            }}

            menu#{DARK_WIDGET_NAME} {{
                color: {};
                background-color: {};
                font: {}px "{}";
                font-weight: {};
                border-color:{};
            }}

            menu#{LIGHT_WIDGET_NAME} {{
                color: {};
                background-color: {};
                font: {}px "{}";
                font-weight: {};
                border-color:{};
            }}
        "#,
        // system
        config.size.horizontal_padding,
        config.size.horizontal_padding,
        vertical_padding,
        vertical_padding,
        if config.size.border_size > 0 {
            config.size.border_size
        } else {
            0
        },
        if config.corner == Corner::Round {
            CORNER_RADIUS
        } else {
            0
        },
        // dark
        to_rgba_string(config.color.dark.color),
        to_rgba_string(config.color.dark.background_color),
        config.font.dark_font_size,
        config.font.font_family,
        dark_weight,
        if config.size.border_size > 0 {
            to_rgba_string(config.color.dark.border)
        } else {
            to_rgba_string(config.color.dark.background_color)
        },
        // light
        to_rgba_string(config.color.light.color),
        to_rgba_string(config.color.light.background_color),
        config.font.light_font_size,
        config.font.font_family,
        light_weight,
        if config.size.border_size > 0 {
            to_rgba_string(config.color.light.border)
        } else {
            to_rgba_string(config.color.light.background_color)
        },
    )
}

pub(crate) fn get_menu_item_css(config: &Config) -> String {
    let weight = to_font_weight(config.font.dark_font_weight);

    format!(
        r#"
            menuitem#{DARK_WIDGET_NAME} accelerator,
            menuitem#{LIGHT_WIDGET_NAME} accelerator {{
                font: {}px "{}";
                font-weight: {};
            }}
            menuitem#{DARK_WIDGET_NAME} accelerator {{
                color: {};
            }}
            menuitem#{LIGHT_WIDGET_NAME} accelerator {{
                color: {};
            }}

            separator#{WIDGET_NAME},
            separator#{DARK_WIDGET_NAME},
            separator#{LIGHT_WIDGET_NAME} {{
                margin-top: {SEPARATOR_MARGIN}px;
                margin-bottom: {SEPARATOR_MARGIN}px;
            }}
            separator#{DARK_WIDGET_NAME} {{
                background-color: {};
            }}
            separator#{LIGHT_WIDGET_NAME} {{
                background-color: {};
            }}

            menuitem#{DARK_WIDGET_NAME} check,
            menuitem#{LIGHT_WIDGET_NAME} check {{
                border-width: 0px;
                outline-width: 0px;
            }}
            menuitem#{DARK_WIDGET_NAME} check {{
                color: {};
                background-color: {};
            }}
            menuitem#{LIGHT_WIDGET_NAME} check {{
                color: {};
                background-color: {};
            }}
            menuitem#{DARK_WIDGET_NAME} check:hover {{
                background-color: {};
            }}
            menuitem#{LIGHT_WIDGET_NAME} check:hover {{
                background-color: {};
            }}

            menu#{DARK_WIDGET_NAME} menuitem#{DARK_WIDGET_NAME} arrow {{
                color: {};
                background-color: {};
            }}
            menu#{LIGHT_WIDGET_NAME} menuitem#{LIGHT_WIDGET_NAME} arrow {{
                color: {};
                background-color: {};
            }}
            menu#{DARK_WIDGET_NAME} menuitem#{DARK_WIDGET_NAME}:hover arrow {{
                background-color: {};
            }}
            menu#{LIGHT_WIDGET_NAME} menuitem#{LIGHT_WIDGET_NAME}:hover arrow {{
                background-color: {};
            }}

            menuitem#{WIDGET_NAME},
            menuitem#{DARK_WIDGET_NAME},
            menuitem#{LIGHT_WIDGET_NAME} {{
                padding-left: {}px;
                padding-right: {}px;
                padding-top: {}px;
                padding-bottom: {}px;
                border: none;
            }}
            menuitem#{DARK_WIDGET_NAME} {{
                color: {};
            }}
            menuitem#{LIGHT_WIDGET_NAME} {{
                color: {};
            }}
            menuitem#{DARK_WIDGET_NAME}:hover {{
                background-color: {};
            }}
            menuitem#{LIGHT_WIDGET_NAME}:hover {{
                background-color: {};
            }}

            menuitem#{DARK_WIDGET_NAME}:disabled, menu#{DARK_WIDGET_NAME} menuitem#{DARK_WIDGET_NAME}:disabled check, menu#{DARK_WIDGET_NAME} menuitem#{DARK_WIDGET_NAME}:disabled arrow {{
                color: {};
            }}
            menuitem#{LIGHT_WIDGET_NAME}:disabled, menu#{LIGHT_WIDGET_NAME} menuitem#{LIGHT_WIDGET_NAME}:disabled check, menu#{LIGHT_WIDGET_NAME} menuitem#{LIGHT_WIDGET_NAME}:disabled arrow {{
                color: {};
            }}
        "#,
        // accelerator
        config.font.dark_font_size,
        config.font.font_family,
        weight,
        to_rgba_string(config.color.dark.accelerator),
        to_rgba_string(config.color.light.accelerator),
        // separator
        to_rgba_string(config.color.dark.separator),
        to_rgba_string(config.color.light.separator),
        // check
        to_rgba_string(config.color.dark.color),
        to_rgba_string(config.color.dark.background_color),
        to_rgba_string(config.color.light.color),
        to_rgba_string(config.color.light.background_color),
        // check hover
        to_rgba_string(config.color.dark.hover_background_color),
        to_rgba_string(config.color.light.hover_background_color),
        // arrow
        to_rgba_string(config.color.dark.color),
        to_rgba_string(config.color.dark.background_color),
        to_rgba_string(config.color.light.color),
        to_rgba_string(config.color.light.background_color),
        // arrow hover
        to_rgba_string(config.color.dark.hover_background_color),
        to_rgba_string(config.color.light.hover_background_color),
        // item
        config.size.item_horizontal_padding,
        config.size.item_horizontal_padding,
        config.size.item_vertical_padding,
        config.size.item_vertical_padding,
        to_rgba_string(config.color.dark.color),
        to_rgba_string(config.color.light.color),
        to_rgba_string(config.color.dark.hover_background_color),
        to_rgba_string(config.color.light.hover_background_color),
        to_rgba_string(config.color.dark.disabled),
        to_rgba_string(config.color.light.disabled),
    )
}
