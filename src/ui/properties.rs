//! Right panel (Noggit-like): tool settings, grid, selection, scene outline.

use eframe::egui::{self, Align, Layout, RichText, Ui};
use glam::Vec3;

use super::theme;
use crate::editor::Editor;
use crate::math::Aabb;
use crate::scene::SelectMode;
use crate::tools::ToolKind;
use crate::units;

pub fn show(ui: &mut Ui, editor: &mut Editor) {
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        section(ui, editor.tools.active.label(), |ui| tool_settings(ui, editor));
        section(ui, "Grid", |ui| grid_settings(ui, editor));
        let title = if editor.edit_context().is_some() { "Edit Selection" } else { "Selection" };
        section(ui, title, |ui| selection(ui, editor));
        scene_outline(ui, editor);
    });
}

/// Centered bold title, then the content, then a thin separator.
fn section(ui: &mut Ui, title: &str, add_contents: impl FnOnce(&mut Ui)) {
    ui.add_space(8.0);
    ui.vertical_centered(|ui| ui.label(RichText::new(title).strong().size(13.0)));
    ui.add_space(6.0);
    add_contents(ui);
    ui.add_space(8.0);
    ui.separator();
}

fn hint(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).small().color(theme::TEXT_WEAK));
}

/// "Label:" on the left, widget on the right (Noggit settings rows).
fn row<R>(ui: &mut Ui, label: &str, add: impl FnOnce(&mut Ui) -> R) -> R {
    ui.horizontal(|ui| {
        ui.label(format!("{label}:"));
        ui.with_layout(Layout::right_to_left(Align::Center), add).inner
    })
    .inner
}

// --- Tool ------------------------------------------------------------------------

fn tool_settings(ui: &mut Ui, editor: &mut Editor) {
    match editor.tools.active {
        ToolKind::Select => hint(ui, "Click to select, Shift+click to add.\nTab: Edit Mode, 1/2/3: vertex/edge/face."),
        ToolKind::Move => {
            hint(ui, "Arrow: move along an axis. Square: on a plane.\nCircle: on the grid plane.");
            ui.checkbox(&mut editor.grid.snap, "Snap to grid");
        }
        ToolKind::Scale => {
            hint(ui, "Drag a handle to resize the selection.\nThe opposite side stays in place.");
            ui.checkbox(&mut editor.grid.snap, "Snap to grid");
        }
        ToolKind::Block => {
            hint(ui, "Drag on the grid to create a block.");
            ui.add_space(4.0);
            let height = &mut editor.tools.block.height;
            row(ui, "Height", |ui| {
                ui.add(egui::DragValue::new(height).speed(1.0).range(1.0..=units::MAP_HALF_EXTENT * 2.0).suffix(" u"))
            });
            ui.spacing_mut().slider_width = ui.available_width();
            ui.add(egui::Slider::new(height, 1.0..=1024.0).logarithmic(true).show_value(false).step_by(1.0));
            hint(ui, &units::format_length(*height));
        }
    }
}

// --- Grid ------------------------------------------------------------------------

fn grid_settings(ui: &mut Ui, editor: &mut Editor) {
    // Hammer grid sizes as a two-column radio grid (like Noggit's brush types).
    let sizes: Vec<u32> = (0..10).map(|i| 1 << i).collect();
    egui::Grid::new("grid_sizes").num_columns(2).spacing([24.0, 2.0]).show(ui, |ui| {
        for pair in sizes.chunks(2) {
            for &size in pair {
                if ui.radio(editor.grid.size() == size, format!("{size} u")).clicked() {
                    editor.grid.set_size(size);
                }
            }
            ui.end_row();
        }
    });
    ui.add_space(4.0);
    ui.checkbox(&mut editor.grid.snap, "Snap to grid");
    ui.checkbox(&mut editor.grid.visible, "Show grid");
    ui.checkbox(&mut editor.view.show_player_reference, "Player reference (32×32×72)");
}

// --- Selection -------------------------------------------------------------------

