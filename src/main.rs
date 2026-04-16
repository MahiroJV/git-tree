#![windows_subsystem = "windows"]
use dioxus::prelude::*;
use dioxus::desktop::{Config, WindowBuilder};

mod app;
mod components;
mod git;
mod theme;

use app::App;

fn main() {
    let config = Config::default()
        .with_window(
            WindowBuilder::new()
                .with_title("git-tree")
                .with_decorations(true)
                .with_inner_size(dioxus::desktop::tao::dpi::LogicalSize::new(1280.0_f64, 800.0_f64))
                .with_min_inner_size(dioxus::desktop::tao::dpi::LogicalSize::new(900.0_f64, 600.0_f64))
                .with_resizable(true),
        )
        .with_custom_head(
            r#"<link rel="preconnect" href="https://fonts.googleapis.com">
<link href="https://fonts.googleapis.com/css2?family=Space+Mono:wght@400;700&display=swap" rel="stylesheet">"#
                .to_string(),
        );

    LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(App);
}