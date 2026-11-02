//! Horizontal toolbar above the viewport: grid and view settings.

use eframe::egui::{self, Ui};

use crate::editor::{Editor, EditorMode};
use crate::scene::SelectMode;

pub fn show(ui: &mut Ui, editor: &mut Editor) {
    ui.horizontal(|ui| {
        mode_selector(ui, editor);
        ui.separator();

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

/// Blender-like header: Object / Edit Mode (Tab) and vertex / edge / face select mode (1, 2, 3).
fn mode_selector(ui: &mut Ui, editor: &mut Editor) {
    let mut mode = editor.mode();
    egui::ComboBox::from_id_salt("editor_mode")
        .selected_text(mode.label())
        .width(110.0)
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut mode, EditorMode::Object, EditorMode::Object.label());
            ui.selectable_value(&mut mode, EditorMode::Edit, EditorMode::Edit.label());
        })
        .response
        .on_hover_text("Switch mode (Tab)");
    if mode != editor.mode() {
        editor.set_mode(mode);
    }

    let editing = editor.mode() == EditorMode::Edit;
    for select_mode in SelectMode::ALL {
        let selected = editing && editor.select_mode() == select_mode;
        let button = egui::Button::selectable(selected, select_mode.label());
        if ui
            .add_enabled(editing, button)
            .on_hover_text(format!("{} select ({})", select_mode.label(), select_mode.key()))
            .on_disabled_hover_text("Available in Edit Mode (Tab)")
            .clicked()
        {
            editor.set_select_mode(select_mode);
        }
    }
}
