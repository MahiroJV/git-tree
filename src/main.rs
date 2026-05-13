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
mod theme;

fn main() {
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

    LaunchBuilder::desktop().with_cfg(config).launch(App);
}
