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

use eframe::egui::{Align2, Color32, FontId, PointerButton, Rect, Response, Sense, Ui, vec2};
use glam::Vec3;

use crate::editor::Editor;
use crate::tools::{ToolInput, ToolKind};
use nav_gizmo::GizmoAction;
use projector::Projector;

const BACKGROUND: Color32 = Color32::from_rgb(38, 39, 43);

pub fn show(ui: &mut Ui, editor: &mut Editor) {
    let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, BACKGROUND);

    handle_navigation(ui, &response, editor, rect);
    let proj = Projector::new(&editor.camera, rect);

    update_tools(ui, &response, editor, &proj, rect);

    // --- Rendering ---
    if editor.grid.visible {
        draw_grid::draw(&painter, &proj, &editor.camera, &editor.grid);
    }
    draw_scene::draw_objects(&painter, &proj, &editor.scene);
    if editor.view.show_player_reference {
        draw_scene::draw_player_reference(&painter, &proj, &Editor::player_reference_bounds());
    }
    if editor.tools.active == ToolKind::Block {
        if let Some(bounds) = editor.tools.block.preview() {
            draw_scene::draw_block_preview(&painter, &proj, &bounds);
        }
        if let Some(cursor) = editor.cursor {
            draw_scene::draw_cursor(&painter, &proj, cursor, [Vec3::X, Vec3::Y], editor.grid.size() as f32 * 0.5);
        }
    }

    draw_overlay_text(&painter, rect, editor);

    // Gizmo last so it is drawn on top.
    match nav_gizmo::show(ui, rect, &proj) {
        Some(GizmoAction::LookFrom(dir)) => editor.camera.look_from(dir),
        Some(GizmoAction::Orbit(delta)) => editor.camera.orbit(delta.x, delta.y),
        None => {}
    }
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

    let over_gizmo = pointer.is_some_and(|p| nav_gizmo::rect(rect).contains(p));
    let in_viewport = response.hovered() && !over_gizmo;

    let ray = pointer.map(|p| proj.ray(p));
    let ground = ray.and_then(|r| r.intersect_plane(Vec3::ZERO, Vec3::Z).map(|t| editor.grid.snap_point(r.at(t))));
    editor.cursor = if response.hovered() { ground } else { None };

    let input = ToolInput {
        ray,
        ground,
        pressed: pressed && in_viewport,
        down,
        released,
        clicked: response.clicked() && !over_gizmo,
        shift,
    };
    editor.tools.update(&input, &mut editor.scene);
}

fn draw_overlay_text(painter: &eframe::egui::Painter, rect: Rect, editor: &Editor) {
    let text = format!(
        "Perspective  ·  Grid {}{}  ·  {}",
        editor.grid.size(),
        if editor.grid.snap { "" } else { " (snap off)" },
        editor.tools.active.label(),
    );
    painter.text(
        rect.left_top() + vec2(10.0, 8.0),
        Align2::LEFT_TOP,
        text,
        FontId::proportional(13.0),
        Color32::from_gray(190),
    );
}
