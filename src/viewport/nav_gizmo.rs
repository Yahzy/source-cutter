//! Blender-like navigation gizmo in the top-right corner of the viewport.
//! Click an axis bubble to look from that side, drag the gizmo to orbit.

use eframe::egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Ui, Vec2, vec2};
use glam::Vec3;

use super::Projector;

const RADIUS: f32 = 42.0;
const BUBBLE: f32 = 9.0;
const MARGIN: f32 = 16.0;

const X_COLOR: Color32 = Color32::from_rgb(230, 70, 80);
const Y_COLOR: Color32 = Color32::from_rgb(130, 200, 40);
const Z_COLOR: Color32 = Color32::from_rgb(70, 140, 240);

pub enum GizmoAction {
    /// Look from this side (unit axis).
    LookFrom(Vec3),
    /// Orbit by this mouse delta in pixels.
    Orbit(Vec2),
}

/// Screen area of the gizmo inside the viewport.
pub fn rect(viewport: Rect) -> Rect {
    let size = 2.0 * (RADIUS + BUBBLE + 4.0);
    let center = viewport.right_top() + vec2(-MARGIN - size * 0.5, MARGIN + size * 0.5);
    Rect::from_center_size(center, Vec2::splat(size))
}

struct Bubble {
    axis: Vec3,
    label: &'static str,
    color: Color32,
    positive: bool,
    screen: Pos2,
    depth: f32,
}

pub fn show(ui: &mut Ui, viewport: Rect, proj: &Projector) -> Option<GizmoAction> {
    let area = rect(viewport);
    let center = area.center();
    let response = ui.interact(area, ui.scope_id().with("nav_gizmo"), Sense::click_and_drag());
    let painter = ui.painter_at(viewport);

    let mut bubbles: Vec<Bubble> = [(Vec3::X, "X", X_COLOR), (Vec3::Y, "Y", Y_COLOR), (Vec3::Z, "Z", Z_COLOR)]
        .into_iter()
        .flat_map(|(axis, label, color)| [(axis, label, color, true), (-axis, label, color, false)])
        .map(|(axis, label, color, positive)| {
            let v = proj.view_direction(axis);
            Bubble { axis, label, color, positive, screen: center + vec2(v.x, -v.y) * RADIUS, depth: v.z }
        })
        .collect();
    // Farthest first so the closest bubbles are drawn on top.
    bubbles.sort_by(|a, b| a.depth.total_cmp(&b.depth));

    // Bubble under the mouse (the top-most one wins).
    let hovered = response
        .hover_pos()
        .and_then(|p| bubbles.iter().rposition(|b| b.screen.distance(p) <= BUBBLE + 1.0));

    if response.hovered() || response.dragged() {
        painter.circle_filled(center, RADIUS + BUBBLE + 4.0, Color32::from_white_alpha(18));
    }

    for (i, b) in bubbles.iter().enumerate() {
        let is_hovered = hovered == Some(i);
        if b.positive {
            painter.line_segment([center, b.screen], Stroke::new(2.0, b.color));
            painter.circle_filled(b.screen, BUBBLE, b.color);
            painter.text(b.screen, Align2::CENTER_CENTER, b.label, FontId::proportional(11.0), Color32::BLACK);
        } else {
            painter.circle_filled(b.screen, BUBBLE - 1.0, b.color.gamma_multiply(0.35));
            painter.circle_stroke(b.screen, BUBBLE - 1.0, Stroke::new(1.0, b.color.gamma_multiply(0.7)));
        }
        if is_hovered {
            painter.circle_stroke(b.screen, BUBBLE + 1.0, Stroke::new(1.5, Color32::WHITE));
        }
    }

    if response.clicked()
        && let Some(i) = hovered
    {
        return Some(GizmoAction::LookFrom(bubbles[i].axis));
    }
    if response.dragged() {
        return Some(GizmoAction::Orbit(response.drag_delta()));
    }
    None
}
