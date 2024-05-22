# RMenu
Rust context/popup menu for Windows.  
Supports dark/light theme and color/size configuration. 
- Colors
    - Text color
    - Background color
    - Border color
- Size
    - Menu top/bottom margin
    - Menu item top/bottom and left/right padding
    - Font size/weight
  
![sample](https://github.com/mrdkprj/rpopup/blob/main/assets/light.jpg?raw=true)![sample](https://github.com/mrdkprj/rpopup/blob/main/assets/dark.jpg?raw=true)  

# Usage
Use ManuBuilder to create a Menu with MenuItems, and then call Menu.popup_at() to show Menu.  
When an MenuItem is clicked, SelectedMenuItem data is returned.

```rust
fn example(hwnd: HWND) {
    let mut builder = MenuBuilder::new(hwnd);

    builder.check("menu_item1", "Fit To Window", "", true, None);
    builder.separator();
    builder.text_with_accelerator("menu_item2", "Playlist", None, "Ctrl+P");
    builder.text_with_accelerator("menu_item3", "Toggle Fullscreen", None, "F11");
    builder.text("menu_item4", "Picture In Picture", None);
    builder.separator();
    builder.text_with_accelerator("menu_item5", "Capture", None, "Ctrl+S");
    builder.separator();

    let mut submenu = builder.submenu("Theme", None);
    submenu.radio("submenu_item1", "Light", "Light", "Theme", true, None);
    submenu.radio("submenu_item2", "Dark", "Dark", "Theme", false, None);
    submenu.build().unwrap();

    let menu = builder.build().unwrap();

    let selected_item = menu.popup_at(100, 100);
}
```
