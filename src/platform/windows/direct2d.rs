use std::mem::ManuallyDrop;

use super::{get_current_theme, rgba_from_hex, to_hex_string, to_pcwstr, Config, FontWeight, MenuFont, Theme};
use windows::{
    core::{w, Error, Interface, PCWSTR},
    Win32::{
        Foundation::{GENERIC_READ, RECT},
        Graphics::{
            Direct2D::{
                Common::{D2D1_ALPHA_MODE_IGNORE, D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_COLOR_F, D2D1_PIXEL_FORMAT, D2D_RECT_F, D2D_SIZE_F},
                D2D1CreateFactory, ID2D1DCRenderTarget, ID2D1DeviceContext5, ID2D1Factory1, ID2D1SvgDocument, ID2D1SvgElement, D2D1_BITMAP_INTERPOLATION_MODE_NEAREST_NEIGHBOR,
                D2D1_BITMAP_OPTIONS_NONE, D2D1_BITMAP_PROPERTIES1, D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_FEATURE_LEVEL_DEFAULT, D2D1_RENDER_TARGET_PROPERTIES, D2D1_RENDER_TARGET_TYPE_DEFAULT,
                D2D1_RENDER_TARGET_USAGE_NONE, D2D1_SVG_ATTRIBUTE_STRING_TYPE_SVG, D2D1_SVG_PATH_COMMAND_ARC_RELATIVE, D2D1_SVG_PATH_COMMAND_LINE_ABSOLUTE, D2D1_SVG_PATH_COMMAND_LINE_RELATIVE,
                D2D1_SVG_PATH_COMMAND_MOVE_ABSOLUTE,
            },
            DirectWrite::{
                DWriteCreateFactory, IDWriteFactory, IDWriteTextFormat, DWRITE_FACTORY_TYPE_SHARED, DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT_BOLD,
                DWRITE_FONT_WEIGHT_LIGHT, DWRITE_FONT_WEIGHT_MEDIUM, DWRITE_FONT_WEIGHT_NORMAL, DWRITE_FONT_WEIGHT_THIN, DWRITE_PARAGRAPH_ALIGNMENT_CENTER, DWRITE_TEXT_ALIGNMENT_LEADING,
                DWRITE_TEXT_ALIGNMENT_TRAILING, DWRITE_TEXT_METRICS, DWRITE_WORD_WRAPPING_NO_WRAP,
            },
            Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
            Imaging::{CLSID_WICImagingFactory, IWICImagingFactory, WICDecodeMetadataCacheOnDemand},
        },
        System::Com::{CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED},
    },
};

#[derive(PartialEq)]
pub(crate) enum TextAlignment {
    Leading,
    Trailing,
}

pub(crate) struct SvgDocument {
    pub(crate) document: ID2D1SvgDocument,
    pub(crate) size: f32,
}

pub(crate) fn create_render_target() -> ID2D1DCRenderTarget {
    let factory: ID2D1Factory1 = unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None).unwrap() };

    let prop = D2D1_RENDER_TARGET_PROPERTIES {
        r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
        pixelFormat: D2D1_PIXEL_FORMAT {
            format: DXGI_FORMAT_B8G8R8A8_UNORM,
            alphaMode: D2D1_ALPHA_MODE_IGNORE,
        },
        dpiX: 0.0,
        dpiY: 0.0,
        usage: D2D1_RENDER_TARGET_USAGE_NONE,
        minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
    };

    unsafe { factory.CreateDCRenderTarget(&prop).unwrap() }
}

pub(crate) fn get_device_context(target: &ID2D1DCRenderTarget) -> ID2D1DeviceContext5 {
    target.cast().unwrap()
}

pub(crate) fn create_write_factory() -> IDWriteFactory {
    unsafe { DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED).unwrap() }
}

pub(crate) fn set_fill_color(element: &ID2D1SvgElement, color: u32) {
    let hex_string = to_hex_string(color);
    unsafe { element.SetAttributeValue3(w!("fill"), D2D1_SVG_ATTRIBUTE_STRING_TYPE_SVG, to_pcwstr(&hex_string)).unwrap() };
}

pub(crate) fn set_stroke_color(element: &ID2D1SvgElement, color: u32) {
    let hex_string = to_hex_string(color);
    unsafe { element.SetAttributeValue3(w!("stroke"), D2D1_SVG_ATTRIBUTE_STRING_TYPE_SVG, to_pcwstr(&hex_string)).unwrap() };
}

fn font_point_to_pixel(font_point: f32) -> f32 {
    1.3 * font_point
}

