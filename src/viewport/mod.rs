//! The 3D viewport: camera navigation, tool input and rendering.
//!
//! Controls (Blender-like):
//! - Middle mouse drag        : orbit
//! - Shift + middle mouse drag: pan
//! - Mouse wheel              : zoom
//! - Left mouse               : active tool

mod draw_grid;
mod draw_scene;
mod nav_gizmo;
mod projector;
mod visibility;

pub use draw_grid::axis_color;
pub use draw_scene::{draw_block_preview, draw_wire_box};
pub use projector::Projector;
pub use visibility::point_visible;

use eframe::egui::{Align2, Color32, FontId, PointerButton, Rect, Response, Sense, Ui, vec2};
use glam::Vec3;

use crate::editor::{Editor, GridPlane};
use crate::tools::{ToolInput, ToolKind};
use nav_gizmo::GizmoAction;

const BACKGROUND: Color32 = Color32::from_rgb(38, 39, 43);

/// Draws the viewport and returns its screen rectangle.
pub fn show(ui: &mut Ui, editor: &mut Editor) -> Rect {
    let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, BACKGROUND);

    handle_navigation(ui, &response, editor, rect);
    let proj = Projector::new(&editor.camera, rect);

    update_tools(ui, &response, editor, &proj, rect);

    // --- Rendering ---
    if editor.grid.visible {
        draw_grid::draw(&painter, &proj, &editor.camera, &editor.grid, GridPlane::from_camera(&editor.camera));
    }
    draw_scene::draw_objects(&painter, &proj, &editor.scene, editor.edit_context());
    if editor.view.show_player_reference {
        draw_scene::draw_player_reference(&painter, &proj, &Editor::player_reference_bounds());
    }
    editor.tools.draw(&painter, &proj, &editor.scene, editor.edit_context());
    if editor.tools.active == ToolKind::Block {
        if let Some(cursor) = editor.cursor {
            let axes = GridPlane::from_camera(&editor.camera).axes();
            draw_scene::draw_cursor(&painter, &proj, cursor, axes, editor.grid.size() as f32 * 0.5);
        }
    }

    draw_overlay_text(&painter, rect, editor);

    // Gizmo last so it is drawn on top.
    match nav_gizmo::show(ui, rect, &proj, editor.camera.orthographic) {
        Some(GizmoAction::LookFrom(dir)) => editor.camera.look_from(dir),
        Some(GizmoAction::Orbit(delta)) => editor.camera.orbit(delta.x, delta.y),
        Some(GizmoAction::ToggleProjection) => editor.camera.toggle_projection(),
        None => {}
    }
    rect
}

fn handle_navigation(ui: &Ui, response: &Response, editor: &mut Editor, rect: Rect) {
    if response.dragged_by(PointerButton::Middle) {
        let delta = response.drag_delta();
        if ui.input(|i| i.modifiers.shift) {
            editor.camera.pan(delta.x, delta.y, rect.height());
        } else {
            editor.camera.orbit(delta.x, delta.y);
        }
    }

    if response.hovered() {
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll != 0.0 {
            editor.camera.zoom(scroll);
        }
    }
}

fn update_tools(ui: &Ui, response: &Response, editor: &mut Editor, proj: &Projector, rect: Rect) {
    let (pointer, pressed, down, released, shift) = ui.input(|i| {
        (
            i.pointer.latest_pos(),
            i.pointer.primary_pressed(),
            i.pointer.primary_down(),
            i.pointer.primary_released(),
            i.modifiers.shift,
        )
    });

    let over_gizmo = pointer.is_some_and(|p| nav_gizmo::contains(rect, p));
    let in_viewport = response.hovered() && !over_gizmo;

    let plane = GridPlane::from_camera(&editor.camera);
    let ray = pointer.map(|p| proj.ray(p));
    let on_plane = ray.and_then(|r| {
        let t = r.intersect_plane(Vec3::ZERO, plane.normal())?;
        Some(editor.grid.snap_point(r.at(t)))
    });
    editor.cursor = if response.hovered() { on_plane } else { None };

    let input = ToolInput {
        proj,
        grid: &editor.grid,
        pointer,
        ray,
        plane,
        on_plane,
        pressed: pressed && in_viewport,
        down,
        released,
        clicked: response.clicked() && !over_gizmo,
        shift,
        edit: editor.edit_context(),
    };
    editor.tools.update(&input, &mut editor.scene, &mut editor.history);
}

fn draw_overlay_text(painter: &eframe::egui::Painter, rect: Rect, editor: &Editor) {
    let text = format!(
        "{}  ·  {}  ·  Grid {}{}  ·  {}",
        editor.camera.view_name(),
        editor.mode().label(),
        editor.grid.size(),
        if editor.grid.snap { "" } else { " (snap off)" },
        editor.tools.active.label(),
    );
    painter.text(
        // Below the floating toolbar.
        rect.left_top() + vec2(12.0, crate::ui::toolbar::HEIGHT + 18.0),
        Align2::LEFT_TOP,
        text,
        FontId::proportional(13.0),
        Color32::from_gray(190),
    );
}
