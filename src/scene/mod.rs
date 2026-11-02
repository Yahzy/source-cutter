//! The document being edited: a list of brushes and the current selection.

mod brush;

pub use brush::{Brush, BrushId};

use crate::math::{Aabb, Ray};

#[derive(Default)]
pub struct Scene {
    brushes: Vec<Brush>,
    selected: Option<BrushId>,
    next_id: u32,
}

impl Scene {
    pub fn brushes(&self) -> &[Brush] {
        &self.brushes
    }

    pub fn add_brush(&mut self, bounds: Aabb) -> BrushId {
        self.next_id += 1;
        let id = BrushId(self.next_id);
        self.brushes.push(Brush::new(id, bounds));
        id
    }

    pub fn remove(&mut self, id: BrushId) {
        self.brushes.retain(|b| b.id != id);
        if self.selected == Some(id) {
            self.selected = None;
        }
    }

    // --- Selection -------------------------------------------------------

    pub fn selected_id(&self) -> Option<BrushId> {
        self.selected
    }

    pub fn select(&mut self, id: Option<BrushId>) {
        self.selected = id;
    }

    pub fn selected(&self) -> Option<&Brush> {
        let id = self.selected?;
        self.brushes.iter().find(|b| b.id == id)
    }

    pub fn selected_mut(&mut self) -> Option<&mut Brush> {
        let id = self.selected?;
        self.brushes.iter_mut().find(|b| b.id == id)
    }

    pub fn delete_selected(&mut self) {
        if let Some(id) = self.selected {
            self.remove(id);
        }
    }

    // --- Queries ---------------------------------------------------------

    /// Returns the closest brush hit by the ray.
    pub fn pick(&self, ray: &Ray) -> Option<BrushId> {
        self.brushes
            .iter()
            .filter_map(|b| ray.intersect_aabb(&b.bounds).map(|t| (t, b.id)))
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, id)| id)
    }
}
