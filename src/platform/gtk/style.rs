use super::{
    to_font_weight,
    util::{get_custom_check_width, is_svg},
};
use crate::{
    config::{to_rgba_string, Config, Theme},
    Corner, DataIcon, MenuIconKind, PathIcon, SvgIcon,
};

const CORNER_RADIUS: i32 = 8;

const WIDGET_NAME: &str = "wcpopup";
const DARK_WIDGET_NAME: &str = "wcpopup-dark";
const LIGHT_WIDGET_NAME: &str = "wcpopup-light";
pub(crate) const CUSTOM_CHECKMARK_NAME: &str = "wcpopup-check";

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

    let check = if let Some(check) = &config.icon.as_ref().unwrap().check {
        match &check.icon {
            MenuIconKind::Path(icon) => {
                format!(
                    r#"
                        -gtk-icon-source:none;
                        min-width: {}px;
                        min-height: {}px;
                        padding:0;
                        margin:0;
                        border:none;
                        outline:none;
                        background-image:none;
                        opacity:0;
                    "#,
                    icon.width, icon.height,
                )
            }
            _ => "
                -gtk-icon-source:none;
                min-width: 0px;
                min-height: 0px;
                padding:0;
                margin:0;
                border:none;
                outline:none;
                background-image:none;
                opacity:0;
                "
            .to_string(),
        }
    } else {
        format!(
            r#"
                min-width: {font_size}px;
                min-height: {font_size}px;
            "#,
        )
    };

    let checked = if let Some(check) = &config.icon.as_ref().unwrap().check {
        match &check.icon {
            MenuIconKind::Path(icon) => {
                if is_svg(&icon.path) {
                    format!(
                        r#"
                            -gtk-icon-source: -gtk-recolor(url("{}"));
                        "#,
                        icon.path.display(),
                    )
                } else {
                    format!(
                        r#"
                            -gtk-icon-source: url("{}");
                        "#,
                        icon.path.display(),
                    )
                }
            }
            _ => String::new(),
        }
    } else {
        String::new()
    };

    let arrow = if let Some(arrow) = &config.icon.as_ref().unwrap().arrow {
        match &arrow.icon {
            MenuIconKind::Path(icon) => {
                if is_svg(&icon.path) {
                    format!(
                        r#"
                            -gtk-icon-source: -gtk-recolor(url("{}"));
                            min-width: {}px;
                            min-height: {}px;
                        "#,
                        icon.path.display(),
                        icon.width,
                        icon.height,
                    )
                } else {
                    format!(
                        r#"
                            -gtk-icon-source: url("{}");
                            min-width: {}px;
                            min-height: {}px;
                        "#,
                        icon.path.display(),
                        icon.width,
                        icon.height,
                    )
                }
            }
            _ => "
                    -gtk-icon-source:none;
                    min-width: 0px;
                    min-height: 0px;
                    padding:0;
                    margin:0;
                    border:none;
                    outline:none;
                    background-image:none;
                    opacity:0;
                "
            .to_string(),
        }
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
            #{WIDGET_NAME} accelerator,
            #{DARK_WIDGET_NAME} accelerator,
            #{LIGHT_WIDGET_NAME} accelerator {{
                font: {}px "{}";
                font-weight: {};
            }}
            #{DARK_WIDGET_NAME} accelerator {{
                color: {};
            }}
            #{LIGHT_WIDGET_NAME} accelerator {{
                color: {};
            }}

            #{WIDGET_NAME} check,
            #{DARK_WIDGET_NAME} check,
            #{LIGHT_WIDGET_NAME} check {{
                border-width: 0px;
                outline-width: 0px;
            }}
            #{WIDGET_NAME} check:not(:checked)+box image#{CUSTOM_CHECKMARK_NAME}:first-child,
            #{DARK_WIDGET_NAME} check:not(:checked)+box image#{CUSTOM_CHECKMARK_NAME}:first-child,
            #{LIGHT_WIDGET_NAME} check:not(:checked)+box image#{CUSTOM_CHECKMARK_NAME}:first-child{{
                opacity:0;
            }}
            #{WIDGET_NAME} check:checked+box image#{CUSTOM_CHECKMARK_NAME}:first-child,
            #{DARK_WIDGET_NAME} check:checked+box image#{CUSTOM_CHECKMARK_NAME}:first-child,
            #{LIGHT_WIDGET_NAME} check:checked+box image#{CUSTOM_CHECKMARK_NAME}:first-child{{
                opacity:1;
            }}
            #{WIDGET_NAME} check,
            #{DARK_WIDGET_NAME} check,
            #{LIGHT_WIDGET_NAME} check{{
                {}
            }}
            #{WIDGET_NAME} check:checked,
            #{DARK_WIDGET_NAME} check:checked,
            #{LIGHT_WIDGET_NAME} check:checked{{
                {}
            }}
            #{DARK_WIDGET_NAME} check,
            #{DARK_WIDGET_NAME} check:checked {{
                color: {};
                background-color: {};
            }}
            #{LIGHT_WIDGET_NAME} check,
            #{LIGHT_WIDGET_NAME} check:checked {{
                color: {};
                background-color: {};
            }}
            #{DARK_WIDGET_NAME} check:hover,
            #{DARK_WIDGET_NAME} check:checked:hover {{
                background-color: {};
            }}
            #{LIGHT_WIDGET_NAME} check:hover,
            #{LIGHT_WIDGET_NAME} check:checked:hover {{
                background-color: {};
            }}

            #{DARK_WIDGET_NAME} arrow,
            #{LIGHT_WIDGET_NAME} arrow {{
                {}
            }}
            #{DARK_WIDGET_NAME} arrow {{
                color: {};
                background-color: {};
            }}
            #{LIGHT_WIDGET_NAME} arrow {{
                color: {};
                background-color: {};
            }}
            #{DARK_WIDGET_NAME}:hover arrow {{
                background-color: {};
            }}
            #{LIGHT_WIDGET_NAME}:hover arrow {{
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
            #{WIDGET_NAME} label,
            #{DARK_WIDGET_NAME} label,
            #{LIGHT_WIDGET_NAME} label{{
                {}
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
        /* checked */
        checked,
        /* check color */
        to_rgba_string(config.color.dark.color),
        to_rgba_string(config.color.dark.background_color),
        to_rgba_string(config.color.light.color),
        to_rgba_string(config.color.light.background_color),
        /* check hover */
        to_rgba_string(config.color.dark.hover_background_color),
        to_rgba_string(config.color.light.hover_background_color),
        /* arrow */
        arrow,
        /* arrow color */
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
        /* padding for custom checkmark */
        get_label_padding(config)
    )
}

