//! Interactive editing tools. Tools never talk to egui directly: the viewport
//! translates mouse input into a [`ToolInput`] so tools stay easy to test and extend.

mod block;
mod select;

pub use block::BlockTool;
pub use select::SelectTool;

use glam::Vec3;

use crate::editor::{GridPlane, History};
use crate::math::Ray;
use crate::scene::Scene;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolKind {
    Select,
    Block,
}

impl ToolKind {
    pub const ALL: [ToolKind; 2] = [ToolKind::Select, ToolKind::Block];

    pub fn label(self) -> &'static str {
        match self {
            ToolKind::Select => "Select",
            ToolKind::Block => "Block",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            ToolKind::Select => "↖",
            ToolKind::Block => "⬛",
        }
    }

    pub fn shortcut(self) -> &'static str {
        match self {
            ToolKind::Select => "Shift+S",
            ToolKind::Block => "Shift+B",
        }
    }
}

/// Mouse state seen by tools, already converted to world space.
pub struct ToolInput {
    /// Ray under the mouse.
    pub ray: Option<Ray>,
    /// Plane the tools work on (follows orthographic views).
    pub plane: GridPlane,
    /// Snapped point under the mouse on the work plane.
    pub on_plane: Option<Vec3>,
    /// Left button pressed this frame inside the viewport.
    pub pressed: bool,
    /// Left button held.
    pub down: bool,
    /// Left button released this frame.
    pub released: bool,
    /// Left click (press + release without dragging) inside the viewport.
    pub clicked: bool,
    /// Shift held (add to selection).
    pub shift: bool,
}

pub struct Tools {
    pub active: ToolKind,
    pub select: SelectTool,
    pub block: BlockTool,
}

impl Default for Tools {
    fn default() -> Self {
        Self { active: ToolKind::Block, select: SelectTool, block: BlockTool::default() }
    }
}

impl Tools {
    pub fn set_active(&mut self, tool: ToolKind) {
        if self.active != tool {
            self.cancel();
            self.active = tool;
        }
    }

    pub fn update(&mut self, input: &ToolInput, scene: &mut Scene, history: &mut History) {
        match self.active {
            ToolKind::Select => self.select.update(input, scene),
            ToolKind::Block => self.block.update(input, scene, history),
        }
    }

    /// Cancels the current action. Returns `true` if something was cancelled.
    pub fn cancel(&mut self) -> bool {
        self.block.cancel()
    }
}
