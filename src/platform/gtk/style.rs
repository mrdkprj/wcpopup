use super::{Config, Theme};
use crate::{config::to_rgba_string, platform::platform_impl::to_font_weight, Corner, RgbaIcon, SvgData};
use std::path::Path;

const CORNER_RADIUS: i32 = 8;

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
            window#{WIDGET_NAME},
            window#{DARK_WIDGET_NAME},
            window#{LIGHT_WIDGET_NAME},
            window#{WIDGET_NAME} decoration,
            window#{DARK_WIDGET_NAME} decoration,
            window#{LIGHT_WIDGET_NAME} decoration
            {{
                border-radius: {}px;
            }}

            window#{DARK_WIDGET_NAME} {{
                background: {};
            }}

            window#{LIGHT_WIDGET_NAME} {{
                background: {};
            }}

            window#{WIDGET_NAME} decoration,
            window#{DARK_WIDGET_NAME} decoration,
            window#{LIGHT_WIDGET_NAME} decoration {{
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
        /* system */
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
        /* dark */
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
        /* light */
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
    let horizonta_padding = if config.size.item_horizontal_padding > 0 {
        format!(
            r#"
                padding-right: {}px;
                padding-left: {}px;
            "#,
            config.size.item_horizontal_padding, config.size.item_horizontal_padding,
        )
    } else {
        String::new()
    };

    let weight = to_font_weight(config.font.dark_font_weight);

    let font_size = config.font.dark_font_size.max(config.font.light_font_size);

    let check = if let Some(svg) = &config.icon.as_ref().unwrap().check_svg {
        format!(
            r#"
                -gtk-icon-source: -gtk-recolor(url("{}"));
                min-width: {}px;
                min-height: {}px;
            "#,
            svg.path.to_string_lossy(),
            svg.width,
            svg.height,
        )
    } else {
        format!(
            r#"
                min-width: {font_size}px;
                min-height: {font_size}px;
            "#,
        )
    };

    let arrow = if let Some(svg) = &config.icon.as_ref().unwrap().arrow_svg {
        format!(
            r#"
                -gtk-icon-source: -gtk-recolor(url("{}"));
                min-width: {}px;
                min-height: {}px;
            "#,
            svg.path.to_string_lossy(),
            svg.width,
            svg.height,
        )
    } else {
        format!(
            r#"
                min-width: {font_size}px;
                min-height: {font_size}px;
            "#,
        )
    };

    format!(
        r#"
            menuitem#{WIDGET_NAME} accelerator,
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

            menuitem#{WIDGET_NAME} check,
            menuitem#{DARK_WIDGET_NAME} check,
            menuitem#{LIGHT_WIDGET_NAME} check {{
                border-width: 0px;
                outline-width: 0px;
            }}
            menuitem#{DARK_WIDGET_NAME} check:checked,
            menuitem#{LIGHT_WIDGET_NAME} check:checked{{
                {}
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

            menuitem#{DARK_WIDGET_NAME} arrow,
            menuitem#{LIGHT_WIDGET_NAME} arrow {{
                {}
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
                {}
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

            separator#{WIDGET_NAME},
            separator#{DARK_WIDGET_NAME},
            separator#{LIGHT_WIDGET_NAME} {{
                padding-left: {}px;
                padding-right: {}px;
                margin-top: {}px;
                margin-bottom: {}px;
                min-height: {}px;
            }}
            separator#{DARK_WIDGET_NAME} {{
                background-color: {};
            }}
            separator#{LIGHT_WIDGET_NAME} {{
                background-color: {};
            }}
        "#,
        /* accelerator */
        config.font.dark_font_size,
        config.font.font_family,
        weight,
        to_rgba_string(config.color.dark.accelerator),
        to_rgba_string(config.color.light.accelerator),
        /* check */
        check,
        to_rgba_string(config.color.dark.color),
        to_rgba_string(config.color.dark.background_color),
        to_rgba_string(config.color.light.color),
        to_rgba_string(config.color.light.background_color),
        /* check hover */
        to_rgba_string(config.color.dark.hover_background_color),
        to_rgba_string(config.color.light.hover_background_color),
        /* arrow */
        arrow,
        to_rgba_string(config.color.dark.color),
        to_rgba_string(config.color.dark.background_color),
        to_rgba_string(config.color.light.color),
        to_rgba_string(config.color.light.background_color),
        /* arrow hover */
        to_rgba_string(config.color.dark.hover_background_color),
        to_rgba_string(config.color.light.hover_background_color),
        /* item */
        horizonta_padding,
        config.size.item_vertical_padding,
        config.size.item_vertical_padding,
        to_rgba_string(config.color.dark.color),
        to_rgba_string(config.color.light.color),
        to_rgba_string(config.color.dark.hover_background_color),
        to_rgba_string(config.color.light.hover_background_color),
        to_rgba_string(config.color.dark.disabled),
        to_rgba_string(config.color.light.disabled),
        /* separator */
        config.size.item_horizontal_padding,
        config.size.item_horizontal_padding,
        config.size.item_vertical_padding,
        config.size.item_vertical_padding,
        config.size.separator_size,
        to_rgba_string(config.color.dark.separator),
        to_rgba_string(config.color.light.separator),
    )
}

