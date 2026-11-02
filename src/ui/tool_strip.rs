//! Vertical tool strip on the left, with icon buttons.

use eframe::egui::{self, Color32, Rect, Sense, Ui, vec2};

use super::icons::{self, Icons};
use super::theme;
use crate::editor::Editor;
use crate::tools::ToolKind;

const BUTTON: f32 = 34.0;
const ICON: f32 = 20.0;

pub fn show(ui: &mut Ui, editor: &mut Editor, icons: &Icons) {
    ui.add_space(4.0);
    ui.spacing_mut().item_spacing.y = 2.0;
    for tool in ToolKind::ALL {
        let active = editor.tools.active == tool;
        let response = ui
            .vertical_centered(|ui| icon_button(ui, icons.tool(tool), active))
            .inner
            .on_hover_text(format!("{}  ({})", tool.label(), tool.shortcut()));
        if response.clicked() {
            editor.set_tool(tool);
        }
    }
}

/// Square icon button: blue icon and a left accent bar when active.
fn icon_button(ui: &mut Ui, texture: &egui::TextureHandle, active: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(vec2(BUTTON, BUTTON), Sense::click());
    let painter = ui.painter();

    if active {
        painter.rect_filled(rect, 4.0, theme::ACCENT.gamma_multiply(0.18));
        let bar = Rect::from_min_size(rect.left_top() + vec2(-4.0, 6.0), vec2(3.0, BUTTON - 12.0));
        painter.rect_filled(bar, 1.5, theme::ACCENT);
    } else if response.hovered() {
        painter.rect_filled(rect, 4.0, Color32::from_white_alpha(14));
    }

    let tint = if active {
        theme::ACCENT_LIGHT
    } else if response.hovered() {
        Color32::WHITE
    } else {
        Color32::from_gray(185)
    };
    icons::paint(ui, texture, Rect::from_center_size(rect.center(), vec2(ICON, ICON)), tint);
    response
}
