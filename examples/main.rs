use async_std::sync::Mutex;
use once_cell::sync::Lazy;
use std::collections::HashMap;
#[cfg(target_os = "windows")]
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy, EventLoopWindowTarget},
    platform::windows::WindowExtWindows,
    window::{Window, WindowBuilder, WindowId},
};
#[cfg(target_os = "linux")]
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy, EventLoopWindowTarget},
    window::{Window, WindowBuilder, WindowId},
};
use wcpopup::{
    config::{ColorScheme, Config, Corner, MenuSize, Theme, ThemeColor, DEFAULT_DARK_COLOR_SCHEME},
    Menu, MenuBuilder, MenuEvent, MenuIcon, MenuItem,
};
#[cfg(target_os = "windows")]
use wry::WebViewBuilderExtWindows;
use wry::{http::Request, WebView, WebViewBuilder};

static MENU_MAP: Lazy<Mutex<Menu>> = Lazy::new(|| Mutex::new(Menu::default()));
static DARK_MODE: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(true));

enum UserEvent {
    CloseWindow(WindowId),
    Popup(i32, i32),
    ChangeTheme,
    Append,
    ChangeStateAndIcon,
    Remove,
}

const START_DARK: bool = true;
const ASYNC: bool = true;

fn main() -> wry::Result<()> {
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let mut webviews = HashMap::new();
    let proxy = event_loop.create_proxy();

    let new_window = create_new_window(format!("Window {}", webviews.len() + 1), &event_loop, proxy.clone());

    webviews.insert(new_window.0.id(), (new_window.0, new_window.1));

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if !ASYNC {
            if let Ok(event) = MenuEvent::receiver().try_recv() {
                println!("MenuEvent:{:?}", event.item.label);
            }
        }

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
                ..
            } => {
                webviews.remove(&window_id);
                if webviews.is_empty() {
                    *control_flow = ControlFlow::Exit
                }
            }
            Event::UserEvent(UserEvent::CloseWindow(id)) => {
                webviews.remove(&id);
                if webviews.is_empty() {
                    println!("exit");
                    *control_flow = ControlFlow::Exit
                }
            }
            Event::UserEvent(UserEvent::Popup(x, y)) => {
                #[cfg(target_os = "windows")]
                if ASYNC {
                    async_std::task::spawn(async move {
                        let menu = MENU_MAP.lock().await;
                        let result = menu.popup_at_async(x, y).await;
                        if let Some(item) = result {
                            println!("Async MenuEvent:{:?}", item.label);
                        }
                    });
                } else {
                    let menu = MENU_MAP.try_lock().unwrap();
                    menu.popup_at(x, y);
                }
                #[cfg(target_os = "linux")]
                if ASYNC {
                    gtk::glib::spawn_future_local(async move {
                        let menu = MENU_MAP.lock().await;
                        let result = menu.popup_at_async(x, y).await;
                        if let Some(item) = result {
                            println!("Async MenuEvent:{:?}", item.label);
                        }
                    });
                } else {
                    let menu = MENU_MAP.try_lock().unwrap();
                    menu.popup_at(x, y);
                }
            }
            Event::UserEvent(UserEvent::ChangeTheme) => {
                let mut is_dark = DARK_MODE.try_lock().unwrap();
                let should_be_dark = !(*is_dark);

                let menu = MENU_MAP.try_lock().unwrap();
                menu.set_theme(if should_be_dark {
                    Theme::Dark
                } else {
                    Theme::Light
                });

                (*is_dark) = should_be_dark;
            }
            Event::UserEvent(UserEvent::Append) => {
                let mut menu = MENU_MAP.try_lock().unwrap();
                let radio = MenuItem::new_radio_item("new_radio", "new_radio_label", "Theme", None, true, false, None);
                let playback_speed = menu.get_menu_item_by_id("Theme").unwrap();
                playback_speed.submenu.unwrap().insert(radio, 1);

                let mut item = MenuItem::new_submenu_item("newsubmenu_id", "label", false, None);
                item.add_menu_item(MenuItem::new_text_item("id1", "label1", Some("Alt+G"), false, None));
                item.add_menu_item(MenuItem::new_text_item("id2", "label2", None, false, None));
                menu.append(item);
            }
            Event::UserEvent(UserEvent::ChangeStateAndIcon) => {
                let menu = MENU_MAP.try_lock().unwrap();
                if let Some(target) = menu.get_menu_item_by_id("id1").as_mut() {
                    target.set_label("Changed Label");
                }

                if let Some(target) = menu.get_menu_item_by_id("fittowindow").as_mut() {
                    target.set_checked(!target.checked);
                }

                if let Some(target) = menu.get_menu_item_by_id("dark").as_mut() {
                    target.set_checked(!target.checked);
                }

                if let Some(target) = menu.get_menu_item_by_id("SeekSpeed").as_mut() {
                    target.set_disabled(!target.disabled);
                }

                if let Some(target) = menu.get_menu_item_by_id("TogglePlaylistWindow").as_mut() {
                    let icon = target.icon.clone();
                    target.set_icon(None);

                    if let Some(target) = menu.get_menu_item_by_id("Capture").as_mut() {
                        target.set_icon(icon);
                    }
                }
            }
            Event::UserEvent(UserEvent::Remove) => {
                let mut menu = MENU_MAP.try_lock().unwrap();
                if let Some(target) = menu.get_menu_item_by_id("new_radio") {
                    menu.get_menu_item_by_id("Theme").unwrap().submenu.unwrap().remove(&target);
                }

                if let Some(target) = menu.get_menu_item_by_id("newsubmenu_id") {
                    menu.remove(&target);
                }
            }
            _ => (),
        }
    });
}

