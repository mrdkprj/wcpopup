# wcpopup &emsp;  [![Latest Version]][crates.io] [![Documentation]][docs]

[Documentation]: https://docs.rs/wcpopup/badge.svg
[docs]: https://docs.rs/wcpopup
[Latest Version]: https://img.shields.io/crates/v/wcpopup.svg
[crates.io]: https://crates.io/crates/wcpopup

Rust context menu for Windows and Linux(Gtk3).  
Supports dark/light theme and color/size configuration. 
- Colors
    - Text color
    - Background color
    - Border color
- Size
    - Menu padding
    - Menu item padding
- Font
    - Font family
    - Size and weight
  
![sample](https://github.com/mrdkprj/rpopup/blob/main/assets/light.jpg?raw=true)![sample](https://github.com/mrdkprj/rpopup/blob/main/assets/dark.jpg?raw=true)  

# Usage
Use ManuBuilder to create a Menu with MenuItems.  

```rust
fn example(window_handle: isize) {
    let mut builder = MenuBuilder::new(window_handle);
    // Using HWND
    // let mut builder = MenuBuilder::new_for_hwnd(hwnd);
    // Using gtk::ApplicationWindow or gkt::Window
    // let mut builder = MenuBuilder::new_for_window(window);

    builder.check("menu_item1", "Fit To Window", true, None);
    builder.separator();
    builder.text_with_accelerator("menu_item2", "Playlist", None, "Ctrl+P");
    builder.text_with_accelerator("menu_item3", "Toggle Fullscreen", None, "F11");
    builder.text("menu_item4", "Picture In Picture", None);
    builder.separator();
    builder.text_with_accelerator("menu_item5", "Capture", None, "Ctrl+S");
    builder.separator();

    let mut submenu = builder.submenu("submenu1", "Theme", None);
    submenu.radio("submenu_item1", "Light", "Theme", true, None);
    submenu.radio("submenu_item2", "Dark", "Theme", false, None);
    submenu.build().unwrap();

    let menu = builder.build().unwrap();

}
```

Call Menu.popup_at() to show Menu and receive the selected MenuItem using MenuEvent.
```rust
fn show_context_menu(x:i32, y:i32) {
    menu.popup_at(x, y);
}

if let Ok(event) = MenuEvent::receiver().try_recv() {
    let selected_menu_item = event.item;    
}
```

Or call Menu.popup_at_async() to show Menu and wait asynchronously for a selected MenuItem.
```rust
async fn show_context_menu(x:i32, y:i32) {
    let selected_menu_item = menu.popup_at(x, y).await;
}
```



## Platform-specific notes
### Windows
WebView2 may receive all keyboard input instead of its parent window([#1703](https://github.com/MicrosoftEdge/WebView2Feedback/issues/1703)).    
Using WebView2, you may need to enable the feature flag.
```
--enable-features=msWebView2BrowserHitTransparent
```

### Linux
Gtk3 is required.  
MenuItem's text color is applied to SVG icon if the SVG file contains the "symbolic" term as the last component of the file name.  

## Accelerator
Accelerators are used only to display available shortcut keys by default.  
Use "accelerator" feature to treat accelerators as commands.  
With this feature, when a shortcut key is pressed, the corresponding MenuItem is returned as the result of Menu.popup_at().  

```rust
features = ["accelerator"]
```
