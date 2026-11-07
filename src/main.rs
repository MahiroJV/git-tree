#![windows_subsystem = "windows"]
use app::App;
use dioxus::desktop::tao::window::Icon;
use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;
#[allow(unused_imports)]
use image::load_from_memory;

mod app;
mod components;
mod git;
mod recent;
mod settings_store;
mod theme;
mod updater;

fn main() {
    // WebKitGTK's DMA-BUF renderer shows a black/blank window on NixOS,
    // NVIDIA and some Wayland compositors. Disable it unless the user
    // explicitly set the variable themselves. Must run before any threads start.
    #[cfg(target_os = "linux")]
    {
        if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }

        // On native Wayland (seen on GNOME) the title-bar buttons — close,
        // minimize, maximize — ignore clicks until the window is maximized
        // once. Under XWayland they work, so prefer it when it's available.
        // Set GDK_BACKEND yourself (e.g. GDK_BACKEND=wayland) to override.
        let on_wayland = std::env::var_os("WAYLAND_DISPLAY").is_some();
        let xwayland_available = std::env::var_os("DISPLAY").is_some();
        if on_wayland && xwayland_available && std::env::var_os("GDK_BACKEND").is_none() {
            std::env::set_var("GDK_BACKEND", "x11");
        }
    }

    let icon_bytes = include_bytes!("../assets/icon/png/icon.png");
    let img = load_from_memory(icon_bytes).unwrap().to_rgba8();
    let (w, h) = img.dimensions();
    let icon = Icon::from_rgba(img.into_raw(), w, h).unwrap();

    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("git-tree");

    let config = Config::default().with_data_directory(data_dir).with_window(
        WindowBuilder::new()
            .with_title("git-tree")
            .with_window_icon(Some(icon))
            .with_decorations(true)
            .with_inner_size(dioxus::desktop::tao::dpi::LogicalSize::new(
                1280.0_f64, 800.0_f64,
            ))
            .with_min_inner_size(dioxus::desktop::tao::dpi::LogicalSize::new(
                900.0_f64, 600.0_f64,
            ))
            .with_resizable(true),
    );

    // Dioxus adds a default "Window / Edit" menu bar. On Linux most of its
    // "Window" items (hide, show all, …) are macOS-only, so the menu opens
    // empty (seen on Wayland/NixOS). The app has its own toolbar, and copy/
    // paste works in the webview without the menu, so drop it. macOS keeps it
    // for the standard app menu (Cmd+Q, Cmd+C/V).
    #[cfg(not(target_os = "macos"))]
    let config = config.with_menu(None);

    LaunchBuilder::desktop().with_cfg(config).launch(App);
}
