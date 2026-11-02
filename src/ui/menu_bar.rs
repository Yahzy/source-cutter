use eframe::egui::{self, Ui};

use crate::editor::Editor;

pub fn show(ui: &mut Ui, editor: &mut Editor) {
    egui::MenuBar::new().ui(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("New scene        Ctrl+N").clicked() {
                editor.new_scene();
            }
            ui.separator();
            if ui.button("Quit").clicked() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });

        ui.menu_button("Edit", |ui| {
            let has_selection = editor.scene.selected().is_some();
            if ui.add_enabled(has_selection, egui::Button::new("Delete            Del")).clicked() {
                editor.delete_selected();
            }
            if ui.button("Deselect          Esc").clicked() {
                editor.scene.select(None);
            }
        });

        ui.menu_button("View", |ui| {
            ui.checkbox(&mut editor.grid.visible, "Show grid");
            ui.checkbox(&mut editor.view.show_player_reference, "Player reference");
            ui.separator();
            if ui.button("Frame selection   F").clicked() {
                editor.frame_selection();
            }
        });

        ui.menu_button("Help", |ui| {
            ui.label("Middle mouse: orbit");
            ui.label("Shift + middle mouse: pan");
            ui.label("Wheel: zoom");
            ui.label("[ / ] : grid size");
        });
    });
}