pub(crate) fn create_check_svg(target: &ID2D1DCRenderTarget, menu_font: &MenuFont) -> SvgDocument {
    let dc5 = get_device_context(target);

    let document = unsafe {
        dc5.CreateSvgDocument(
            None,
            D2D_SIZE_F {
                width: 1.0,
                height: 1.0,
            },
        )
        .unwrap()
    };

    let font_size = font_point_to_pixel(menu_font.dark_font_size.max(menu_font.light_font_size));
    let size = font_size.ceil();
    unsafe {
        document
            .SetViewportSize(D2D_SIZE_F {
                width: size,
                height: size,
            })
            .unwrap()
    };

    let root = unsafe { document.GetRoot().unwrap() };
    unsafe { root.SetAttributeValue3(w!("stroke-width"), D2D1_SVG_ATTRIBUTE_STRING_TYPE_SVG, w!("0.8")).unwrap() };
    unsafe { root.SetAttributeValue3(w!("viewBox"), D2D1_SVG_ATTRIBUTE_STRING_TYPE_SVG, w!("-0.8 -0.8 16 16")).unwrap() };

    let segmentdata = [
        13.854, 3.646, 0.5, 0.5, 0.0, 0.0, 1.0, 0.0, 0.708, -7.0, 7.0, 0.5, 0.5, 0.0, 0.0, 1.0, -0.708, 0.0, -3.5, -3.5, 0.5, 0.5, 0.0, 1.0, 1.0, 0.708, -0.708, 6.5, 10.293, 6.646, -6.647, 0.5, 0.5,
        0.0, 0.0, 1.0, 0.708, 0.0,
    ];

    let commands = [
        D2D1_SVG_PATH_COMMAND_MOVE_ABSOLUTE,
        D2D1_SVG_PATH_COMMAND_ARC_RELATIVE,
        D2D1_SVG_PATH_COMMAND_LINE_RELATIVE,
        D2D1_SVG_PATH_COMMAND_ARC_RELATIVE,
        D2D1_SVG_PATH_COMMAND_LINE_RELATIVE,
        D2D1_SVG_PATH_COMMAND_ARC_RELATIVE,
        D2D1_SVG_PATH_COMMAND_LINE_ABSOLUTE,
        D2D1_SVG_PATH_COMMAND_LINE_RELATIVE,
        D2D1_SVG_PATH_COMMAND_ARC_RELATIVE,
    ];
    let path = unsafe { document.CreatePathData(Some(&segmentdata), Some(&commands)).unwrap() };

    let root = unsafe { document.GetRoot().unwrap() };
    let element = unsafe { root.CreateChild(w!("path")).unwrap() };
    unsafe { element.SetAttributeValue(w!("d"), &path).unwrap() };

    unsafe { root.AppendChild(&element).unwrap() };

    SvgDocument {
        document,
        size,
    }
}

pub(crate) fn draw_image(target: &ID2D1DCRenderTarget, icon: &std::path::Path, base_rect: &RECT) -> Result<(), Error> {
    let dc5 = get_device_context(target);

    let _ = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };

    let wic_factory: IWICImagingFactory = unsafe { CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)? };

    let image_path = super::encode_wide(icon.as_os_str());
    let file_path = PCWSTR::from_raw(image_path.as_ptr());
    let decoder = unsafe { wic_factory.CreateDecoderFromFilename(file_path, None, GENERIC_READ, WICDecodeMetadataCacheOnDemand)? };

    // Get the first frame of the image
    let frame = unsafe { decoder.GetFrame(0).unwrap() };

    let bitmap = unsafe {
        dc5.CreateBitmapFromWicBitmap(
            &frame,
            Some(&D2D1_BITMAP_PROPERTIES1 {
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                },
                dpiX: 0.0,
                dpiY: 0.0,
                bitmapOptions: D2D1_BITMAP_OPTIONS_NONE,
                colorContext: ManuallyDrop::new(None),
            }),
        )
    }?;

    unsafe { target.DrawBitmap(&bitmap, Some(&to_2d_rect(base_rect)), 1.0, D2D1_BITMAP_INTERPOLATION_MODE_NEAREST_NEIGHBOR, None) };

    unsafe { CoUninitialize() };

    Ok(())
}

