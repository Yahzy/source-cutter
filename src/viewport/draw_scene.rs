//! Draws brushes with flat shading (painter's algorithm) and editor overlays.

use eframe::egui::{Align2, Color32, FontId, Painter, Shape, Stroke};
use glam::Vec3;

use super::Projector;
use crate::math::Aabb;
use crate::scene::Scene;

const SELECTED_TINT: Color32 = Color32::from_rgb(230, 80, 70);
const SELECTED_EDGE: Color32 = Color32::from_rgb(255, 120, 100);
const PREVIEW_EDGE: Color32 = Color32::from_rgb(255, 230, 120);
const PLAYER_EDGE: Color32 = Color32::from_rgb(80, 200, 220);
const CURSOR: Color32 = Color32::from_rgb(255, 255, 255);

/// Fixed light direction used for flat shading.
fn light_dir() -> Vec3 {
    Vec3::new(0.35, 0.25, 0.9).normalize()
}

/// A visible face waiting to be sorted and drawn.
struct FaceDraw {
    depth: f32,
    points: [Vec3; 4],
    fill: Color32,
    edge: Stroke,
}

pub fn draw_brushes(painter: &Painter, proj: &Projector, scene: &Scene) {
    let eye = proj.eye();
    let selected = scene.selected_id();
    let mut faces = Vec::new();

    for brush in scene.brushes() {
        let is_selected = Some(brush.id) == selected;
        let [r, g, b] = brush.color;
        let base = if is_selected {
            lerp_color(Color32::from_rgb(r, g, b), SELECTED_TINT, 0.45)
        } else {
            Color32::from_rgb(r, g, b)
        };
        let edge = Stroke::new(1.0, base.gamma_multiply(0.45).to_opaque());
        collect_faces(&brush.bounds, eye, base, edge, &mut faces);
    }

    // Painter's algorithm: farthest faces first.
    faces.sort_by(|a, b| b.depth.total_cmp(&a.depth));
    for face in faces {
        let pts = proj.polygon(&face.points);
        if pts.len() >= 3 {
            painter.add(Shape::convex_polygon(pts, face.fill, face.edge));
        }
    }

    // Selection outline drawn on top, like Hammer.
    if let Some(brush) = scene.selected() {
        draw_wire_box(painter, proj, &brush.bounds, Stroke::new(1.5, SELECTED_EDGE));
    }
}

fn collect_faces(bounds: &Aabb, eye: Vec3, base: Color32, edge: Stroke, out: &mut Vec<FaceDraw>) {
    let corners = bounds.corners();
    for face in &Aabb::FACES {
        let points = face.corners.map(|i| corners[i]);
        let center = points.iter().copied().sum::<Vec3>() * 0.25;
        // Back-face culling.
        if face.normal.dot(eye - center) <= 0.0 {
            continue;
        }
        let light = 0.55 + 0.45 * face.normal.dot(light_dir()).max(0.0);
        out.push(FaceDraw {
            depth: (center - eye).length_squared(),
            points,
            fill: base.gamma_multiply(light).to_opaque(),
            edge,
        });
    }
}

/// Box being drawn by the Block tool, with its dimensions.
pub fn draw_block_preview(painter: &Painter, proj: &Projector, bounds: &Aabb) {
    draw_wire_box(painter, proj, bounds, Stroke::new(1.5, PREVIEW_EDGE));
    let size = bounds.size();
    let label_pos = Vec3::new(bounds.center().x, bounds.center().y, bounds.max.z);
    if let Some(pos) = proj.point(label_pos) {
        painter.text(
            pos,
            Align2::CENTER_BOTTOM,
            format!("{} × {} × {}", size.x, size.y, size.z),
            FontId::monospace(13.0),
            PREVIEW_EDGE,
        );
    }
}

/// 32×32×72 player hull used as a scale reference.
pub fn draw_player_reference(painter: &Painter, proj: &Projector, bounds: &Aabb) {
    draw_wire_box(painter, proj, bounds, Stroke::new(1.0, PLAYER_EDGE.gamma_multiply(0.8)));
    let top = Vec3::new(bounds.center().x, bounds.center().y, bounds.max.z);
    if let Some(pos) = proj.point(top) {
        painter.text(pos, Align2::CENTER_BOTTOM, "Player 72u", FontId::proportional(11.0), PLAYER_EDGE);
    }
}

/// Small cross showing the snapped point under the mouse.
pub fn draw_cursor(painter: &Painter, proj: &Projector, p: Vec3, size: f32) {
    let stroke = Stroke::new(1.5, CURSOR);
    for axis in [Vec3::X, Vec3::Y] {
        if let Some(seg) = proj.segment(p - axis * size, p + axis * size) {
            painter.line_segment(seg, stroke);
        }
    }
}

fn draw_wire_box(painter: &Painter, proj: &Projector, bounds: &Aabb, stroke: Stroke) {
    let corners = bounds.corners();
    for (a, b) in Aabb::EDGES {
        if let Some(seg) = proj.segment(corners[a], corners[b]) {
            painter.line_segment(seg, stroke);
        }
    }
}

fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let mix = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
    Color32::from_rgb(mix(a.r(), b.r()), mix(a.g(), b.g()), mix(a.b(), b.b()))
}
