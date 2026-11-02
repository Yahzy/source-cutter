//! Source Cutter — a small 3D modelling tool dedicated to the Source Engine.
//!
//! Module overview:
//! - `app`      : eframe application, panel layout and global shortcuts.
//! - `editor`   : all editable state (scene, camera, grid, tools).
//! - `scene`    : the document being edited (brushes).
//! - `tools`    : interactive tools (select, block...).
//! - `viewport` : 3D viewport rendering and input.
//! - `ui`       : side panels, toolbar, menus, status bar, theme.
//! - `math`     : small geometry helpers (AABB, rays).
//! - `units`    : Source Engine unit conventions.

mod app;
mod editor;
mod math;
mod scene;
mod tools;
mod ui;
mod units;
mod viewport;

use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Source Cutter")
            .with_inner_size([1600.0, 900.0])
            .with_min_inner_size([900.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Source Cutter",
        options,
        Box::new(|cc| Ok(Box::new(app::SourceCutterApp::new(cc)))),
    )
}