fn get_label_padding(config: &Config) -> String {
    if let Some(width) = get_custom_check_width(config) {
        format!("padding-right:{:?}px", width)
    } else {
        String::new()
    }
}

pub(crate) fn get_path_icon_css(icon: &PathIcon, config: &Config) -> String {
    let url = icon.path.to_string_lossy();
    let width = icon.width;
    let height = icon.height;

    if let Some(margin) = config.icon.as_ref().unwrap().horizontal_margin {
        if is_svg(&icon.path) {
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
                        background-image: url("{url}");
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
        }
    } else {
        if is_svg(&icon.path) {
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
        } else {
            format!(
                r#"
                    menuitem image {{
                        background-image: url("{url}");
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
}

pub(crate) fn get_data_icon_css(data_icon: &DataIcon, config: &Config) -> String {
    let width = data_icon.width;
    let height = data_icon.height;
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

pub(crate) fn get_svg_icon_css(svg: &SvgIcon, config: &Config) -> String {
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
    if let Some(width) = get_custom_check_width(config) {
        if let Some(margin) = config.icon.as_ref().unwrap().horizontal_margin {
            format!(
                r#"
                    menuitem image {{
                        margin-left: {margin}px;
                        margin-right: {margin}px;
                        min-width: {width}px;
                    }}
                "#
            )
        } else {
            format!(
                r#"
                    menuitem image {{
                        min-width: {width}px;
                    }}
                "#
            )
        }
    } else if let Some(margin) = config.icon.as_ref().unwrap().horizontal_margin {
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
