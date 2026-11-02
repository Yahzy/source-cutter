use glam::Vec3;

use super::ToolInput;
use crate::editor::History;
use crate::math::Aabb;
use crate::scene::{Mesh, Scene};

/// Hammer's Block tool: drag on the work plane to draw a box,
/// extruded along the plane normal by `height`.
pub struct BlockTool {
    /// Height in units of the created brushes.
    pub height: f32,
    drag: Option<BlockDrag>,
}

#[derive(Clone, Copy)]
struct BlockDrag {
    start: Vec3,
    end: Vec3,
    normal: Vec3,
}

impl Default for BlockTool {
    fn default() -> Self {
        Self { height: 64.0, drag: None }
    }
}

impl BlockTool {
    pub fn update(&mut self, input: &ToolInput, scene: &mut Scene, history: &mut History) {
        if input.pressed
            && let Some(p) = input.on_plane
        {
            self.drag = Some(BlockDrag { start: p, end: p, normal: input.plane.normal() });
        }

        let Some(drag) = &mut self.drag else { return };

        if input.down
            && let Some(p) = input.on_plane
        {
            drag.end = p;
        }

        if input.released {
            if let Some(bounds) = self.preview() {
                history.checkpoint(scene);
                let id = scene.add_object("Block", Mesh::cuboid(&bounds));
                scene.selection.set(id);
            }
            self.drag = None;
        }
    }

    /// The box being drawn, if the drag currently describes a valid brush.
    pub fn preview(&self) -> Option<Aabb> {
        let drag = self.drag?;
        let height = self.height.max(1.0);
        let bounds = Aabb::from_corners(drag.start, drag.end + drag.normal * height);
        (!bounds.is_degenerate()).then_some(bounds)
    }

    pub fn cancel(&mut self) -> bool {
        self.drag.take().is_some()
    }
}
