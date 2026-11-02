//! Editor state: everything the user edits or configures, independent of the UI layout.

mod camera;
mod grid;
mod history;

pub use camera::OrbitCamera;
pub use grid::{GridPlane, GridSettings};
pub use history::History;

use glam::Vec3;

use crate::math::Aabb;
use crate::scene::Scene;
use crate::tools::Tools;

/// Viewport display toggles.
pub struct ViewOptions {
    /// Shows a 32x32x72 player hull at the origin, as a scale reference.
    pub show_player_reference: bool,
}

impl Default for ViewOptions {
    fn default() -> Self {
        Self { show_player_reference: true }
    }
}

#[derive(Default)]
pub struct Editor {
    pub scene: Scene,
    pub history: History,
    pub camera: OrbitCamera,
    pub grid: GridSettings,
    pub tools: Tools,
    pub view: ViewOptions,
    /// Snapped point under the mouse on the ground plane (for the status bar).
    pub cursor: Option<Vec3>,
}

impl Editor {
    pub fn new_scene(&mut self) {
        self.tools.cancel();
        self.history.checkpoint(&self.scene);
        self.scene = Scene::default();
    }

    pub fn delete_selected(&mut self) {
        if self.scene.selection.objects.is_empty() {
            return;
        }
        self.history.checkpoint(&self.scene);
        self.scene.delete_selected();
    }

    pub fn undo(&mut self) {
        self.tools.cancel();
        self.history.undo(&mut self.scene);
    }

    pub fn redo(&mut self) {
        self.tools.cancel();
        self.history.redo(&mut self.scene);
    }

    pub fn select_all(&mut self) {
        self.scene.select_all();
    }

    pub fn deselect_all(&mut self) {
        self.scene.selection.clear();
    }

    /// Centers the camera on the selection, or resets the view if nothing is selected.
    pub fn frame_selection(&mut self) {
        match self.scene.selection_bounds() {
            Some(bounds) => self.camera.frame(&bounds),
            None => self.camera = OrbitCamera::default(),
        }
    }

    /// Escape: cancel the current tool action first, then clear the selection.
    pub fn escape(&mut self) {
        if !self.tools.cancel() {
            self.scene.selection.clear();
        }
    }

    pub fn player_reference_bounds() -> Aabb {
        use crate::units::{PLAYER_HEIGHT, PLAYER_WIDTH};
        let half = PLAYER_WIDTH * 0.5;
        Aabb::from_corners(Vec3::new(-half, -half, 0.0), Vec3::new(half, half, PLAYER_HEIGHT))
    }
}
