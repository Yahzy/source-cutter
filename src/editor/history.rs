//! Undo / redo using scene snapshots.
//!
//! Simple and robust: before each modification, the whole scene is saved.
//! Fine for now; can later become a command-based history if scenes get big.

use crate::scene::Scene;

const MAX_STEPS: usize = 200;

#[derive(Default)]
pub struct History {
    undo: Vec<Scene>,
    redo: Vec<Scene>,
}

impl History {
    /// Saves the scene state *before* a modification.
    /// Does nothing if the scene did not change since the last checkpoint.
    pub fn checkpoint(&mut self, scene: &Scene) {
        if self.undo.last() == Some(scene) {
            return;
        }
        self.undo.push(scene.clone());
        if self.undo.len() > MAX_STEPS {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    /// Forgets the last checkpoint (used when an action is cancelled).
    pub fn discard_last(&mut self) {
        self.undo.pop();
    }

    pub fn undo(&mut self, scene: &mut Scene) -> bool {
        // Skip checkpoints identical to the current state (nothing was changed after them).
        while self.undo.last() == Some(scene) {
            self.undo.pop();
        }
        let Some(previous) = self.undo.pop() else { return false };
        self.redo.push(std::mem::replace(scene, previous));
        true
    }

    pub fn redo(&mut self, scene: &mut Scene) -> bool {
        let Some(next) = self.redo.pop() else { return false };
        self.undo.push(std::mem::replace(scene, next));
        true
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Aabb;
    use crate::scene::Mesh;
    use glam::Vec3;

    fn add_block(scene: &mut Scene) {
        scene.add_object("Block", Mesh::cuboid(&Aabb::from_corners(Vec3::ZERO, Vec3::ONE)));
    }

    #[test]
    fn undo_then_redo() {
        let mut history = History::default();
        let mut scene = Scene::default();

        history.checkpoint(&scene);
        add_block(&mut scene);
        history.checkpoint(&scene);
        add_block(&mut scene);

        assert!(history.undo(&mut scene));
        assert_eq!(scene.objects().len(), 1);
        assert!(history.undo(&mut scene));
        assert_eq!(scene.objects().len(), 0);
        assert!(!history.undo(&mut scene));

        assert!(history.redo(&mut scene));
        assert!(history.redo(&mut scene));
        assert_eq!(scene.objects().len(), 2);
    }

    #[test]
    fn unchanged_checkpoints_are_skipped() {
        let mut history = History::default();
        let mut scene = Scene::default();

        history.checkpoint(&scene);
        add_block(&mut scene);
        history.checkpoint(&scene); // e.g. a field was focused but not edited

        assert!(history.undo(&mut scene));
        assert_eq!(scene.objects().len(), 0);
    }
}
