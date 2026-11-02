//! Vertical tool strip on the left.

use eframe::egui::{self, RichText, Ui};

use crate::editor::Editor;
use crate::tools::ToolKind;

pub fn show(ui: &mut Ui, editor: &mut Editor) {
    ui.vertical_centered(|ui| {
        ui.add_space(6.0);
        for tool in ToolKind::ALL {
            let selected = editor.tools.active == tool;
            let button = egui::Button::selectable(selected, RichText::new(tool.icon()).size(20.0))
                .min_size(egui::vec2(34.0, 34.0));
            if ui
                .add(button)
                .on_hover_text(format!("{}  ({})", tool.label(), tool.shortcut()))
                .clicked()
            {
                editor.tools.set_active(tool);
            }
        }
    });
}
