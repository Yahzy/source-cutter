//! Floating toolbar over the top-left of the viewport (like Noggit's icon bar):
//! Object/Edit Mode, vertex/edge/face select mode, grid and view toggles.

use eframe::egui::{self, Align2, Color32, FontId, Id, Order, Painter, Rect, Response, Sense, Stroke, Ui, vec2};

use super::theme;
use crate::editor::{Editor, EditorMode};
use crate::scene::SelectMode;

const BUTTON_H: f32 = 26.0;
/// Height of the toolbar, so the viewport can place its overlay text below.
pub const HEIGHT: f32 = BUTTON_H + 8.0;

pub fn show(ctx: &egui::Context, viewport: Rect, editor: &mut Editor) {
    egui::Area::new(Id::unique("viewport_toolbar"))
        .order(Order::Middle)
        .fixed_pos(viewport.left_top() + vec2(8.0, 8.0))
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(theme::OVERLAY)
                .stroke(Stroke::new(1.0, theme::BORDER))
                .corner_radius(4)
                .inner_margin(4)
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;
                    ui.horizontal(|ui| contents(ui, editor));
                });
        });
}

fn contents(ui: &mut Ui, editor: &mut Editor) {
    mode_selector(ui, editor);
    divider(ui);

    let editing = editor.mode() == EditorMode::Edit;
    for mode in SelectMode::ALL {
        let active = editing && editor.select_mode() == mode;
        let response = icon_button(ui, active, editing, |p, r, c| paint_select_mode(p, r, mode, c))
            .on_hover_text(format!("{} select ({})", mode.label(), mode.key()))
            .on_disabled_hover_text(format!("{} select: Edit Mode only (Tab)", mode.label()));
        if response.clicked() {
            editor.set_select_mode(mode);
        }
    }
    divider(ui);

    // Grid size: - [16] +
    if text_button(ui, "−", false).on_hover_text("Smaller grid  [").clicked() {
        editor.grid.decrease();
    }
    ui.add_sized([34.0, BUTTON_H], egui::Label::new(egui::RichText::new(editor.grid.size().to_string()).strong()))
        .on_hover_text("Grid size (units)");
    if text_button(ui, "+", false).on_hover_text("Bigger grid  ]").clicked() {
        editor.grid.increase();
    }
    if icon_button(ui, editor.grid.visible, true, paint_grid).on_hover_text("Show grid").clicked() {
        editor.grid.visible = !editor.grid.visible;
    }
    if icon_button(ui, editor.grid.snap, true, paint_snap).on_hover_text("Snap to grid").clicked() {
        editor.grid.snap = !editor.grid.snap;
    }
    divider(ui);

    if icon_button(ui, editor.view.show_player_reference, true, paint_player)
        .on_hover_text("Player reference (32×32×72)")
        .clicked()
    {
        editor.view.show_player_reference = !editor.view.show_player_reference;
    }
    let ortho = editor.camera.orthographic;
    if text_button(ui, if ortho { "Ortho" } else { "Persp" }, ortho).on_hover_text("Perspective / orthographic (5)").clicked() {
        editor.camera.toggle_projection();
    }
    if icon_button(ui, false, true, paint_frame).on_hover_text("Frame selection (F)").clicked() {
        editor.frame_selection();
    }
}

/// Object / Edit Mode drop-down (Tab).
fn mode_selector(ui: &mut Ui, editor: &mut Editor) {
    let mut mode = editor.mode();
    ui.scope(|ui| {
        ui.spacing_mut().interact_size.y = BUTTON_H;
        egui::ComboBox::from_id_salt("editor_mode")
            .selected_text(mode.label())
            .width(104.0)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut mode, EditorMode::Object, EditorMode::Object.label());
                ui.selectable_value(&mut mode, EditorMode::Edit, EditorMode::Edit.label());
            })
            .response
            .on_hover_text("Switch mode (Tab)");
    });
    if mode != editor.mode() {
        editor.set_mode(mode);
    }
}

// --- Buttons -------------------------------------------------------------------

fn button_colors(response: &Response, active: bool, enabled: bool) -> (Color32, Color32) {
    let bg = if active {
        theme::ACCENT.gamma_multiply(0.30)
    } else if enabled && response.hovered() {
        Color32::from_white_alpha(16)
    } else {
        Color32::TRANSPARENT
    };
    let fg = if !enabled {
        Color32::from_gray(80)
    } else if active {
        theme::ACCENT_LIGHT
    } else if response.hovered() {
        Color32::WHITE
    } else {
        Color32::from_gray(190)
    };
    (bg, fg)
}

