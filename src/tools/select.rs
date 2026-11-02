use super::ToolInput;
use crate::scene::Scene;

/// Click an object to select it, Shift+click to add/remove, click empty space to deselect.
pub struct SelectTool;

impl SelectTool {
    pub fn update(&mut self, input: &ToolInput, scene: &mut Scene) {
        if input.clicked {
            click_select(input, scene);
        }
    }
}

/// Object picking shared by every tool that also selects on click.
pub fn click_select(input: &ToolInput, scene: &mut Scene) {
    let hit = input.ray.and_then(|ray| scene.pick(&ray));
    match (hit, input.shift) {
        (Some(id), true) => scene.selection.toggle(id),
        (Some(id), false) => scene.selection.set(id),
        (None, true) => {}
        (None, false) => scene.selection.clear(),
    }
}
