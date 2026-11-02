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
            if ui.add_enabled(editor.history.can_undo(), egui::Button::new("Undo              Ctrl+Z")).clicked() {
                editor.undo();
            }
            if ui.add_enabled(editor.history.can_redo(), egui::Button::new("Redo              Ctrl+Shift+Z")).clicked() {
                editor.redo();
            }
            ui.separator();
            let has_selection = !editor.scene.selection.objects.is_empty();
            if ui.add_enabled(has_selection, egui::Button::new("Delete            Del")).clicked() {
                editor.delete_selected();
            }
            ui.separator();
            if ui.button("Select all        A").clicked() {
                editor.select_all();
            }
            if ui.button("Deselect all      Alt+A").clicked() {
                editor.deselect_all();
            }
        });

        ui.menu_button("View", |ui| {
            ui.checkbox(&mut editor.grid.visible, "Show grid");
            ui.checkbox(&mut editor.view.show_player_reference, "Player reference");
            ui.separator();
            let views = [
                ("Top", glam::Vec3::Z),
                ("Front", glam::Vec3::NEG_Y),
                ("Right", glam::Vec3::X),
                ("Bottom", glam::Vec3::NEG_Z),
                ("Back", glam::Vec3::Y),
                ("Left", glam::Vec3::NEG_X),
            ];
            for (name, dir) in views {
                if ui.button(name).clicked() {
                    editor.camera.look_from(dir);
                }
            }
            if ui.button("Perspective / Ortho   5").clicked() {
                editor.camera.toggle_projection();
            }
            ui.separator();
            if ui.button("Frame selection   F").clicked() {
                editor.frame_selection();
            }
        });

        ui.menu_button("Help", |ui| {
            let lines = [
                ("Middle mouse", "orbit"),
                ("Shift + middle mouse", "pan"),
                ("Wheel", "zoom"),
                ("Compass click / 5", "ortho views / toggle"),
                ("Tab", "Object / Edit Mode"),
                ("1 / 2 / 3", "vertex / edge / face"),
                ("G / S", "Move / Scale tool"),
                ("Shift+S / Shift+B", "Select / Block tool"),
                ("[ / ]", "grid size"),
                ("Ctrl+Z / Ctrl+Shift+Z", "undo / redo"),
            ];
            egui::Grid::new("help_keys").num_columns(2).show(ui, |ui| {
                for (key, action) in lines {
                    ui.monospace(key);
                    ui.label(action);
                    ui.end_row();
                }
            });
        });
    });
}