fn create_new_window(title: String, event_loop: &EventLoopWindowTarget<UserEvent>, proxy: EventLoopProxy<UserEvent>) -> (Window, WebView) {
    #[cfg(target_os = "linux")]
    use gtk::{ffi::GtkApplicationWindow, glib::translate::ToGlibPtr};
    #[cfg(target_os = "linux")]
    use tao::platform::unix::WindowExtUnix;

    let builder = WindowBuilder::new()
        .with_title(title)
        .with_resizable(true)
        .with_maximizable(true)
        .with_minimizable(true)
        .with_closable(true)
        .with_focused(true)
        .with_visible(true)
        .with_transparent(false)
        .with_theme(Some(if START_DARK {
            tao::window::Theme::Dark
        } else {
            tao::window::Theme::Light
        }));

    let window = builder.build(event_loop).unwrap();
    let window_id = window.id();

    #[cfg(target_os = "windows")]
    let ptr = window.hwnd();
    #[cfg(target_os = "linux")]
    let ptr: *mut GtkApplicationWindow = window.gtk_window().to_glib_none().0;
    #[cfg(target_os = "linux")]
    let ptr = ptr as isize;
    add_menu(ptr);

    #[cfg(target_os = "windows")]
    let builder = WebViewBuilder::new(&window);
    #[cfg(target_os = "linux")]
    let fixed = {
        use gtk::prelude::*;
        use tao::platform::unix::WindowExtUnix;
        let fixed = gtk::Fixed::new();
        let vbox = window.default_vbox().unwrap();
        vbox.pack_start(&fixed, true, true, 0);
        fixed.show_all();
        fixed
    };
    #[cfg(target_os = "linux")]
    let build_webview = |builder: WebViewBuilder<'_>| -> wry::Result<wry::WebView> {
        let webview = {
            use wry::WebViewBuilderExtUnix;
            builder.build_gtk(&fixed)?
        };

        Ok(webview)
    };

    let handler = move |req: Request<String>| {
        let body = req.body();

        match body.as_str() {
            "change_theme" => {
                let _ = proxy.send_event(UserEvent::ChangeTheme);
            }
            "append" => {
                let _ = proxy.send_event(UserEvent::Append);
            }
            "remove" => {
                let _ = proxy.send_event(UserEvent::Remove);
            }
            "change_state_and_icon" => {
                let _ = proxy.send_event(UserEvent::ChangeStateAndIcon);
            }
            "close" => {
                let _ = proxy.send_event(UserEvent::CloseWindow(window_id));
            }
            _ if body.starts_with("context") => {
                let param: Vec<&str> = body.split(':').collect();
                let mut pos = (0, 0);
                pos.0 = param[1].parse().unwrap();
                pos.1 = param[2].parse().unwrap();
                let _ = proxy.send_event(UserEvent::Popup(pos.0, pos.1));
            }
            _ => {}
        }
    };

    #[cfg(target_os = "windows")]
    let html = format!(
        r#"
        <script>
        let dark = {START_DARK};
        window.onload = () => {{
            if(dark){{
                document.body.style.backgroundColor = "black";
            }}else{{
                document.body.style.backgroundColor = "white";
            }}
        }}
        window.addEventListener("contextmenu", (e) => {{
           e.preventDefault();
            window.ipc.postMessage(`context:${{e.screenX}}:${{e.screenY}}`)
        }});
        function change_theme(){{
            dark = !dark;
            if(dark){{
                document.body.style.backgroundColor = "black";
                document.body.style.color = "white";
            }}else{{
                document.body.style.backgroundColor = "white";
                document.body.style.color = "black";
            }}
            window.ipc.postMessage('change_theme')
        }}
        </script>
        <button onclick="window.ipc.postMessage('append')">Add Items</button>
        <button onclick="window.ipc.postMessage('remove')">Remove Items</button>
        <button onclick="window.ipc.postMessage('change_state_and_icon')">Change Items</button>
        <button onclick="change_theme()">Change Theme</button>
        <button onclick="window.ipc.postMessage('close')">Close</button>
    "#
    );

    #[cfg(target_os = "linux")]
    let html = format!(
        r#"
        <script>
        let dark = {START_DARK};
        window.onload = () => {{
            if(dark){{
                document.body.style.backgroundColor = "black";
                document.body.style.color = "white";
            }}else{{
                document.body.style.backgroundColor = "white";
                document.body.style.color = "black";
            }}
        }}

        let openContext = false;
        window.addEventListener("contextmenu", (e) => {{
            e.preventDefault();
            openContext = true;
        }});
        window.addEventListener("mouseup", (e) => {{
            if (e.button === 2 && openContext) {{
                window.ipc.postMessage(`context:${{e.clientX}}:${{e.clientY}}`)
                openContext = false;
            }}
        }});

        function change_theme(){{
            dark = !dark;
            if(dark){{
                document.body.style.backgroundColor = "black";
                document.body.style.color = "white";
            }}else{{
                document.body.style.backgroundColor = "white";
                document.body.style.color = "black";
            }}
            window.ipc.postMessage('change_theme')
        }}
        </script>
        <button onclick="window.ipc.postMessage('append')">Add Items</button>
        <button onclick="window.ipc.postMessage('remove')">Remove Items</button>
        <button onclick="window.ipc.postMessage('change_state_and_icon')">Change Items</button>
        <button onclick="change_theme()">Change Theme</button>
        <button onclick="window.ipc.postMessage('close')">Close</button>
"#
    );

    #[cfg(target_os = "windows")]
    let webview = builder.with_html(html).with_ipc_handler(handler).with_devtools(true).with_transparent(false).with_focused(true).build().unwrap();

    #[cfg(target_os = "linux")]
    let webview = {
        let size = window.inner_size().to_logical::<u32>(window.scale_factor());
        let builder = WebViewBuilder::new()
            .with_bounds(wry::Rect {
                position: tao::dpi::LogicalPosition::new(0, 0).into(),
                size: tao::dpi::LogicalSize::new(size.width, size.height).into(),
            })
            .with_html(html)
            .with_ipc_handler(handler)
            .with_devtools(true)
            .with_transparent(false)
            .with_focused(true);

        build_webview(builder).unwrap()
    };
    // webview.open_devtools();
    (window, webview)
}

