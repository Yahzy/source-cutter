//! Hammer-style ground grid on the z = 0 plane.

use eframe::egui::{Color32, Painter, Stroke};
use glam::Vec3;

use super::Projector;
use crate::editor::{GridSettings, OrbitCamera};
use crate::units::{GRID_HIGHLIGHT_STEP, GRID_MAJOR_STEP, MAP_HALF_EXTENT};

/// Maximum number of lines per axis; finer levels are skipped beyond that (like Hammer).
const MAX_LINES: f32 = 160.0;
/// Each line is cut into segments so it can fade out with distance.
const SEGMENTS: usize = 24;

const MINOR: Color32 = Color32::from_rgb(58, 59, 64);
const HIGHLIGHT: Color32 = Color32::from_rgb(86, 88, 96);
const MAJOR: Color32 = Color32::from_rgb(135, 95, 45);
const AXIS_X: Color32 = Color32::from_rgb(200, 60, 70);
const AXIS_Y: Color32 = Color32::from_rgb(110, 180, 40);
const MAP_BOUNDS: Color32 = Color32::from_rgb(180, 40, 40);

pub fn draw(painter: &Painter, proj: &Projector, camera: &OrbitCamera, grid: &GridSettings) {
    // Visible area around the camera target, limited to the map bounds.
    let half_span = (camera.distance * 4.0).clamp(256.0, MAP_HALF_EXTENT);
    let center = camera.target;

    // Skip the finest levels when lines would be too dense.
    let mut step = grid.size() as f32;
    while half_span * 2.0 / step > MAX_LINES {
        step *= 2.0;
    }

    let min_x = ((center.x - half_span) / step).floor() as i64;
    let max_x = ((center.x + half_span) / step).ceil() as i64;
    let min_y = ((center.y - half_span) / step).floor() as i64;
    let max_y = ((center.y + half_span) / step).ceil() as i64;
    let clamp = |v: f32| v.clamp(-MAP_HALF_EXTENT, MAP_HALF_EXTENT);

    let fade = Fade { center, radius: half_span };

    // Lines parallel to Y (constant x).
    for i in min_x..=max_x {
        let x = i as f32 * step;
        if x.abs() > MAP_HALF_EXTENT {
            continue;
        }
        let a = Vec3::new(x, clamp(center.y - half_span), 0.0);
        let b = Vec3::new(x, clamp(center.y + half_span), 0.0);
        draw_faded_line(painter, proj, &fade, a, b, line_color(x, AXIS_Y));
    }

    // Lines parallel to X (constant y).
    for i in min_y..=max_y {
        let y = i as f32 * step;
        if y.abs() > MAP_HALF_EXTENT {
            continue;
        }
        let a = Vec3::new(clamp(center.x - half_span), y, 0.0);
        let b = Vec3::new(clamp(center.x + half_span), y, 0.0);
        draw_faded_line(painter, proj, &fade, a, b, line_color(y, AXIS_X));
    }

    draw_map_bounds(painter, proj);
}

/// Picks the line color depending on its coordinate, like Hammer.
fn line_color(coord: f32, axis_color: Color32) -> Color32 {
    if coord == 0.0 {
        axis_color
    } else if coord % GRID_MAJOR_STEP == 0.0 {
        MAJOR
    } else if coord % GRID_HIGHLIGHT_STEP == 0.0 {
        HIGHLIGHT
    } else {
        MINOR
    }
}

struct Fade {
    center: Vec3,
    radius: f32,
}

impl Fade {
    fn alpha(&self, p: Vec3) -> f32 {
        let d = (p - self.center).truncate().length() / self.radius;
        (1.0 - d * d).clamp(0.0, 1.0)
    }
}

fn draw_faded_line(painter: &Painter, proj: &Projector, fade: &Fade, a: Vec3, b: Vec3, color: Color32) {
    for s in 0..SEGMENTS {
        let p0 = a.lerp(b, s as f32 / SEGMENTS as f32);
        let p1 = a.lerp(b, (s + 1) as f32 / SEGMENTS as f32);
        let alpha = fade.alpha((p0 + p1) * 0.5);
        if alpha <= 0.01 {
            continue;
        }
        if let Some(seg) = proj.segment(p0, p1) {
            painter.line_segment(seg, Stroke::new(1.0, color.gamma_multiply(alpha)));
        }
    }
}

fn draw_map_bounds(painter: &Painter, proj: &Projector) {
    let e = MAP_HALF_EXTENT;
    let corners = [
        Vec3::new(-e, -e, 0.0),
        Vec3::new(e, -e, 0.0),
        Vec3::new(e, e, 0.0),
        Vec3::new(-e, e, 0.0),
    ];
    for i in 0..4 {
        if let Some(seg) = proj.segment(corners[i], corners[(i + 1) % 4]) {
            painter.line_segment(seg, Stroke::new(1.5, MAP_BOUNDS));
        }
    }
}
