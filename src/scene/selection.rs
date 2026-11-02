use std::collections::BTreeSet;

use super::{EdgeKey, Mesh, ObjectId};

/// Which objects are selected, which one is active (shown in the properties and
/// edited in Edit Mode), and the selected vertices/edges/faces of the active object.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Selection {
    pub objects: BTreeSet<ObjectId>,
    pub active: Option<ObjectId>,
    pub elements: ElementSelection,
}

impl Selection {
    pub fn clear(&mut self) {
        self.objects.clear();
        self.active = None;
        self.elements.clear();
    }

    /// Selects only this object.
    pub fn set(&mut self, id: ObjectId) {
        self.objects = BTreeSet::from([id]);
        self.set_active(Some(id));
    }

    /// Shift+click behaviour: add to the selection, or remove if already active.
    pub fn toggle(&mut self, id: ObjectId) {
        if self.active == Some(id) {
            self.objects.remove(&id);
            self.set_active(self.objects.iter().next_back().copied());
        } else {
            self.objects.insert(id);
            self.set_active(Some(id));
        }
    }

    pub fn contains(&self, id: ObjectId) -> bool {
        self.objects.contains(&id)
    }

    pub fn remove(&mut self, id: ObjectId) {
        self.objects.remove(&id);
        if self.active == Some(id) {
            self.set_active(None);
        }
    }

    /// Element selection belongs to the active object: reset it when that changes.
    fn set_active(&mut self, id: Option<ObjectId>) {
        if self.active != id {
            self.elements.clear();
        }
        self.active = id;
    }
}

// --- Edit Mode ---------------------------------------------------------------

/// Blender's select modes (keys 1, 2, 3 in Edit Mode).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SelectMode {
    #[default]
    Vertex,
    Edge,
    Face,
}

impl SelectMode {
    pub const ALL: [SelectMode; 3] = [SelectMode::Vertex, SelectMode::Edge, SelectMode::Face];

    pub fn label(self) -> &'static str {
        match self {
            SelectMode::Vertex => "Vertex",
            SelectMode::Edge => "Edge",
            SelectMode::Face => "Face",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            SelectMode::Vertex => "1",
            SelectMode::Edge => "2",
            SelectMode::Face => "3",
        }
    }
}

/// One mesh element.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Element {
    Vertex(u32),
    Edge(EdgeKey),
    Face(u32),
}

/// Selected elements of the active mesh. Only the set matching the current
/// select mode is authoritative; [`ElementSelection::resolve`] derives the others.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ElementSelection {
    pub verts: BTreeSet<u32>,
    pub edges: BTreeSet<EdgeKey>,
    pub faces: BTreeSet<u32>,
}

impl ElementSelection {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Selects only this element.
    pub fn set(&mut self, element: Element) {
        self.clear();
        self.toggle(element);
    }

    pub fn toggle(&mut self, element: Element) {
        fn flip<T: Ord>(set: &mut BTreeSet<T>, item: T) {
            if !set.remove(&item) {
                set.insert(item);
            }
        }
        match element {
            Element::Vertex(v) => flip(&mut self.verts, v),
            Element::Edge(e) => flip(&mut self.edges, e),
            Element::Face(f) => flip(&mut self.faces, f),
        }
    }

    pub fn select_all(&mut self, mesh: &Mesh, mode: SelectMode) {
        self.clear();
        match mode {
            SelectMode::Vertex => self.verts = (0..mesh.vertices.len() as u32).collect(),
            SelectMode::Edge => self.edges = mesh.edges(),
            SelectMode::Face => self.faces = (0..mesh.faces.len() as u32).collect(),
        }
    }

    pub fn is_empty(&self, mode: SelectMode) -> bool {
        match mode {
            SelectMode::Vertex => self.verts.is_empty(),
            SelectMode::Edge => self.edges.is_empty(),
            SelectMode::Face => self.faces.is_empty(),
        }
    }

    /// All selected vertices, edges and faces implied by the selection in `mode`
    /// (e.g. in vertex mode, a face is selected when all its vertices are).
    pub fn resolve(&self, mesh: &Mesh, mode: SelectMode) -> ElementSelection {
        let vert_count = mesh.vertices.len() as u32;
        let face_count = mesh.faces.len();
        match mode {
            SelectMode::Vertex => {
                let verts: BTreeSet<u32> = self.verts.iter().copied().filter(|&v| v < vert_count).collect();
                let edges = mesh.edges().into_iter().filter(|e| verts.contains(&e.0) && verts.contains(&e.1)).collect();
                let faces = (0..face_count)
                    .filter(|&f| mesh.faces[f].verts.iter().all(|v| verts.contains(v)))
                    .map(|f| f as u32)
                    .collect();
                ElementSelection { verts, edges, faces }
            }
            SelectMode::Edge => {
                let edges: BTreeSet<EdgeKey> = mesh.edges().intersection(&self.edges).copied().collect();
                let verts = edges.iter().flat_map(|e| [e.0, e.1]).collect();
                let faces = (0..face_count)
                    .filter(|&f| mesh.face_edges(f).all(|e| edges.contains(&e)))
                    .map(|f| f as u32)
                    .collect();
                ElementSelection { verts, edges, faces }
            }
            SelectMode::Face => {
                let faces: BTreeSet<u32> = self.faces.iter().copied().filter(|&f| (f as usize) < face_count).collect();
                let verts = faces.iter().flat_map(|&f| mesh.faces[f as usize].verts.iter().copied()).collect();
                let edges = faces.iter().flat_map(|&f| mesh.face_edges(f as usize)).collect();
                ElementSelection { verts, edges, faces }
            }
        }
    }

    /// Switching select mode keeps what was selected, like Blender.
    pub fn convert(&mut self, mesh: &Mesh, from: SelectMode, to: SelectMode) {
        let resolved = self.resolve(mesh, from);
        self.clear();
        match to {
            SelectMode::Vertex => self.verts = resolved.verts,
            SelectMode::Edge => self.edges = resolved.edges,
            SelectMode::Face => self.faces = resolved.faces,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Aabb;
    use glam::Vec3;

    fn cube() -> Mesh {
        Mesh::cuboid(&Aabb::from_corners(Vec3::ZERO, Vec3::splat(64.0)))
    }

    #[test]
    fn top_face_selects_four_verts_and_edges() {
        let mesh = cube();
        let top = (0..6).find(|&f| mesh.face_normal(f) == Vec3::Z).unwrap() as u32;
        let mut sel = ElementSelection::default();
        sel.set(Element::Face(top));
        let r = sel.resolve(&mesh, SelectMode::Face);
        assert_eq!((r.verts.len(), r.edges.len(), r.faces.len()), (4, 4, 1));
    }

    #[test]
    fn converting_face_to_vertex_and_back_keeps_the_face() {
        let mesh = cube();
        let mut sel = ElementSelection::default();
        sel.set(Element::Face(2));
        sel.convert(&mesh, SelectMode::Face, SelectMode::Vertex);
        assert_eq!(sel.verts.len(), 4);
        sel.convert(&mesh, SelectMode::Vertex, SelectMode::Face);
        assert_eq!(sel.faces, BTreeSet::from([2]));
    }
}
