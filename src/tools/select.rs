//! Click selection: objects in Object Mode, vertices / edges / faces in Edit Mode.

use super::ToolInput;
use super::transform::distance_to_segment;
use crate::scene::{Element, Mesh, Scene, SelectMode};
use crate::viewport::point_visible;

/// Pick radius in pixels for vertices and edges.
const VERTEX_PICK_PX: f32 = 12.0;
const EDGE_PICK_PX: f32 = 8.0;

/// Click to select, Shift+click to add/remove, click empty space to deselect.
pub struct SelectTool;

impl SelectTool {
    pub fn update(&mut self, input: &ToolInput, scene: &mut Scene) {
        if input.clicked {
            click_select(input, scene);
        }
    }
}

/// Picking shared by every tool that also selects on click.
pub fn click_select(input: &ToolInput, scene: &mut Scene) {
    match input.edit {
        None => select_object(input, scene),
        Some(mode) => select_element(input, scene, mode),
    }
}

fn select_object(input: &ToolInput, scene: &mut Scene) {
    let hit = input.ray.and_then(|ray| scene.pick(&ray));
    match (hit, input.shift) {
        (Some(id), true) => scene.selection.toggle(id),
        (Some(id), false) => scene.selection.set(id),
        (None, true) => {}
        (None, false) => scene.selection.clear(),
    }
}

fn select_element(input: &ToolInput, scene: &mut Scene, mode: SelectMode) {
    let hit = scene.active().and_then(|o| pick_element(input, &o.mesh, mode));
    let elements = &mut scene.selection.elements;
    match (hit, input.shift) {
        (Some(e), true) => elements.toggle(e),
        (Some(e), false) => elements.set(e),
        (None, true) => {}
        (None, false) => elements.clear(),
    }
}

/// Closest visible element of `mesh` under the mouse.
pub fn pick_element(input: &ToolInput, mesh: &Mesh, mode: SelectMode) -> Option<Element> {
    let pointer = input.pointer?;
    let proj = input.proj;
    let closest = |candidates: Vec<(f32, Element)>, max: f32| {
        candidates.into_iter().filter(|(d, _)| *d <= max).min_by(|a, b| a.0.total_cmp(&b.0)).map(|(_, e)| e)
    };

    match mode {
        SelectMode::Vertex => {
            let candidates = mesh
                .vertices
                .iter()
                .enumerate()
                .filter(|(_, p)| point_visible(mesh, proj, **p))
                .filter_map(|(i, p)| Some((proj.point(*p)?.distance(pointer), Element::Vertex(i as u32))))
                .collect();
            closest(candidates, VERTEX_PICK_PX)
        }
        SelectMode::Edge => {
            let candidates = mesh
                .edges()
                .into_iter()
                .filter_map(|e| {
                    let (a, b) = (mesh.vertices[e.0 as usize], mesh.vertices[e.1 as usize]);
                    if !point_visible(mesh, proj, (a + b) * 0.5) {
                        return None;
                    }
                    let [sa, sb] = proj.segment(a, b)?;
                    Some((distance_to_segment(pointer, sa, sb), Element::Edge(e)))
                })
                .collect();
            closest(candidates, EDGE_PICK_PX)
        }
        SelectMode::Face => {
            let ray = input.ray?;
            mesh.raycast(&ray).map(|(_, f)| Element::Face(f as u32))
        }
    }
}
