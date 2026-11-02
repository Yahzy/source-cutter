use glam::Vec3;

use super::ToolInput;
use crate::math::Aabb;
use crate::scene::Scene;

/// Hammer's Block tool: drag on the ground to draw a box brush.
pub struct BlockTool {
    /// Height in units of the created brushes.
    pub height: f32,
    /// Start and current corner of the drag, on the ground plane.
    drag: Option<(Vec3, Vec3)>,
}

impl Default for BlockTool {
    fn default() -> Self {
        Self { height: 64.0, drag: None }
    }
}

impl BlockTool {
    pub fn update(&mut self, input: &ToolInput, scene: &mut Scene) {
        if input.pressed
            && let Some(p) = input.ground
        {
            self.drag = Some((p, p));
        }

        let Some((_, end)) = &mut self.drag else { return };

        if input.down
            && let Some(p) = input.ground
        {
            *end = p;
        }

        if input.released {
            if let Some(bounds) = self.preview() {
                let id = scene.add_brush(bounds);
                scene.select(Some(id));
            }
            self.drag = None;
        }
    }

    /// The box being drawn, if the drag currently describes a valid brush.
    pub fn preview(&self) -> Option<Aabb> {
        let (start, end) = self.drag?;
        let height = self.height.max(1.0);
        let bounds = Aabb::from_corners(start, end + Vec3::Z * height);
        (!bounds.is_degenerate()).then_some(bounds)
    }

    pub fn cancel(&mut self) -> bool {
        self.drag.take().is_some()
    }
}
