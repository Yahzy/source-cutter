//! Draws objects with flat shading (painter's algorithm) and editor overlays.

use eframe::egui::{Align2, Color32, FontId, Painter, Shape, Stroke};
use glam::Vec3;

use super::Projector;
use crate::math::Aabb;
use crate::scene::{Object, Scene};

const SELECTED_TINT: Color32 = Color32::from_rgb(230, 80, 70);
const SELECTED_EDGE: Color32 = Color32::from_rgb(255, 120, 100);
const ACTIVE_EDGE: Color32 = Color32::from_rgb(255, 190, 120);
const PREVIEW_EDGE: Color32 = Color32::from_rgb(255, 230, 120);
const PLAYER_EDGE: Color32 = Color32::from_rgb(80, 200, 220);
const CURSOR: Color32 = Color32::from_rgb(255, 255, 255);

/// Fixed light direction used for flat shading.
fn light_dir() -> Vec3 {
    Vec3::new(0.35, 0.25, 0.9).normalize()
}

/// A visible face waiting to be sorted and drawn.
pub struct FaceDraw {
    pub depth: f32,
    pub points: Vec<Vec3>,
    pub fill: Color32,
    pub edge: Stroke,
}

pub fn draw_objects(painter: &Painter, proj: &Projector, scene: &Scene) {
    let mut faces = Vec::new();
    for object in scene.objects() {
        let base = object_color(object, scene.selection.contains(object.id));
        collect_faces(proj, object, base, &mut faces);
    }
    draw_sorted(painter, proj, faces);

    // Selection outline drawn on top, like Hammer.
    for object in scene.objects().iter().filter(|o| scene.selection.contains(o.id)) {
        let color = if scene.selection.active == Some(object.id) { ACTIVE_EDGE } else { SELECTED_EDGE };
        draw_object_edges(painter, proj, object, Stroke::new(1.5, color));
    }
}

pub fn object_color(object: &Object, selected: bool) -> Color32 {
    let [r, g, b] = object.color;
    let base = Color32::from_rgb(r, g, b);
    if selected { lerp_color(base, SELECTED_TINT, 0.45) } else { base }
}

/// Collects the front-facing faces of an object with flat shading.
pub fn collect_faces(proj: &Projector, object: &Object, base: Color32, out: &mut Vec<FaceDraw>) {
    let mesh = &object.mesh;
    let edge = Stroke::new(1.0, base.gamma_multiply(0.45).to_opaque());
    for f in 0..mesh.faces.len() {
        let center = mesh.face_center(f);
        let normal = mesh.face_normal(f);
        if !proj.is_front_facing(center, normal) {
            continue;
        }
        let light = 0.55 + 0.45 * normal.dot(light_dir()).max(0.0);
        out.push(FaceDraw {
            depth: proj.depth(center),
            points: mesh.face_points(f),
            fill: base.gamma_multiply(light).to_opaque(),
            edge,
        });
    }
}

/// Painter's algorithm: farthest faces first.
pub fn draw_sorted(painter: &Painter, proj: &Projector, mut faces: Vec<FaceDraw>) {
    faces.sort_by(|a, b| b.depth.total_cmp(&a.depth));
    for face in faces {
        let pts = proj.polygon(&face.points);
        if pts.len() >= 3 {
            painter.add(Shape::convex_polygon(pts, face.fill, face.edge));
        }
    }
}

fn draw_object_edges(painter: &Painter, proj: &Projector, object: &Object, stroke: Stroke) {
    let verts = &object.mesh.vertices;
    for e in object.mesh.edges() {
        if let Some(seg) = proj.segment(verts[e.0 as usize], verts[e.1 as usize]) {
            painter.line_segment(seg, stroke);
        }
    }
}

/// Box being drawn by the Block tool, with its dimensions.
pub fn draw_block_preview(painter: &Painter, proj: &Projector, bounds: &Aabb) {
    draw_wire_box(painter, proj, bounds, Stroke::new(1.5, PREVIEW_EDGE));
    let size = bounds.size();
    if let Some(pos) = proj.point(bounds.center()) {
        painter.text(
            pos,
            Align2::CENTER_CENTER,
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

/// Small cross showing the snapped point under the mouse, drawn on the work plane.
pub fn draw_cursor(painter: &Painter, proj: &Projector, p: Vec3, axes: [Vec3; 2], size: f32) {
    let stroke = Stroke::new(1.5, CURSOR);
    for axis in axes {
        if let Some(seg) = proj.segment(p - axis * size, p + axis * size) {
            painter.line_segment(seg, stroke);
        }
    }
}

pub fn draw_wire_box(painter: &Painter, proj: &Projector, bounds: &Aabb, stroke: Stroke) {
    let corners = bounds.corners();
    for (a, b) in Aabb::EDGES {
        if let Some(seg) = proj.segment(corners[a], corners[b]) {
            painter.line_segment(seg, stroke);
        }
    }
}

pub fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let mix = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
    Color32::from_rgb(mix(a.r(), b.r()), mix(a.g(), b.g()), mix(a.b(), b.b()))
}
