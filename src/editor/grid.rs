use glam::Vec3;

use super::OrbitCamera;
use crate::units::{GRID_DEFAULT, GRID_MAX, GRID_MIN};

/// Plane the grid is drawn on and tools work on.
/// XY in perspective; follows the view in orthographic axis views (like Hammer's 2D views).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridPlane {
    XY,
    XZ,
    YZ,
}

impl GridPlane {
    pub fn from_camera(camera: &OrbitCamera) -> Self {
        match camera.aligned_axis() {
            Some(a) if a.x != 0.0 => GridPlane::YZ,
            Some(a) if a.y != 0.0 => GridPlane::XZ,
            _ => GridPlane::XY,
        }
    }

    pub fn normal(self) -> Vec3 {
        match self {
            GridPlane::XY => Vec3::Z,
            GridPlane::XZ => Vec3::Y,
            GridPlane::YZ => Vec3::X,
        }
    }

    /// The two axes spanning the plane.
    pub fn axes(self) -> [Vec3; 2] {
        match self {
            GridPlane::XY => [Vec3::X, Vec3::Y],
            GridPlane::XZ => [Vec3::X, Vec3::Z],
            GridPlane::YZ => [Vec3::Y, Vec3::Z],
        }
    }
}

/// Hammer-style grid: power-of-two size in units, with snapping.
pub struct GridSettings {
    size: u32,
    pub visible: bool,
    pub snap: bool,
}

impl Default for GridSettings {
    fn default() -> Self {
        Self { size: GRID_DEFAULT, visible: true, snap: true }
    }
}

impl GridSettings {
    pub fn size(&self) -> u32 {
        self.size
    }

    /// `]` in Hammer.
    pub fn increase(&mut self) {
        self.size = (self.size * 2).min(GRID_MAX);
    }

    /// `[` in Hammer.
    pub fn decrease(&mut self) {
        self.size = (self.size / 2).max(GRID_MIN);
    }

    pub fn snap_value(&self, value: f32) -> f32 {
        if self.snap {
            let step = self.size as f32;
            (value / step).round() * step
        } else {
            value.round()
        }
    }

    pub fn snap_point(&self, p: Vec3) -> Vec3 {
        Vec3::new(self.snap_value(p.x), self.snap_value(p.y), self.snap_value(p.z))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snaps_to_grid_size() {
        let grid = GridSettings::default(); // 16
        assert_eq!(grid.snap_value(23.0), 16.0);
        assert_eq!(grid.snap_value(25.0), 32.0);
        assert_eq!(grid.snap_value(-9.0), -16.0);
    }

    #[test]
    fn size_stays_in_hammer_range() {
        let mut grid = GridSettings::default();
        for _ in 0..20 {
            grid.increase();
        }
        assert_eq!(grid.size(), GRID_MAX);
        for _ in 0..20 {
            grid.decrease();
        }
        assert_eq!(grid.size(), GRID_MIN);
    }
}
