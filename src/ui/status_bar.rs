//! One-line status bar with small text, like Noggit.

use eframe::egui::{self, RichText, Ui};

use super::theme;
use crate::editor::Editor;

pub fn show(ui: &mut Ui, editor: &Editor) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 18.0;
        let item = |ui: &mut Ui, text: String| {
            ui.label(RichText::new(text).size(11.0).color(theme::TEXT_WEAK));
        };

        item(
            ui,
            match editor.cursor {
                Some(c) => format!("cursor: ({:.0}, {:.0}, {:.0})", c.x, c.y, c.z),
                None => "cursor: -".into(),
            },
        );
        item(ui, format!("grid: {}{}", editor.grid.size(), if editor.grid.snap { "" } else { " (no snap)" }));
        item(ui, editor.camera.view_name());

        match (editor.edit_context(), editor.scene.active()) {
            (Some(mode), Some(object)) => {
                let r = editor.scene.selection.elements.resolve(&object.mesh, mode);
                item(
                    ui,
                    format!(
                        "Edit Mode ({}): verts {}/{}, edges {}, faces {}/{}",
                        mode.label(),
                        r.verts.len(),
                        object.mesh.vertices.len(),
                        r.edges.len(),
                        r.faces.len(),
                        object.mesh.faces.len()
                    ),
                );
            }
            _ => item(
                ui,
                format!(
                    "Object Mode: objects {}, selected {}",
                    editor.scene.objects().len(),
                    editor.scene.selection.objects.len()
                ),
            ),
        }
        if let Some(b) = crate::tools::selection_bounds(&editor.scene, editor.edit_context()) {
            let s = b.size();
            item(ui, format!("size: {} × {} × {}", s.x, s.y, s.z));
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let dt = ui.input(|i| i.stable_dt);
            item(ui, format!("frame time: {:.1} ms", dt * 1000.0));
        });
    });
}
