//! Shared code for the Move and Scale tools: which vertices are transformed,
//! their original positions, and screen-space handle helpers.

use eframe::egui::{Pos2, Vec2};
use glam::Vec3;

use crate::math::Aabb;
use crate::scene::{ObjectId, Scene, SelectMode};

/// Vertices being transformed, with their positions when the drag started.
pub struct TransformTarget {
    entries: Vec<(ObjectId, Vec<(u32, Vec3)>)>,
    pub bounds: Aabb,
}

impl TransformTarget {
    /// Captures the vertices that the current selection would transform.
    pub fn capture(scene: &Scene, edit: Option<SelectMode>) -> Option<Self> {
        let entries: Vec<_> = selected_vertices(scene, edit)
            .into_iter()
            .filter_map(|(id, verts)| {
                let mesh = &scene.get(id)?.mesh;
                Some((id, verts.into_iter().map(|v| (v, mesh.vertices[v as usize])).collect::<Vec<_>>()))
            })
            .filter(|(_, verts)| !verts.is_empty())
            .collect();

        let points: Vec<Vec3> = entries.iter().flat_map(|(_, v)| v.iter().map(|(_, p)| *p)).collect();
        if points.is_empty() {
            return None;
        }
        Some(Self { entries, bounds: Aabb::from_points(&points) })
    }

    /// Sets every vertex to `f(original position)`.
    pub fn apply(&self, scene: &mut Scene, f: impl Fn(Vec3) -> Vec3) {
        for (id, verts) in &self.entries {
            if let Some(object) = scene.get_mut(*id) {
                for &(index, original) in verts {
                    object.mesh.vertices[index as usize] = f(original);
                }
            }
        }
    }

    pub fn restore(&self, scene: &mut Scene) {
        self.apply(scene, |p| p);
    }
}

/// Vertices affected by a transform: all vertices of the selected objects in Object Mode,
/// the vertices of the selected elements of the active object in Edit Mode.
pub fn selected_vertices(scene: &Scene, edit: Option<SelectMode>) -> Vec<(ObjectId, Vec<u32>)> {
    match edit {
        None => scene
            .objects()
            .iter()
            .filter(|o| scene.selection.contains(o.id))
            .map(|o| (o.id, (0..o.mesh.vertices.len() as u32).collect()))
            .collect(),
        Some(mode) => scene
            .active()
            .map(|o| (o.id, scene.selection.elements.resolve(&o.mesh, mode).verts.into_iter().collect()))
            .into_iter()
            .collect(),
    }
}

/// Bounds of what a transform would affect right now.
pub fn selection_bounds(scene: &Scene, edit: Option<SelectMode>) -> Option<Aabb> {
    TransformTarget::capture(scene, edit).map(|t| t.bounds)
}

/// Distance in pixels from `p` to the segment `a`–`b`.
pub fn distance_to_segment(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab: Vec2 = b - a;
    let t = ((p - a).dot(ab) / ab.length_sq().max(1e-6)).clamp(0.0, 1.0);
    p.distance(a + ab * t)
}
