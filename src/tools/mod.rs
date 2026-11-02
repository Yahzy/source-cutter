//! Interactive editing tools. The viewport converts mouse input into a [`ToolInput`]
//! (world-space ray, snapped point on the work plane, button states) and each tool
//! updates the scene and draws its own overlay (handles, previews).

mod block;
mod move_tool;
mod scale_tool;
mod select;
mod transform;

pub use block::BlockTool;
pub use move_tool::MoveTool;
pub use scale_tool::ScaleTool;
pub use select::SelectTool;

use eframe::egui::{Painter, Pos2};
use glam::Vec3;

use crate::editor::{GridPlane, GridSettings, History};
use crate::math::Ray;
use crate::scene::Scene;
use crate::viewport::Projector;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolKind {
    Select,
    Move,
    Scale,
    Block,
}

impl ToolKind {
    pub const ALL: [ToolKind; 4] = [ToolKind::Select, ToolKind::Move, ToolKind::Scale, ToolKind::Block];

    pub fn label(self) -> &'static str {
        match self {
            ToolKind::Select => "Select",
            ToolKind::Move => "Move",
            ToolKind::Scale => "Scale",
            ToolKind::Block => "Block",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            ToolKind::Select => "↖",
            ToolKind::Move => "✥",
            ToolKind::Scale => "⤢",
            ToolKind::Block => "⬛",
        }
    }

    pub fn shortcut(self) -> &'static str {
        match self {
            ToolKind::Select => "Shift+S",
            ToolKind::Move => "G",
            ToolKind::Scale => "S",
            ToolKind::Block => "Shift+B",
        }
    }
}

/// Mouse state seen by tools, already converted to world space.
pub struct ToolInput<'a> {
    pub proj: &'a Projector,
    pub grid: &'a GridSettings,
    /// Mouse position in screen space.
    pub pointer: Option<Pos2>,
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
    pub move_tool: MoveTool,
    pub scale: ScaleTool,
    pub block: BlockTool,
}

impl Default for Tools {
    fn default() -> Self {
        Self {
            active: ToolKind::Block,
            select: SelectTool,
            move_tool: MoveTool::default(),
            scale: ScaleTool::default(),
            block: BlockTool::default(),
        }
    }
}

impl Tools {
    pub fn update(&mut self, input: &ToolInput, scene: &mut Scene, history: &mut History) {
        match self.active {
            ToolKind::Select => self.select.update(input, scene),
            ToolKind::Move => self.move_tool.update(input, scene, history),
            ToolKind::Scale => self.scale.update(input, scene, history),
            ToolKind::Block => self.block.update(input, scene, history),
        }
    }

    /// Draws the active tool's handles and previews.
    pub fn draw(&self, painter: &Painter, proj: &Projector, scene: &Scene) {
        match self.active {
            ToolKind::Select => {}
            ToolKind::Move => self.move_tool.draw(painter, proj, scene),
            ToolKind::Scale => self.scale.draw(painter, proj, scene),
            ToolKind::Block => self.block.draw(painter, proj),
        }
    }

    /// Cancels the current action, restoring the scene. Returns `true` if something was cancelled.
    pub fn cancel(&mut self, scene: &mut Scene, history: &mut History) -> bool {
        self.move_tool.cancel(scene, history) | self.scale.cancel(scene, history) | self.block.cancel()
    }
}

#[cfg(test)]
mod tests {
    //! Simulates mouse drags on the tool handles in a top orthographic view.

    use eframe::egui::{Rect, pos2};

    use super::*;
    use crate::editor::OrbitCamera;
    use crate::math::Aabb;
    use crate::scene::Mesh;

    struct Harness {
        scene: Scene,
        history: History,
        grid: GridSettings,
        proj: Projector,
    }

    impl Harness {
        /// A 64³ block at the origin, selected, seen from the top.
        fn new() -> Self {
            let mut scene = Scene::default();
            let id = scene.add_object("Block", Mesh::cuboid(&Aabb::from_corners(Vec3::ZERO, Vec3::splat(64.0))));
            scene.selection.set(id);
            let mut camera = OrbitCamera::default();
            camera.target = Vec3::new(32.0, 32.0, 32.0);
            camera.look_from(Vec3::Z);
            let proj = Projector::new(&camera, Rect::from_min_size(pos2(0.0, 0.0), eframe::egui::vec2(800.0, 600.0)));
            Self { scene, history: History::default(), grid: GridSettings::default(), proj }
        }

        /// Hover, press at `from`, drag to `to`, release.
        fn drag(&mut self, tools: &mut Tools, from: Pos2, to: Pos2) {
            let Harness { scene, history, grid, proj } = self;
            let steps = [(from, false, false, false), (from, true, true, false), (to, false, true, false), (to, false, false, true)];
            for (p, pressed, down, released) in steps {
                let input = ToolInput {
                    proj,
                    grid,
                    pointer: Some(p),
                    ray: Some(proj.ray(p)),
                    plane: GridPlane::XY,
                    on_plane: None,
                    pressed,
                    down,
                    released,
                    clicked: false,
                    shift: false,
                };
                tools.update(&input, scene, history);
            }
        }

        fn bounds(&self) -> Aabb {
            self.scene.objects()[0].mesh.bounds()
        }

        fn screen(&self, p: Vec3) -> Pos2 {
            self.proj.point(p).unwrap()
        }
    }

    #[test]
    fn move_along_x_snaps_to_grid() {
        let mut h = Harness::new();
        let mut tools = Tools { active: ToolKind::Move, ..Default::default() };
        let pivot = h.bounds().center();
        // Grab the X arrow halfway, drag ~100 units to the right.
        let grab = h.screen(pivot + Vec3::X * h.proj.pixel_size_at(pivot) * 60.0);
        let target = grab + eframe::egui::vec2(100.0 / h.proj.pixel_size_at(pivot), 0.0);
        h.drag(&mut tools, grab, target);

        let b = h.bounds();
        assert_eq!(b.min.x % 16.0, 0.0, "min x must be on the grid: {b:?}");
        assert!((b.min.x - 96.0).abs() <= 16.0, "moved about 100 units: {b:?}");
        assert_eq!((b.min.y, b.min.z), (0.0, 0.0), "only X changes");
        assert_eq!(b.size(), Vec3::splat(64.0));

        // Undo restores the original position.
        h.history.undo(&mut h.scene);
        assert_eq!(h.bounds().min, Vec3::ZERO);
    }

    #[test]
    fn scale_max_x_keeps_opposite_side() {
        let mut h = Harness::new();
        let mut tools = Tools { active: ToolKind::Scale, ..Default::default() };
        let b = h.bounds();
        let side = Vec3::new(b.max.x, b.center().y, b.center().z);
        let handle = h.screen(side + Vec3::X * h.proj.pixel_size_at(side) * 14.0);
        let target = handle + eframe::egui::vec2(64.0 / h.proj.pixel_size_at(side), 0.0);
        h.drag(&mut tools, handle, target);

        let b = h.bounds();
        assert_eq!(b.min.x, 0.0, "opposite side stays: {b:?}");
        assert_eq!(b.max.x, 128.0, "moved side snapped: {b:?}");
        assert_eq!(b.size().y, 64.0);
    }
}
