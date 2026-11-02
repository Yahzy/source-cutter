//! Editor state: everything the user edits or configures, independent of the UI layout.

mod camera;
mod grid;

pub use camera::OrbitCamera;
pub use grid::GridSettings;

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
    pub camera: OrbitCamera,
    pub grid: GridSettings,
    pub tools: Tools,
    pub view: ViewOptions,
    /// Snapped point under the mouse on the ground plane (for the status bar).
    pub cursor: Option<Vec3>,
}

impl Editor {
    pub fn new_scene(&mut self) {
        self.scene = Scene::default();
        self.tools.cancel();
    }

    pub fn delete_selected(&mut self) {
        self.scene.delete_selected();
    }

    /// Centers the camera on the selection, or resets the view if nothing is selected.
    pub fn frame_selection(&mut self) {
        match self.scene.selected().map(|b| b.bounds) {
            Some(bounds) => self.camera.frame(&bounds),
            None => self.camera = OrbitCamera::default(),
        }
    }

    /// Escape: cancel the current tool action first, then clear the selection.
    pub fn escape(&mut self) {
        if !self.tools.cancel() {
            self.scene.select(None);
        }
    }

    pub fn player_reference_bounds() -> Aabb {
        use crate::units::{PLAYER_HEIGHT, PLAYER_WIDTH};
        let half = PLAYER_WIDTH * 0.5;
        Aabb::from_corners(Vec3::new(-half, -half, 0.0), Vec3::new(half, half, PLAYER_HEIGHT))
    }
}
