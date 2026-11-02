//! Occlusion tests for Edit Mode: only visible vertices/edges are drawn and pickable.

use glam::Vec3;

use super::Projector;
use crate::scene::Mesh;

/// Tolerance (units) so points lying on a face are not hidden by that face.
const EPSILON: f32 = 0.5;

/// True if `p` is not hidden behind a face of `mesh`.
pub fn point_visible(mesh: &Mesh, proj: &Projector, p: Vec3) -> bool {
    let Some(screen) = proj.point(p) else { return false };
    let ray = proj.ray(screen);
    let t_point = (p - ray.origin).dot(ray.dir);
    match mesh.raycast(&ray) {
        Some((t_hit, _)) => t_hit >= t_point - EPSILON,
        None => true,
    }
}