fn icon_button(ui: &mut Ui, active: bool, enabled: bool, paint: impl FnOnce(&Painter, Rect, Color32)) -> Response {
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let (rect, response) = ui.allocate_exact_size(vec2(BUTTON_H, BUTTON_H), sense);
    let (bg, fg) = button_colors(&response, active, enabled);
    ui.painter().rect_filled(rect, 3.0, bg);
    paint(ui.painter(), rect.shrink(6.0), fg);
    if enabled { response } else { response.on_hover_cursor(egui::CursorIcon::NotAllowed) }
}

fn text_button(ui: &mut Ui, text: &str, active: bool) -> Response {
    let width = (text.len() as f32 * 7.5 + 12.0).max(BUTTON_H);
    let (rect, response) = ui.allocate_exact_size(vec2(width, BUTTON_H), Sense::click());
    let (bg, fg) = button_colors(&response, active, true);
    ui.painter().rect_filled(rect, 3.0, bg);
    ui.painter().text(rect.center(), Align2::CENTER_CENTER, text, FontId::proportional(13.0), fg);
    response
}

fn divider(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(vec2(9.0, BUTTON_H), Sense::hover());
    ui.painter().vline(rect.center().x, rect.y_range().shrink(4.0), Stroke::new(1.0, theme::BORDER));
}

// --- Painted icons ---------------------------------------------------------------

/// Blender-like select mode icons: a square with its corner / side / face highlighted.
fn paint_select_mode(p: &Painter, r: Rect, mode: SelectMode, c: Color32) {
    let faint = Stroke::new(1.0, c.gamma_multiply(0.5));
    p.rect_stroke(r, 0.0, faint, egui::StrokeKind::Inside);
    match mode {
        SelectMode::Vertex => {
            p.circle_filled(r.left_top(), 3.0, c);
            p.circle_filled(r.right_bottom(), 3.0, c);
        }
        SelectMode::Edge => {
            p.line_segment([r.left_top(), r.left_bottom()], Stroke::new(2.5, c));
        }
        SelectMode::Face => {
            p.rect_filled(r.shrink(1.0), 0.0, c.gamma_multiply(0.8));
        }
    }
}

fn paint_grid(p: &Painter, r: Rect, c: Color32) {
    let s = Stroke::new(1.2, c);
    for t in [0.0, 0.5, 1.0] {
        p.vline(r.left() + r.width() * t, r.y_range(), s);
        p.hline(r.x_range(), r.top() + r.height() * t, s);
    }
}

fn paint_snap(p: &Painter, r: Rect, c: Color32) {
    // Dotted grid with a cursor dot snapped on a point.
    for i in 0..3 {
        for j in 0..3 {
            let pt = r.left_top() + vec2(r.width() * i as f32 / 2.0, r.height() * j as f32 / 2.0);
            p.circle_filled(pt, 1.2, c.gamma_multiply(0.7));
        }
    }
    p.circle_stroke(r.center(), 3.5, Stroke::new(1.5, c));
}

fn paint_player(p: &Painter, r: Rect, c: Color32) {
    let s = Stroke::new(1.5, c);
    let head = r.center_top() + vec2(0.0, 2.5);
    p.circle_stroke(head, 2.5, s);
    p.line_segment([head + vec2(0.0, 2.5), r.center() + vec2(0.0, 2.0)], s);
    p.line_segment([r.center() + vec2(0.0, 2.0), r.left_bottom() + vec2(3.0, 0.0)], s);
    p.line_segment([r.center() + vec2(0.0, 2.0), r.right_bottom() - vec2(3.0, 0.0)], s);
    p.line_segment([r.left_center() + vec2(2.0, -1.0), r.right_center() + vec2(-2.0, -1.0)], s);
}

fn paint_frame(p: &Painter, r: Rect, c: Color32) {
    let s = Stroke::new(1.5, c);
    let k = 4.0;
    for (corner, dx, dy) in [
        (r.left_top(), k, k),
        (r.right_top(), -k, k),
        (r.left_bottom(), k, -k),
        (r.right_bottom(), -k, -k),
    ] {
        p.line_segment([corner, corner + vec2(dx, 0.0)], s);
        p.line_segment([corner, corner + vec2(0.0, dy)], s);
    }
    p.circle_filled(r.center(), 2.0, c);
}