pub fn add_menu(window_handle: isize) {
    let size = MenuSize {
        horizontal_padding: 0,
        border_size: 0,
        ..Default::default()
    };

    let color = ThemeColor {
        dark: ColorScheme {
            color: 0xefefef,
            background_color: 0x202020,
            ..DEFAULT_DARK_COLOR_SCHEME
        },
        ..Default::default()
    };
    let mut builder = MenuBuilder::new_from_config(
        window_handle,
        Config {
            theme: if START_DARK {
                Theme::Dark
            } else {
                Theme::Light
            },
            size,
            color,
            corner: Corner::Round,
            ..Default::default()
        },
    );

    builder.text("PlaybackSpeed", "Playback Speed", false);
    builder.text("SeekSpeed", "Seek Speed", false);
    builder.check("fittowindow", "Fit To Window", true, false);
    builder.separator();
    #[cfg(target_os = "windows")]
    let icon_path = concat!(env!("CARGO_MANIFEST_DIR"), r"\examples\img\icon_audio.png");
    #[cfg(target_os = "linux")]
    let icon_path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/img/icon_audio.png");
    builder.text_with_icon("TogglePlaylistWindow", "Playlist", false, MenuIcon::new(icon_path));
    builder.text_with_accelerator("ToggleFullscreen", "Toggle Fullscreen", false, "F11");
    builder.text("PictureInPicture", "Picture In Picture", true);
    builder.separator();
    #[cfg(target_os = "windows")]
    let icon_path = concat!(env!("CARGO_MANIFEST_DIR"), r"\examples\img\camera.svg");
    #[cfg(target_os = "linux")]
    let icon_path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/img/camera-symbolic.svg");
    builder.text_with_accel_icon("Capture", "Capture", false, "Ctrl+S", MenuIcon::new(icon_path));
    builder.separator();
    create_theme_submenu(&mut builder);
    let menu = builder.build().unwrap();

    *MENU_MAP.try_lock().unwrap() = menu;
}

fn create_theme_submenu(builder: &mut MenuBuilder) {
    let id = "Theme";
    let mut parent = builder.submenu(id, "Theme", false);
    let theme = if START_DARK {
        "Dark"
    } else {
        "Light "
    };
    parent.radio("dark", theme, id, theme == "Dark", false);
    let theme = "Light";
    parent.radio("light", theme, id, theme == "Dark", false);

    parent.build().unwrap();
}