pub(crate) fn create_submenu_svg(target: &ID2D1DCRenderTarget, menu_font: &MenuFont) -> SvgDocument {
    let dc5 = get_device_context(target);

    let document = unsafe {
        dc5.CreateSvgDocument(
            None,
            D2D_SIZE_F {
                width: 1.0,
                height: 1.0,
            },
        )
        .unwrap()
    };

    let font_size = font_point_to_pixel(menu_font.dark_font_size.max(menu_font.light_font_size));
    let size = (font_size * 0.625).ceil();
    unsafe {
        document
            .SetViewportSize(D2D_SIZE_F {
                width: size,
                height: size,
            })
            .unwrap()
    };

    let root = unsafe { document.GetRoot().unwrap() };
    unsafe { root.SetAttributeValue3(w!("stroke-width"), D2D1_SVG_ATTRIBUTE_STRING_TYPE_SVG, w!("0.8")).unwrap() };
    unsafe { root.SetAttributeValue3(w!("viewBox"), D2D1_SVG_ATTRIBUTE_STRING_TYPE_SVG, w!("-0.8 -0.8 16 16")).unwrap() };

    let segmentdata = [
        4.646, 1.646, 0.5, 0.5, 0.0, 0.0, 1.0, 0.708, 0.0, 6.0, 6.0, 0.5, 0.5, 0.0, 0.0, 1.0, 0.0, 0.708, -6.0, 6.0, 0.5, 0.5, 0.0, 0.0, 1.0, -0.708, -0.708, 10.293, 8.0, 4.646, 2.354, 0.5, 0.5, 0.0,
        0.0, 1.0, 0.0, -0.708,
    ];

    let commands = [
        D2D1_SVG_PATH_COMMAND_MOVE_ABSOLUTE,
        D2D1_SVG_PATH_COMMAND_ARC_RELATIVE,
        D2D1_SVG_PATH_COMMAND_LINE_RELATIVE,
        D2D1_SVG_PATH_COMMAND_ARC_RELATIVE,
        D2D1_SVG_PATH_COMMAND_LINE_RELATIVE,
        D2D1_SVG_PATH_COMMAND_ARC_RELATIVE,
        D2D1_SVG_PATH_COMMAND_LINE_ABSOLUTE,
        D2D1_SVG_PATH_COMMAND_LINE_ABSOLUTE,
        D2D1_SVG_PATH_COMMAND_ARC_RELATIVE,
    ];
    let path = unsafe { document.CreatePathData(Some(&segmentdata), Some(&commands)).unwrap() };

    let root = unsafe { document.GetRoot().unwrap() };
    let element = unsafe { root.CreateChild(w!("path")).unwrap() };
    unsafe { element.SetAttributeValue3(w!("fill-rule"), D2D1_SVG_ATTRIBUTE_STRING_TYPE_SVG, w!("evenodd")).unwrap() };
    unsafe { element.SetAttributeValue(w!("d"), &path).unwrap() };
    unsafe { root.AppendChild(&element).unwrap() };

    SvgDocument {
        document,
        size,
    }
}

pub(crate) fn get_text_format(factory: &IDWriteFactory, theme: Theme, config: &Config, alignment: TextAlignment) -> Result<IDWriteTextFormat, Error> {
    let current_theme = get_current_theme(theme);

    let (font_size, font_weight_value) = match current_theme {
        Theme::Dark => (config.font.dark_font_size, config.font.dark_font_weight),
        Theme::Light => (config.font.light_font_size, config.font.light_font_weight),
        // get_current_theme never returns System,
        Theme::System => (0.0, FontWeight::Normal),
    };

    let font_weight = match font_weight_value {
        FontWeight::Thin => DWRITE_FONT_WEIGHT_THIN,
        FontWeight::Light => DWRITE_FONT_WEIGHT_LIGHT,
        FontWeight::Normal => DWRITE_FONT_WEIGHT_NORMAL,
        FontWeight::Medium => DWRITE_FONT_WEIGHT_MEDIUM,
        FontWeight::Bold => DWRITE_FONT_WEIGHT_BOLD,
    };

    let format = unsafe { factory.CreateTextFormat(to_pcwstr(&config.font.font_family), None, font_weight, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_STRETCH_NORMAL, font_size, w!(""))? };

    if alignment == TextAlignment::Leading {
        unsafe { format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING) }?;
    }

    if alignment == TextAlignment::Trailing {
        unsafe { format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_TRAILING) }?;
    }

    unsafe { format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER)? };

    unsafe { format.SetWordWrapping(DWRITE_WORD_WRAPPING_NO_WRAP)? };

    Ok(format)
}

pub(crate) fn get_text_metrics(factory: &IDWriteFactory, theme: Theme, config: &Config, text: &mut Vec<u16>) -> Result<DWRITE_TEXT_METRICS, Error> {
    let format = get_text_format(factory, theme, config, TextAlignment::Leading).unwrap();
    let layout = unsafe { factory.CreateTextLayout(text.as_mut(), &format, 0.0, 0.0).unwrap() };
    let mut textmetrics = DWRITE_TEXT_METRICS::default();
    unsafe { layout.GetMetrics(&mut textmetrics).unwrap() }

    Ok(textmetrics)
}

pub(crate) fn to_2d_rect(rect: &RECT) -> D2D_RECT_F {
    D2D_RECT_F {
        left: rect.left as f32,
        top: rect.top as f32,
        right: rect.right as f32,
        bottom: rect.bottom as f32,
    }
}

pub(crate) fn colorref_to_d2d1_color_f(color: u32) -> D2D1_COLOR_F {
    let rgba = rgba_from_hex(color);
    D2D1_COLOR_F {
        r: (rgba.r as f32) / 255.0,
        g: (rgba.g as f32) / 255.0,
        b: (rgba.b as f32) / 255.0,
        a: rgba.a,
    }
}
