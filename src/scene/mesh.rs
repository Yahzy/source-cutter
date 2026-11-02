//! Polygon mesh: shared vertices + polygonal faces (like Blender, not triangles only).

use std::collections::BTreeSet;

use glam::Vec3;

use crate::math::{Aabb, Ray};

/// A polygon, as indices into `Mesh::vertices`, counter-clockwise seen from outside.
#[derive(Clone, Debug, PartialEq)]
pub struct Face {
    pub verts: Vec<u32>,
}

/// An edge identified by its two vertex indices, smallest first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdgeKey(pub u32, pub u32);

impl EdgeKey {
    pub fn new(a: u32, b: u32) -> Self {
        if a < b { Self(a, b) } else { Self(b, a) }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mesh {
    pub vertices: Vec<Vec3>,
    pub faces: Vec<Face>,
}

impl Mesh {
    /// A box with 8 vertices and 6 quads.
    pub fn cuboid(bounds: &Aabb) -> Self {
        // Corner index bits: 1 = max x, 2 = max y, 4 = max z.
        const QUADS: [[u32; 4]; 6] = [
            [0, 4, 6, 2], // -X
            [1, 3, 7, 5], // +X
            [0, 1, 5, 4], // -Y
            [2, 6, 7, 3], // +Y
            [0, 2, 3, 1], // -Z
            [4, 5, 7, 6], // +Z
        ];
        Self {
            vertices: bounds.corners().to_vec(),
            faces: QUADS.iter().map(|q| Face { verts: q.to_vec() }).collect(),
        }
    }

    // --- Queries ---------------------------------------------------------

    pub fn bounds(&self) -> Aabb {
        Aabb::from_points(&self.vertices)
    }

    pub fn face_points(&self, face: usize) -> Vec<Vec3> {
        self.faces[face].verts.iter().map(|&i| self.vertices[i as usize]).collect()
    }

    pub fn face_center(&self, face: usize) -> Vec3 {
        let verts = &self.faces[face].verts;
        verts.iter().map(|&i| self.vertices[i as usize]).sum::<Vec3>() / verts.len().max(1) as f32
    }

    /// Face normal computed with Newell's method (robust for any planar polygon).
    pub fn face_normal(&self, face: usize) -> Vec3 {
        let pts = self.face_points(face);
        let mut n = Vec3::ZERO;
        for i in 0..pts.len() {
            let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
            n += Vec3::new((a.y - b.y) * (a.z + b.z), (a.z - b.z) * (a.x + b.x), (a.x - b.x) * (a.y + b.y));
        }
        n.normalize_or_zero()
    }

    /// Edges of one face, in order.
    pub fn face_edges(&self, face: usize) -> impl Iterator<Item = EdgeKey> + '_ {
        let verts = &self.faces[face].verts;
        (0..verts.len()).map(move |i| EdgeKey::new(verts[i], verts[(i + 1) % verts.len()]))
    }

    /// All unique edges of the mesh.
    pub fn edges(&self) -> BTreeSet<EdgeKey> {
        (0..self.faces.len()).flat_map(|f| self.face_edges(f)).collect()
    }

    /// Closest face hit by the ray: `(distance, face index)`.
    pub fn raycast(&self, ray: &Ray) -> Option<(f32, usize)> {
        (0..self.faces.len())
            .filter_map(|f| {
                let pts = self.face_points(f);
                // Fan triangulation (faces are convex for now).
                (1..pts.len().saturating_sub(1))
                    .filter_map(|i| ray.intersect_triangle(pts[0], pts[i], pts[i + 1]))
                    .min_by(f32::total_cmp)
                    .map(|t| (t, f))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
    }

    // --- Editing ---------------------------------------------------------

    pub fn translate(&mut self, offset: Vec3) {
        for v in &mut self.vertices {
            *v += offset;
        }
    }

    /// Stretches the mesh so its bounds become `target` (axes with no size are just moved).
    pub fn fit_to_bounds(&mut self, target: &Aabb) {
        let current = self.bounds();
        let size = current.size();
        let new_size = target.size();
        for v in &mut self.vertices {
            for axis in 0..3 {
                let t = if size[axis] > 0.0 { (v[axis] - current.min[axis]) / size[axis] } else { 0.0 };
                v[axis] = target.min[axis] + t * new_size[axis];
            }
        }
    }

    /// Removes the given faces, then any vertex no longer used.
    pub fn delete_faces(&mut self, faces: &BTreeSet<u32>) {
        let mut index = 0;
        self.faces.retain(|_| {
            let keep = !faces.contains(&index);
            index += 1;
            keep
        });
        self.remove_unused_vertices();
    }

    /// Removes the faces using any of the given edges.
    pub fn delete_edges(&mut self, edges: &BTreeSet<EdgeKey>) {
        let faces = (0..self.faces.len())
            .filter(|&f| self.face_edges(f).any(|e| edges.contains(&e)))
            .map(|f| f as u32)
            .collect();
        self.delete_faces(&faces);
    }

    /// Removes the given vertices and every face using them.
    pub fn delete_vertices(&mut self, verts: &BTreeSet<u32>) {
        let faces = (0..self.faces.len())
            .filter(|&f| self.faces[f].verts.iter().any(|v| verts.contains(v)))
            .map(|f| f as u32)
            .collect();
        self.delete_faces(&faces);
    }

    fn remove_unused_vertices(&mut self) {
        let mut used = vec![false; self.vertices.len()];
        for face in &self.faces {
            for &v in &face.verts {
                used[v as usize] = true;
            }
        }
        let mut remap = vec![u32::MAX; self.vertices.len()];
        let mut kept = Vec::new();
        for (i, v) in self.vertices.iter().enumerate() {
            if used[i] {
                remap[i] = kept.len() as u32;
                kept.push(*v);
            }
        }
        self.vertices = kept;
        for face in &mut self.faces {
            for v in &mut face.verts {
                *v = remap[*v as usize];
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_cube() -> Mesh {
        Mesh::cuboid(&Aabb::from_corners(Vec3::ZERO, Vec3::splat(64.0)))
    }

    #[test]
    fn cuboid_normals_point_outwards() {
        let mesh = unit_cube();
        let center = mesh.bounds().center();
        for f in 0..mesh.faces.len() {
            let outward = mesh.face_center(f) - center;
            assert!(mesh.face_normal(f).dot(outward) > 0.0, "face {f} is inverted");
        }
    }

    #[test]
    fn cuboid_has_12_edges() {
        assert_eq!(unit_cube().edges().len(), 12);
    }

    #[test]
    fn deleting_a_vertex_removes_its_faces() {
        let mut mesh = unit_cube();
        mesh.delete_vertices(&BTreeSet::from([0]));
        assert_eq!(mesh.faces.len(), 3);
        assert_eq!(mesh.vertices.len(), 7);
    }

    #[test]
    fn raycast_hits_top_face() {
        let mesh = unit_cube();
        let ray = Ray { origin: Vec3::new(32.0, 32.0, 200.0), dir: Vec3::NEG_Z };
        let (t, face) = mesh.raycast(&ray).unwrap();
        assert_eq!(t, 136.0);
        assert_eq!(mesh.face_normal(face), Vec3::Z);
    }
}
