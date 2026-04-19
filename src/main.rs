#![windows_subsystem = "windows"]
use dioxus::desktop::tao::window::Icon;
use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;
use image;

mod app;
mod components;
mod git;
mod theme;

use app::App;

fn main() {
    let icon_bytes = include_bytes!("assets/icon.png");
    let img = image::load_from_memory(icon_bytes).unwrap().to_rgba8();
    let (w, h) = img.dimensions();
    let icon = Icon::from_rgba(img.into_raw(), w, h).unwrap();

    let config = Config::default()
        .with_window(
            WindowBuilder::new()
                .with_title("git-tree")
                .with_window_icon(Some(icon))
                .with_decorations(true)
                .with_inner_size(dioxus::desktop::tao::dpi::LogicalSize::new(1280.0_f64, 800.0_f64))
                .with_min_inner_size(dioxus::desktop::tao::dpi::LogicalSize::new(900.0_f64, 600.0_f64))
                .with_resizable(true),
        )
        .with_custom_head(
            r#"<link rel="preconnect" href="https://fonts.googleapis.com">
            <link href="https://fonts.googleapis.com/css2?family=Space+Mono:wght@400;700&family=Oxanium:wght@300;600&display=swap" rel="stylesheet">"#
                .to_string(),
        );

    LaunchBuilder::desktop().with_cfg(config).launch(App);
}
