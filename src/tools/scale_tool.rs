//! Scale tool, Hammer style: drag a handle on one side of the selection bounds;
//! the opposite side stays in place and the moved side snaps to the grid.

use eframe::egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Stroke, vec2};
use glam::Vec3;

use super::ToolInput;
use super::select::click_select;
use super::transform::{TransformTarget, selection_bounds};
use crate::editor::History;
use crate::math::Aabb;
use crate::scene::Scene;
use crate::viewport::{Projector, axis_color, draw_wire_box};

/// Handles are pushed this many pixels outside the bounds.
const HANDLE_OFFSET_PX: f32 = 14.0;
const HOVER: Color32 = Color32::from_rgb(255, 220, 90);
const BOUNDS: Color32 = Color32::from_rgb(255, 230, 120);

/// One side of the bounds: axis (0 = X, 1 = Y, 2 = Z) and `true` for the max side.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Handle {
    axis: usize,
    max_side: bool,
}

struct Drag {
    handle: Handle,
    target: TransformTarget,
    /// Mouse position along the axis when the drag started.
    grab: f32,
}

#[derive(Default)]
pub struct ScaleTool {
    drag: Option<Drag>,
    hovered: Option<Handle>,
}

impl ScaleTool {
    pub fn update(&mut self, input: &ToolInput, scene: &mut Scene, history: &mut History) {
        let bounds = selection_bounds(scene);
        self.hovered = match (bounds, input.pointer) {
            (Some(b), Some(p)) if self.drag.is_none() => pick_handle(input.proj, &b, p),
            _ => self.drag.as_ref().map(|d| d.handle),
        };

        if input.pressed
            && let (Some(handle), Some(bounds)) = (self.hovered, bounds)
            && let Some(target) = TransformTarget::capture(scene)
            && let Some(grab) = grab_along_axis(input, &bounds, handle)
        {
            history.checkpoint(scene);
            self.drag = Some(Drag { handle, target, grab });
        }

        if let Some(drag) = &self.drag {
            if input.down
                && let Some(s) = grab_along_axis(input, &drag.target.bounds, drag.handle)
            {
                apply_scale(input, scene, drag, s - drag.grab);
            }
            if input.released {
                self.drag = None;
            }
        } else if input.clicked {
            click_select(input, scene);
        }
    }

    pub fn cancel(&mut self, scene: &mut Scene, history: &mut History) -> bool {
        let Some(drag) = self.drag.take() else { return false };
        drag.target.restore(scene);
        history.discard_last();
        true
    }

    pub fn draw(&self, painter: &Painter, proj: &Projector, scene: &Scene) {
        let Some(bounds) = selection_bounds(scene) else { return };
        draw_wire_box(painter, proj, &bounds, Stroke::new(1.0, BOUNDS.gamma_multiply(0.7)));

        for handle in handles(&bounds) {
            if let Some(p) = proj.point(handle_pos(proj, &bounds, handle)) {
                let c = if self.hovered == Some(handle) { HOVER } else { axis_color(axis(handle.axis)) };
                painter.rect_filled(Rect::from_center_size(p, vec2(10.0, 10.0)), 1.0, c);
                painter.rect_stroke(
                    Rect::from_center_size(p, vec2(10.0, 10.0)),
                    1.0,
                    Stroke::new(1.0, Color32::BLACK),
                    eframe::egui::StrokeKind::Outside,
                );
            }
        }

        if self.drag.is_some() {
            let s = bounds.size();
            if let Some(p) = proj.point(bounds.center()) {
                painter.text(p, Align2::CENTER_CENTER, format!("{} × {} × {}", s.x, s.y, s.z), FontId::monospace(13.0), BOUNDS);
            }
        }
    }
}

fn axis(k: usize) -> Vec3 {
    [Vec3::X, Vec3::Y, Vec3::Z][k]
}

/// Handles on every side that has a size (a flat selection can't be scaled on its thin axis).
fn handles(bounds: &Aabb) -> impl Iterator<Item = Handle> + '_ {
    (0..3)
        .filter(|&k| bounds.size()[k] > 0.0)
        .flat_map(|axis| [false, true].map(|max_side| Handle { axis, max_side }))
}

fn side_center(bounds: &Aabb, handle: Handle) -> Vec3 {
    let mut p = bounds.center();
    p[handle.axis] = if handle.max_side { bounds.max[handle.axis] } else { bounds.min[handle.axis] };
    p
}

fn handle_pos(proj: &Projector, bounds: &Aabb, handle: Handle) -> Vec3 {
    let p = side_center(bounds, handle);
    let sign = if handle.max_side { 1.0 } else { -1.0 };
    p + axis(handle.axis) * sign * proj.pixel_size_at(p) * HANDLE_OFFSET_PX
}

fn pick_handle(proj: &Projector, bounds: &Aabb, pointer: Pos2) -> Option<Handle> {
    handles(bounds)
        .filter_map(|h| proj.point(handle_pos(proj, bounds, h)).map(|p| (p.distance(pointer), h)))
        .filter(|(d, _)| *d <= 9.0)
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, h)| h)
}

/// Mouse position projected on the handle's axis line.
fn grab_along_axis(input: &ToolInput, bounds: &Aabb, handle: Handle) -> Option<f32> {
    input.ray?.closest_on_line(side_center(bounds, handle), axis(handle.axis))
}

fn apply_scale(input: &ToolInput, scene: &mut Scene, drag: &Drag, raw: f32) {
    let k = drag.handle.axis;
    let bounds = drag.target.bounds;
    let (side, pivot) =
        if drag.handle.max_side { (bounds.max[k], bounds.min[k]) } else { (bounds.min[k], bounds.max[k]) };

    // Snap the moved side, and never flip or collapse the selection.
    let mut new_side = input.grid.snap_value(side + raw);
    new_side = if drag.handle.max_side { new_side.max(pivot + 1.0) } else { new_side.min(pivot - 1.0) };

    let factor = (new_side - pivot) / (side - pivot);
    drag.target.apply(scene, |mut p| {
        p[k] = pivot + (p[k] - pivot) * factor;
        p
    });
}
