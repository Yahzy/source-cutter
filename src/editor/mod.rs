//! Editor state: everything the user edits or configures, independent of the UI layout.

mod camera;
mod grid;
mod history;

pub use camera::OrbitCamera;
pub use grid::{GridPlane, GridSettings};
pub use history::History;

use glam::Vec3;

use crate::math::Aabb;
use crate::scene::{Scene, SelectMode};
use crate::tools::{self, ToolKind, Tools};

/// Blender-like interaction modes (Tab switches between them).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EditorMode {
    /// Select and transform whole objects.
    #[default]
    Object,
    /// Select and transform vertices, edges or faces of the active object.
    Edit,
}

impl EditorMode {
    pub fn label(self) -> &'static str {
        match self {
            EditorMode::Object => "Object Mode",
            EditorMode::Edit => "Edit Mode",
        }
    }
}

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
    mode: EditorMode,
    select_mode: SelectMode,
    /// Snapped point under the mouse on the work plane (for the status bar).
    pub cursor: Option<Vec3>,
}

impl Editor {
    // --- Modes -------------------------------------------------------------

    pub fn mode(&self) -> EditorMode {
        self.mode
    }

    pub fn select_mode(&self) -> SelectMode {
        self.select_mode
    }

    /// The select mode when in Edit Mode, `None` in Object Mode.
    pub fn edit_context(&self) -> Option<SelectMode> {
        (self.mode == EditorMode::Edit).then_some(self.select_mode)
    }

    pub fn set_mode(&mut self, mode: EditorMode) {
        if mode == self.mode {
            return;
        }
        self.tools.cancel(&mut self.scene, &mut self.history);
        if mode == EditorMode::Edit {
            // Edit Mode needs an active object; Block creates objects, so leave it.
            if self.scene.active().is_none() {
                return;
            }
            if self.tools.active == ToolKind::Block {
                self.tools.active = ToolKind::Select;
            }
        }
        self.mode = mode;
    }

    /// Tab.
    pub fn toggle_mode(&mut self) {
        let next = match self.mode {
            EditorMode::Object => EditorMode::Edit,
            EditorMode::Edit => EditorMode::Object,
        };
        self.set_mode(next);
    }

    /// Keys 1 / 2 / 3. Converts the current element selection, like Blender.
    pub fn set_select_mode(&mut self, select_mode: SelectMode) {
        if let Some(active) = self.scene.selection.active
            && let Some(object) = self.scene.get(active)
        {
            let mesh = object.mesh.clone();
            self.scene.selection.elements.convert(&mesh, self.select_mode, select_mode);
        }
        self.select_mode = select_mode;
    }

    /// Keeps the mode consistent after undo / deletion (Edit Mode needs an active object).
    pub fn validate(&mut self) {
        if self.mode == EditorMode::Edit && self.scene.active().is_none() {
            self.mode = EditorMode::Object;
        }
    }

    // --- Actions -----------------------------------------------------------

    pub fn set_tool(&mut self, tool: ToolKind) {
        self.tools.cancel(&mut self.scene, &mut self.history);
        if tool == ToolKind::Block {
            self.mode = EditorMode::Object;
        }
        self.tools.active = tool;
    }

    pub fn new_scene(&mut self) {
        self.tools.cancel(&mut self.scene, &mut self.history);
        self.history.checkpoint(&self.scene);
        self.scene = Scene::default();
        self.mode = EditorMode::Object;
    }

    /// Deletes the selected objects, or the selected elements in Edit Mode.
    pub fn delete_selected(&mut self) {
        match self.edit_context() {
            None => {
                if self.scene.selection.objects.is_empty() {
                    return;
                }
                self.history.checkpoint(&self.scene);
                self.scene.delete_selected();
            }
            Some(select_mode) => {
                if self.scene.selection.elements.is_empty(select_mode) {
                    return;
                }
                self.history.checkpoint(&self.scene);
                let elements = std::mem::take(&mut self.scene.selection.elements);
                if let Some(object) = self.scene.active_mut() {
                    match select_mode {
                        SelectMode::Vertex => object.mesh.delete_vertices(&elements.verts),
                        SelectMode::Edge => object.mesh.delete_edges(&elements.edges),
                        SelectMode::Face => object.mesh.delete_faces(&elements.faces),
                    }
                }
            }
        }
    }

    pub fn undo(&mut self) {
        self.tools.cancel(&mut self.scene, &mut self.history);
        self.history.undo(&mut self.scene);
        self.validate();
    }

    pub fn redo(&mut self) {
        self.tools.cancel(&mut self.scene, &mut self.history);
        self.history.redo(&mut self.scene);
        self.validate();
    }

    /// A: all objects, or all elements of the active mesh in Edit Mode.
    pub fn select_all(&mut self) {
        match self.edit_context() {
            None => self.scene.select_all(),
            Some(select_mode) => {
                if let Some(mesh) = self.scene.active().map(|o| o.mesh.clone()) {
                    self.scene.selection.elements.select_all(&mesh, select_mode);
                }
            }
        }
    }

    /// Alt+A.
    pub fn deselect_all(&mut self) {
        match self.mode {
            EditorMode::Object => self.scene.selection.clear(),
            EditorMode::Edit => self.scene.selection.elements.clear(),
        }
    }

    /// Centers the camera on the selection, or resets the view if nothing is selected.
    pub fn frame_selection(&mut self) {
        match tools::selection_bounds(&self.scene, self.edit_context()) {
            Some(bounds) => self.camera.frame(&bounds),
            None => self.camera = OrbitCamera::default(),
        }
    }

    /// Escape: cancel the current tool action first, then clear the selection.
    pub fn escape(&mut self) {
        if !self.tools.cancel(&mut self.scene, &mut self.history) {
            self.deselect_all();
        }
    }

    pub fn player_reference_bounds() -> Aabb {
        use crate::units::{PLAYER_HEIGHT, PLAYER_WIDTH};
        let half = PLAYER_WIDTH * 0.5;
        Aabb::from_corners(Vec3::new(-half, -half, 0.0), Vec3::new(half, half, PLAYER_HEIGHT))
    }
}
