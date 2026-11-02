//! Interactive editing tools. Tools never talk to egui directly: the viewport
//! translates mouse input into a [`ToolInput`] so tools stay easy to test and extend.

mod block;
mod select;

pub use block::BlockTool;
pub use select::SelectTool;

use glam::Vec3;

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
    /// Snapped point under the mouse on the ground plane (z = 0).
    pub ground: Option<Vec3>,
    /// Left button pressed this frame inside the viewport.
    pub pressed: bool,
    /// Left button held.
    pub down: bool,
    /// Left button released this frame.
    pub released: bool,
    /// Left click (press + release without dragging) inside the viewport.
    pub clicked: bool,
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

    pub fn update(&mut self, input: &ToolInput, scene: &mut Scene) {
        match self.active {
            ToolKind::Select => self.select.update(input, scene),
            ToolKind::Block => self.block.update(input, scene),
        }
    }

    /// Cancels the current action. Returns `true` if something was cancelled.
    pub fn cancel(&mut self) -> bool {
        self.block.cancel()
    }
}
