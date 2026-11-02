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
            ui.label("Click a brush to select it.");
        }
        ToolKind::Block => {
            ui.label("Drag on the grid to create a brush.");
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
    let mut delete = false;

    let Some(brush) = editor.scene.selected_mut() else {
        ui.weak("Nothing selected");
        return;
    };

    ui.horizontal(|ui| {
        ui.label("Name");
        ui.text_edit_singleline(&mut brush.name);
    });

    // Edited as origin (min corner) + size, which is more natural than min/max.
    let mut origin = brush.bounds.min;
    let mut size = brush.bounds.size();

    egui::Grid::new("brush_transform").num_columns(4).spacing([6.0, 4.0]).show(ui, |ui| {
        ui.label("");
        ui.label("X");
        ui.label("Y");
        ui.label("Z");
        ui.end_row();

        ui.label("Origin");
        vec3_edit(ui, &mut origin, grid_step, None);
        ui.end_row();

        ui.label("Size");
        vec3_edit(ui, &mut size, grid_step, Some(1.0));
        ui.end_row();
    });

    brush.bounds = Aabb::from_corners(origin, origin + size);

    let s = brush.bounds.size();
    ui.small(format!(
        "{:.2} × {:.2} × {:.2} m",
        s.x * units::METERS_PER_UNIT,
        s.y * units::METERS_PER_UNIT,
        s.z * units::METERS_PER_UNIT
    ));

    ui.add_space(4.0);
    if ui.button("🗑 Delete brush").clicked() {
        delete = true;
    }

    if delete {
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
    if editor.scene.brushes().is_empty() {
        ui.weak("Empty scene");
        return;
    }
    let selected = editor.scene.selected_id();
    let mut clicked = None;
    for brush in editor.scene.brushes() {
        if ui.selectable_label(selected == Some(brush.id), &brush.name).clicked() {
            clicked = Some(brush.id);
        }
    }
    if let Some(id) = clicked {
        editor.scene.select(Some(id));
    }
}