pub(crate) fn get_icon_menu_css(icon: &Path, config: &Config) -> String {
    let url = icon.to_string_lossy();
    let (width, height) = if let Some(svg) = &config.icon.as_ref().unwrap().check_svg {
        (svg.width as f32, svg.height as f32)
    } else {
        let font_size = config.font.dark_font_size.max(config.font.light_font_size);
        (font_size, font_size)
    };

    if let Some(margin) = config.icon.as_ref().unwrap().horizontal_margin {
        format!(
            r#"
                menuitem image {{
                    background-image:-gtk-recolor(url("{url}"));
                    background-repeat: no-repeat;
                    background-size: contain;
                    background-position: center;
                    margin-left: {margin}px;
                    margin-right: {margin}px;
                    min-width: {width}px;
                    min-height: {height}px;
                }}
            "#
        )
    } else {
        format!(
            r#"
                menuitem image {{
                    background-image:-gtk-recolor(url("{url}"));
                    background-repeat: no-repeat;
                    background-size: contain;
                    background-position: center;
                    min-width: {width}px;
                    min-height: {height}px;
                }}
            "#
        )
    }
}

pub(crate) fn get_rgba_icon_menu_css(rgba_icon: &RgbaIcon, config: &Config) -> String {
    let width = rgba_icon.width;
    let height = rgba_icon.height;
    if let Some(margin) = config.icon.as_ref().unwrap().horizontal_margin {
        format!(
            r#"
                menuitem image {{
                    margin-left: {margin}px;
                    margin-right: {margin}px;
                    min-width: {width}px;
                    min-height: {height}px;
                }}
            "#
        )
    } else {
        format!(
            r#"
                menuitem image {{
                    min-width: {width}px;
                    min-height: {height}px;
                }}
            "#
        )
    }
}

pub(crate) fn get_svg_icon_menu_css(svg: &SvgData, config: &Config) -> String {
    let width = svg.width;
    let height = svg.height;
    if let Some(margin) = config.icon.as_ref().unwrap().horizontal_margin {
        format!(
            r#"
                menuitem image {{
                    margin-left: {margin}px;
                    margin-right: {margin}px;
                    min-width: {width}px;
                    min-height: {height}px;
                }}
            "#
        )
    } else {
        format!(
            r#"
                menuitem image {{
                    min-width: {width}px;
                    min-height: {height}px;
                }}
            "#
        )
    }
}

pub(crate) fn get_hidden_image_css(config: &Config) -> String {
    if let Some(margin) = config.icon.as_ref().unwrap().horizontal_margin {
        format!(
            r#"
                menuitem image {{
                    margin-left: {margin}px;
                    margin-right: {margin}px;
                }}
            "#
        )
    } else {
        String::new()
    }
}
