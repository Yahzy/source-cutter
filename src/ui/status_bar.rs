use eframe::egui::Ui;

use crate::editor::Editor;

pub fn show(ui: &mut Ui, editor: &Editor) {
    ui.horizontal(|ui| {
        match editor.cursor {
            Some(c) => ui.monospace(format!("cursor: ({:.0}, {:.0}, {:.0})", c.x, c.y, c.z)),
            None => ui.monospace("cursor: -"),
        };
        ui.separator();
        ui.monospace(format!("grid: {}", editor.grid.size()));
        ui.separator();
        ui.monospace(format!("objects: {}", editor.scene.objects().len()));
        ui.separator();
        ui.monospace(format!("selected: {}", editor.scene.selection.objects.len()));
        if let Some(b) = editor.scene.selection_bounds() {
            let s = b.size();
            ui.monospace(format!("({} × {} × {})", s.x, s.y, s.z));
        }

        ui.with_layout(eframe::egui::Layout::right_to_left(eframe::egui::Align::Center), |ui| {
            let dt = ui.input(|i| i.unstable_dt);
            ui.monospace(format!("frame: {:.1} ms", dt * 1000.0));
        });
    });
}
