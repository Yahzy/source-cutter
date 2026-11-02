//! Horizontal toolbar above the viewport: grid and view settings.

use eframe::egui::{self, Ui};

use crate::editor::Editor;

pub fn show(ui: &mut Ui, editor: &mut Editor) {
    ui.horizontal(|ui| {
        ui.label("Grid");
        if ui.button("−").on_hover_text("Smaller grid  [").clicked() {
            editor.grid.decrease();
        }
        ui.add_sized([36.0, 18.0], egui::Label::new(editor.grid.size().to_string()));
        if ui.button("+").on_hover_text("Bigger grid  ]").clicked() {
            editor.grid.increase();
        }
        ui.toggle_value(&mut editor.grid.snap, "🧲 Snap").on_hover_text("Snap to grid");
        ui.toggle_value(&mut editor.grid.visible, "# Grid").on_hover_text("Show grid");

        ui.separator();
        ui.toggle_value(&mut editor.view.show_player_reference, "🚶 Player")
            .on_hover_text("Show a 32×32×72 player hull for scale");

        ui.separator();
        if ui.button("⟲ Reset view").clicked() {
            editor.camera = Default::default();
        }
    });
}
