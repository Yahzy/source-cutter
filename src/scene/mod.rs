//! The document being edited: objects (meshes) and the current selection.

mod mesh;
mod object;
mod selection;

pub use mesh::{EdgeKey, Mesh};
pub use object::{Object, ObjectId};
pub use selection::{Element, ElementSelection, SelectMode, Selection};

use crate::math::Ray;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scene {
    objects: Vec<Object>,
    pub selection: Selection,
    next_id: u32,
}

impl Scene {
    pub fn objects(&self) -> &[Object] {
        &self.objects
    }

    pub fn get(&self, id: ObjectId) -> Option<&Object> {
        self.objects.iter().find(|o| o.id == id)
    }

    pub fn get_mut(&mut self, id: ObjectId) -> Option<&mut Object> {
        self.objects.iter_mut().find(|o| o.id == id)
    }

    pub fn add_object(&mut self, name: &str, mesh: Mesh) -> ObjectId {
        self.next_id += 1;
        let id = ObjectId(self.next_id);
        self.objects.push(Object::new(id, format!("{name} {}", id.0), mesh));
        id
    }

    pub fn remove(&mut self, id: ObjectId) {
        self.objects.retain(|o| o.id != id);
        self.selection.remove(id);
    }

    // --- Selection helpers -------------------------------------------------

    pub fn active(&self) -> Option<&Object> {
        self.get(self.selection.active?)
    }

    pub fn active_mut(&mut self) -> Option<&mut Object> {
        self.get_mut(self.selection.active?)
    }

    pub fn delete_selected(&mut self) {
        let ids: Vec<_> = self.selection.objects.iter().copied().collect();
        for id in ids {
            self.remove(id);
        }
    }

    pub fn select_all(&mut self) {
        self.selection.objects = self.objects.iter().map(|o| o.id).collect();
        if self.selection.active.is_none() {
            self.selection.active = self.objects.last().map(|o| o.id);
        }
    }

    // --- Queries ---------------------------------------------------------

    /// Returns the closest object hit by the ray.
    pub fn pick(&self, ray: &Ray) -> Option<ObjectId> {
        self.objects
            .iter()
            .filter_map(|o| o.mesh.raycast(ray).map(|(t, _)| (t, o.id)))
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, id)| id)
    }
}
