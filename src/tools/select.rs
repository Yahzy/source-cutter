use super::ToolInput;
use crate::scene::Scene;

/// Click on a brush to select it, click on empty space to deselect.
pub struct SelectTool;

impl SelectTool {
    pub fn update(&mut self, input: &ToolInput, scene: &mut Scene) {
        if input.clicked {
            let hit = input.ray.and_then(|ray| scene.pick(&ray));
            scene.select(hit);
        }
    }
}
