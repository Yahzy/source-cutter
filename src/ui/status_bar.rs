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
        match (editor.edit_context(), editor.scene.active()) {
            (Some(mode), Some(object)) => {
                let r = editor.scene.selection.elements.resolve(&object.mesh, mode);
                ui.monospace(format!(
                    "edit: {}  verts {}/{}  edges {}  faces {}/{}",
                    mode.label(),
                    r.verts.len(),
                    object.mesh.vertices.len(),
                    r.edges.len(),
                    r.faces.len(),
                    object.mesh.faces.len()
                ));
            }
            _ => {
                ui.monospace(format!("selected: {}", editor.scene.selection.objects.len()));
            }
        }
        if let Some(b) = crate::tools::selection_bounds(&editor.scene, editor.edit_context()) {
            let s = b.size();
            ui.monospace(format!("({} × {} × {})", s.x, s.y, s.z));
        }

        ui.with_layout(eframe::egui::Layout::right_to_left(eframe::egui::Align::Center), |ui| {
            let dt = ui.input(|i| i.unstable_dt);
            ui.monospace(format!("frame: {:.1} ms", dt * 1000.0));
        });
    });
}
