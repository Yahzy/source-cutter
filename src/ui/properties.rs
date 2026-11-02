//! Right panel: active tool settings, selected brush, scene outline.

use eframe::egui::{self, RichText, Ui};
use glam::Vec3;

use crate::editor::Editor;
use crate::math::Aabb;
use crate::tools::ToolKind;
use crate::units;

pub fn show(ui: &mut Ui, editor: &mut Editor) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        section(ui, "Tool", |ui| tool_settings(ui, editor));
        section(ui, "Selection", |ui| selection(ui, editor));
        section(ui, "Scene", |ui| outline(ui, editor));
    });
}

fn section(ui: &mut Ui, title: &str, add_contents: impl FnOnce(&mut Ui)) {
    ui.add_space(6.0);
    ui.vertical_centered(|ui| ui.label(RichText::new(title).strong().size(14.0)));
    ui.add_space(4.0);
    add_contents(ui);
    ui.add_space(6.0);
    ui.separator();
}

fn tool_settings(ui: &mut Ui, editor: &mut Editor) {
    match editor.tools.active {
        ToolKind::Select => {
            ui.label("Click an object to select it.
Shift+click to add to the selection.");
        }
        ToolKind::Block => {
            ui.label("Drag on the grid to create a block.");
            ui.horizontal(|ui| {
                ui.label("Height");
                ui.add(
                    egui::DragValue::new(&mut editor.tools.block.height)
                        .speed(1.0)
                        .range(1.0..=units::MAP_HALF_EXTENT * 2.0)
                        .suffix(" u"),
                );
            });
            ui.small(units::format_length(editor.tools.block.height));
        }
    }
}

fn selection(ui: &mut Ui, editor: &mut Editor) {
    let grid_step = editor.grid.size() as f64;
    let count = editor.scene.selection.objects.len();

    let Some(object) = editor.scene.active_mut() else {
        ui.weak("Nothing selected");
        return;
    };
    if count > 1 {
        ui.weak(format!("{count} objects selected, editing the active one"));
    }

    ui.horizontal(|ui| {
        ui.label("Name");
        ui.text_edit_singleline(&mut object.name);
    });

    // Edited as origin (min corner) + size, which is more natural than min/max.
    let bounds = object.mesh.bounds();
    let mut origin = bounds.min;
    let mut size = bounds.size();

    egui::Grid::new("object_transform").num_columns(4).spacing([6.0, 4.0]).show(ui, |ui| {
        ui.label("");
        ui.label("X");
        ui.label("Y");
        ui.label("Z");
        ui.end_row();

        ui.label("Origin");
        vec3_edit(ui, &mut origin, grid_step, None);
        ui.end_row();

        ui.label("Size");
        vec3_edit(ui, &mut size, grid_step, Some(0.0));
        ui.end_row();
    });

    let new_bounds = Aabb::from_corners(origin, origin + size);
    if new_bounds != bounds {
        object.mesh.fit_to_bounds(&new_bounds);
    }

    let mesh = &object.mesh;
    ui.small(format!(
        "{:.2} × {:.2} × {:.2} m  ·  {} verts, {} faces",
        size.x * units::METERS_PER_UNIT,
        size.y * units::METERS_PER_UNIT,
        size.z * units::METERS_PER_UNIT,
        mesh.vertices.len(),
        mesh.faces.len(),
    ));

    ui.add_space(4.0);
    if ui.button("🗑 Delete").clicked() {
        editor.delete_selected();
    }
}

fn vec3_edit(ui: &mut Ui, v: &mut Vec3, speed: f64, min: Option<f32>) {
    for c in [&mut v.x, &mut v.y, &mut v.z] {
        let mut drag = egui::DragValue::new(c).speed(speed * 0.1);
        if let Some(min) = min {
            drag = drag.range(min..=f32::MAX);
        }
        ui.add_sized([58.0, 18.0], drag);
    }
}

fn outline(ui: &mut Ui, editor: &mut Editor) {
    if editor.scene.objects().is_empty() {
        ui.weak("Empty scene");
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
}