fn selection(ui: &mut Ui, editor: &mut Editor) {
    if let Some(mode) = editor.edit_context() {
        edit_selection(ui, editor, mode);
        return;
    }
    let grid_step = editor.grid.size() as f64;
    let count = editor.scene.selection.objects.len();

    let Some(object) = editor.scene.active() else {
        hint(ui, "Nothing selected");
        return;
    };
    if count > 1 {
        hint(ui, &format!("{count} objects selected, editing the active one"));
    }

    // Widgets edit local copies; the scene is only modified after a history checkpoint.
    let mut name = object.name.clone();
    let bounds = object.mesh.bounds();
    let (vert_count, face_count) = (object.mesh.vertices.len(), object.mesh.faces.len());
    // Edited as origin (min corner) + size, which is more natural than min/max.
    let mut origin = bounds.min;
    let mut size = bounds.size();
    let mut began_edit = false;

    row(ui, "Name", |ui| {
        began_edit |= ui.add(egui::TextEdit::singleline(&mut name).desired_width(140.0)).gained_focus();
    });
    ui.add_space(2.0);

    egui::Grid::new("object_transform").num_columns(4).spacing([4.0, 4.0]).show(ui, |ui| {
        ui.label("");
        for axis in ["X", "Y", "Z"] {
            ui.vertical_centered(|ui| ui.label(RichText::new(axis).color(theme::TEXT_WEAK)));
        }
        ui.end_row();

        ui.label("Origin:");
        began_edit |= vec3_edit(ui, &mut origin, grid_step, None);
        ui.end_row();

        ui.label("Size:");
        began_edit |= vec3_edit(ui, &mut size, grid_step, Some(0.0));
        ui.end_row();
    });

    hint(
        ui,
        &format!(
            "{:.2} × {:.2} × {:.2} m  ·  {vert_count} verts, {face_count} faces",
            size.x * units::METERS_PER_UNIT,
            size.y * units::METERS_PER_UNIT,
            size.z * units::METERS_PER_UNIT,
        ),
    );

    ui.add_space(4.0);
    let delete = ui.button("Delete").on_hover_text("Del / X").clicked();

    // --- Apply ---
    if began_edit {
        editor.history.checkpoint(&editor.scene);
    }
    if let Some(object) = editor.scene.active_mut() {
        object.name = name;
        let new_bounds = Aabb::from_corners(origin, origin + size);
        if new_bounds != bounds {
            object.mesh.fit_to_bounds(&new_bounds);
        }
    }
    if delete {
        editor.delete_selected();
    }
}

/// Edit Mode: selected element counts and their median point, editable like Blender's N panel.
fn edit_selection(ui: &mut Ui, editor: &mut Editor, mode: SelectMode) {
    let Some(object) = editor.scene.active() else { return };
    let selected = editor.scene.selection.elements.resolve(&object.mesh, mode);

    row(ui, "Object", |ui| ui.label(RichText::new(&object.name).strong()));
    hint(
        ui,
        &format!(
            "{} verts, {} edges, {} faces selected",
            selected.verts.len(),
            selected.edges.len(),
            selected.faces.len()
        ),
    );
    if selected.verts.is_empty() {
        hint(ui, "Click to select, Shift+click to add.\n1 / 2 / 3: vertex / edge / face.");
        return;
    }

    let points: Vec<Vec3> = selected.verts.iter().map(|&v| object.mesh.vertices[v as usize]).collect();
    let median = points.iter().copied().sum::<Vec3>() / points.len() as f32;
    let mut new_median = median;
    let mut began_edit = false;

    ui.add_space(2.0);
    egui::Grid::new("edit_median").num_columns(4).spacing([4.0, 4.0]).show(ui, |ui| {
        ui.label("Median:");
        began_edit = vec3_edit(ui, &mut new_median, editor.grid.size() as f64, None);
        ui.end_row();
    });

    if began_edit {
        editor.history.checkpoint(&editor.scene);
    }
    let delta = new_median - median;
    if delta != Vec3::ZERO
        && let Some(object) = editor.scene.active_mut()
    {
        for &v in &selected.verts {
            object.mesh.vertices[v as usize] += delta;
        }
    }
}

/// Three drag values. Returns `true` when the user starts editing one of them.
fn vec3_edit(ui: &mut Ui, v: &mut Vec3, speed: f64, min: Option<f32>) -> bool {
    let mut began = false;
    for c in [&mut v.x, &mut v.y, &mut v.z] {
        let mut drag = egui::DragValue::new(c).speed(speed * 0.1);
        if let Some(min) = min {
            drag = drag.range(min..=f32::MAX);
        }
        let response = ui.add_sized([56.0, 18.0], drag);
        began |= response.drag_started() || response.gained_focus();
    }
    began
}

// --- Scene -------------------------------------------------------------------------

fn scene_outline(ui: &mut Ui, editor: &mut Editor) {
    ui.add_space(6.0);
    let title = format!("Scene ({})", editor.scene.objects().len());
    egui::CollapsingHeader::new(RichText::new(title).strong()).id_salt("scene_outline").default_open(true).show(ui, |ui| {
        if editor.scene.objects().is_empty() {
            hint(ui, "Empty scene. Use the Block tool (Shift+B).");
            return;
        }
        let mut clicked = None;
        for object in editor.scene.objects() {
            let selected = editor.scene.selection.contains(object.id);
            if ui.selectable_label(selected, &object.name).clicked() {
                clicked = Some(object.id);
            }
        }
        if let Some(id) = clicked {
            if ui.input(|i| i.modifiers.shift) {
                editor.scene.selection.toggle(id);
            } else {
                editor.scene.selection.set(id);
            }
        }
    });
}
