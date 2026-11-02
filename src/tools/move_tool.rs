//! Move tool: axis arrows, plane handles and a center handle, snapped to the grid.

use eframe::egui::{Color32, Painter, Pos2, Rect, Stroke, vec2};
use glam::Vec3;

use super::ToolInput;
use super::select::click_select;
use super::transform::{TransformTarget, distance_to_segment, selection_bounds};
use crate::editor::History;
use crate::scene::{Scene, SelectMode};
use crate::viewport::{Projector, axis_color};

/// Arrow length in pixels.
const ARROW_PX: f32 = 90.0;
const HOVER: Color32 = Color32::from_rgb(255, 220, 90);

#[derive(Clone, Copy, Debug, PartialEq)]
enum Handle {
    /// Move along one world axis (0 = X, 1 = Y, 2 = Z).
    Axis(usize),
    /// Move in the plane whose normal is this axis.
    Plane(usize),
    /// Move freely on the work plane.
    Center,
}

struct Drag {
    handle: Handle,
    target: TransformTarget,
    /// Point grabbed on the axis / plane when the drag started.
    grab: Vec3,
    plane_normal: Vec3,
}

#[derive(Default)]
pub struct MoveTool {
    drag: Option<Drag>,
    hovered: Option<Handle>,
}

impl MoveTool {
    pub fn update(&mut self, input: &ToolInput, scene: &mut Scene, history: &mut History) {
        let pivot = selection_bounds(scene, input.edit).map(|b| b.center());
        self.hovered = match (pivot, input.pointer) {
            (Some(pivot), Some(p)) if self.drag.is_none() => pick_handle(input.proj, pivot, p),
            _ => self.drag.as_ref().map(|d| d.handle),
        };

        if input.pressed
            && let (Some(handle), Some(pivot)) = (self.hovered, pivot)
        {
            self.start_drag(input, scene, history, handle, pivot);
        }

        if let Some(drag) = &self.drag {
            if input.down
                && let Some(point) = grab_point(input, drag.handle, drag.target.bounds.center(), drag.plane_normal)
            {
                let delta = snapped_delta(input, drag, point - drag.grab);
                drag.target.apply(scene, |p| p + delta);
            }
            if input.released {
                self.drag = None;
            }
        } else if input.clicked {
            click_select(input, scene);
        }
    }

    fn start_drag(&mut self, input: &ToolInput, scene: &Scene, history: &mut History, handle: Handle, pivot: Vec3) {
        let Some(target) = TransformTarget::capture(scene, input.edit) else { return };
        let plane_normal = match handle {
            Handle::Plane(k) => axis(k),
            _ => input.plane.normal(),
        };
        if let Some(grab) = grab_point(input, handle, pivot, plane_normal) {
            history.checkpoint(scene);
            self.drag = Some(Drag { handle, target, grab, plane_normal });
        }
    }

    pub fn cancel(&mut self, scene: &mut Scene, history: &mut History) -> bool {
        let Some(drag) = self.drag.take() else { return false };
        drag.target.restore(scene);
        history.discard_last();
        true
    }

    pub fn draw(&self, painter: &Painter, proj: &Projector, scene: &Scene, edit: Option<SelectMode>) {
        let Some(pivot) = selection_bounds(scene, edit).map(|b| b.center()) else { return };
        let Some(center) = proj.point(pivot) else { return };
        let len = proj.pixel_size_at(pivot) * ARROW_PX;
        let color = |h: Handle, base: Color32| if self.hovered == Some(h) { HOVER } else { base };

        // Plane handles: small squares between two axes.
        for k in 0..3 {
            if !plane_visible(proj, k) {
                continue;
            }
            if let Some(p) = proj.point(plane_handle_pos(pivot, k, len)) {
                let c = color(Handle::Plane(k), axis_color(axis(k)).gamma_multiply(0.8));
                painter.rect_filled(Rect::from_center_size(p, vec2(12.0, 12.0)), 1.0, c);
            }
        }
        // Axis arrows.
        for k in 0..3 {
            if let Some(end) = proj.point(pivot + axis(k) * len) {
                let c = color(Handle::Axis(k), axis_color(axis(k)));
                painter.line_segment([center, end], Stroke::new(3.0, c));
                painter.circle_filled(end, 6.0, c);
            }
        }
        let c = color(Handle::Center, Color32::WHITE);
        painter.circle_stroke(center, 8.0, Stroke::new(2.0, c));
    }
}

fn axis(k: usize) -> Vec3 {
    [Vec3::X, Vec3::Y, Vec3::Z][k]
}

fn plane_handle_pos(pivot: Vec3, normal: usize, len: f32) -> Vec3 {
    let [a, b] = [(normal + 1) % 3, (normal + 2) % 3];
    pivot + (axis(a) + axis(b)) * len * 0.3
}

/// A plane seen edge-on can't be dragged meaningfully.
fn plane_visible(proj: &Projector, normal: usize) -> bool {
    proj.view_direction(axis(normal)).z.abs() > 0.2
}

fn pick_handle(proj: &Projector, pivot: Vec3, pointer: Pos2) -> Option<Handle> {
    let center = proj.point(pivot)?;
    if pointer.distance(center) <= 10.0 {
        return Some(Handle::Center);
    }
    let len = proj.pixel_size_at(pivot) * ARROW_PX;
    for k in 0..3 {
        if plane_visible(proj, k)
            && let Some(p) = proj.point(plane_handle_pos(pivot, k, len))
            && pointer.distance(p) <= 9.0
        {
            return Some(Handle::Plane(k));
        }
    }
    (0..3).find(|&k| {
        proj.point(pivot + axis(k) * len).is_some_and(|end| {
            // Ignore axes pointing at the camera.
            end.distance(center) > 12.0 && distance_to_segment(pointer, center, end) <= 7.0
        })
    })
    .map(Handle::Axis)
}

/// Point under the mouse on the handle's axis or plane.
fn grab_point(input: &ToolInput, handle: Handle, pivot: Vec3, plane_normal: Vec3) -> Option<Vec3> {
    let ray = input.ray?;
    match handle {
        Handle::Axis(k) => ray.closest_on_line(pivot, axis(k)).map(|s| pivot + axis(k) * s),
        Handle::Plane(_) | Handle::Center => ray.intersect_plane(pivot, plane_normal).map(|t| ray.at(t)),
    }
}

/// Keeps the moved components only, and snaps the selection's min corner to the grid (like Hammer).
fn snapped_delta(input: &ToolInput, drag: &Drag, raw: Vec3) -> Vec3 {
    let min = drag.target.bounds.min;
    let mut delta = Vec3::ZERO;
    for k in 0..3 {
        let moves = match drag.handle {
            Handle::Axis(a) => a == k,
            Handle::Plane(_) | Handle::Center => drag.plane_normal[k] == 0.0,
        };
        if moves {
            delta[k] = input.grid.snap_value(min[k] + raw[k]) - min[k];
        }
    }
    delta
}
