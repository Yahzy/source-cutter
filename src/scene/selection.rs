use std::collections::BTreeSet;

use super::ObjectId;

/// Which objects are selected, and which one is active (the one shown in the properties).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Selection {
    pub objects: BTreeSet<ObjectId>,
    pub active: Option<ObjectId>,
}

impl Selection {
    pub fn clear(&mut self) {
        self.objects.clear();
        self.active = None;
    }

    /// Selects only this object.
    pub fn set(&mut self, id: ObjectId) {
        self.objects = BTreeSet::from([id]);
        self.active = Some(id);
    }

    /// Shift+click behaviour: add to the selection, or remove if already active.
    pub fn toggle(&mut self, id: ObjectId) {
        if self.active == Some(id) {
            self.objects.remove(&id);
            self.active = self.objects.iter().next_back().copied();
        } else {
            self.objects.insert(id);
            self.active = Some(id);
        }
    }

    pub fn contains(&self, id: ObjectId) -> bool {
        self.objects.contains(&id)
    }

    pub fn remove(&mut self, id: ObjectId) {
        self.objects.remove(&id);
        if self.active == Some(id) {
            self.active = None;
        }
    }
}
