//! Hammer-style grid, drawn on the current work plane (XY, XZ or YZ).

use eframe::egui::{Color32, Painter, Stroke};
use glam::Vec3;

use super::Projector;
use crate::editor::{GridPlane, GridSettings, OrbitCamera};
use crate::units::{GRID_HIGHLIGHT_STEP, GRID_MAJOR_STEP, MAP_HALF_EXTENT};

/// Maximum number of lines per axis; finer levels are skipped beyond that (like Hammer).
const MAX_LINES: f32 = 160.0;
/// Each line is cut into segments so it can fade out with distance (perspective only).
const SEGMENTS: usize = 24;

const MINOR: Color32 = Color32::from_rgb(58, 59, 64);
const HIGHLIGHT: Color32 = Color32::from_rgb(86, 88, 96);
const MAJOR: Color32 = Color32::from_rgb(135, 95, 45);
const MAP_BOUNDS: Color32 = Color32::from_rgb(180, 40, 40);

/// Color of the world axis line running along `axis`.
pub fn axis_color(axis: Vec3) -> Color32 {
    if axis.x != 0.0 {
        Color32::from_rgb(200, 60, 70)
    } else if axis.y != 0.0 {
        Color32::from_rgb(110, 180, 40)
    } else {
        Color32::from_rgb(60, 120, 220)
    }
}

pub fn draw(painter: &Painter, proj: &Projector, camera: &OrbitCamera, grid: &GridSettings, plane: GridPlane) {
    let [u, v] = plane.axes();
    // Visible area around the camera target, limited to the map bounds.
    let half_span = if camera.orthographic {
        (camera.half_height() * 3.0).min(MAP_HALF_EXTENT)
    } else {
        (camera.distance * 4.0).clamp(256.0, MAP_HALF_EXTENT)
    };
    let center = camera.target - plane.normal() * camera.target.dot(plane.normal());
    let fade = Fade { center, radius: if camera.orthographic { f32::INFINITY } else { half_span } };

    // Skip the finest levels when lines would be too dense.
    let mut step = grid.size() as f32;
    while half_span * 2.0 / step > MAX_LINES {
        step *= 2.0;
    }

    // Lines along `line_dir`, spaced along `across`.
    for (across, line_dir) in [(u, v), (v, u)] {
        let c = center.dot(across);
        let along = center.dot(line_dir);
        let from = (along - half_span).max(-MAP_HALF_EXTENT);
        let to = (along + half_span).min(MAP_HALF_EXTENT);
        let first = ((c - half_span) / step).floor() as i64;
        let last = ((c + half_span) / step).ceil() as i64;

        for i in first..=last {
            let coord = i as f32 * step;
            if coord.abs() > MAP_HALF_EXTENT {
                continue;
            }
            let a = across * coord + line_dir * from;
            let b = across * coord + line_dir * to;
            draw_faded_line(painter, proj, &fade, a, b, line_color(coord, axis_color(line_dir)));
        }
    }

    draw_map_bounds(painter, proj, u, v);
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
        let d = (p - self.center).length() / self.radius;
        (1.0 - d * d).clamp(0.0, 1.0)
    }
}

fn draw_faded_line(painter: &Painter, proj: &Projector, fade: &Fade, a: Vec3, b: Vec3, color: Color32) {
    if fade.radius.is_infinite() {
        if let Some(seg) = proj.segment(a, b) {
            painter.line_segment(seg, Stroke::new(1.0, color));
        }
        return;
    }
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

fn draw_map_bounds(painter: &Painter, proj: &Projector, u: Vec3, v: Vec3) {
    let e = MAP_HALF_EXTENT;
    let corners = [(-e, -e), (e, -e), (e, e), (-e, e)].map(|(a, b)| u * a + v * b);
    for i in 0..4 {
        if let Some(seg) = proj.segment(corners[i], corners[(i + 1) % 4]) {
            painter.line_segment(seg, Stroke::new(1.5, MAP_BOUNDS));
        }
    }
}
