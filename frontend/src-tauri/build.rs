fn main() {
    let windows_attrs = tauri_build::WindowsAttributes::new()
        .window_icon_path("icons/icon.ico")
        .append_rc_content("1 ICON \"icons/icon.ico\"\n");
    let attrs = tauri_build::Attributes::new().windows_attributes(windows_attrs);
    tauri_build::try_build(attrs).expect("failed to run tauri-build");
}
