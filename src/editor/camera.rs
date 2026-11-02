use glam::camera::rh::{proj, view};
use glam::{Mat4, Vec3};

use crate::math::Aabb;

/// Orbit camera around a target point. Source Engine is Z-up.
pub struct OrbitCamera {
    pub target: Vec3,
    /// Horizontal angle (radians) of the eye around the target, from +X towards +Y.
    pub yaw: f32,
    /// Vertical angle (radians) of the eye above the ground plane.
    pub pitch: f32,
    pub distance: f32,
    /// Vertical field of view (radians).
    pub fov_y: f32,
}

impl OrbitCamera {
    pub const NEAR: f32 = 1.0;
    pub const FAR: f32 = 200_000.0;
    const MIN_DISTANCE: f32 = 4.0;
    const MAX_DISTANCE: f32 = 60_000.0;
    const MAX_PITCH: f32 = 89.9_f32.to_radians();
    const ORBIT_SPEED: f32 = 0.008;
    const ZOOM_SPEED: f32 = 0.0015;
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,
            yaw: -45_f32.to_radians(),
            pitch: 30_f32.to_radians(),
            distance: 400.0,
            fov_y: 70_f32.to_radians(),
        }
    }
}

impl OrbitCamera {
    /// Unit vector from the target towards the eye.
    pub fn eye_direction(&self) -> Vec3 {
        let (sp, cp) = self.pitch.sin_cos();
        let (sy, cy) = self.yaw.sin_cos();
        Vec3::new(cp * cy, cp * sy, sp)
    }

    pub fn eye(&self) -> Vec3 {
        self.target + self.eye_direction() * self.distance
    }

    pub fn view_matrix(&self) -> Mat4 {
        view::look_at_mat4(self.eye(), self.target, Vec3::Z)
    }

    pub fn projection_matrix(&self, aspect: f32) -> Mat4 {
        // Depth range [0, 1], which `Projector::ray` relies on.
        proj::directx::perspective(self.fov_y, aspect, Self::NEAR, Self::FAR)
    }

    // --- Navigation ------------------------------------------------------

    /// Rotates around the target from a mouse delta in pixels.
    pub fn orbit(&mut self, dx: f32, dy: f32) {
        self.yaw -= dx * Self::ORBIT_SPEED;
        self.pitch = (self.pitch + dy * Self::ORBIT_SPEED).clamp(-Self::MAX_PITCH, Self::MAX_PITCH);
    }

    /// Moves the target so the scene follows the mouse.
    pub fn pan(&mut self, dx: f32, dy: f32, viewport_height: f32) {
        let units_per_pixel = 2.0 * self.distance * (self.fov_y * 0.5).tan() / viewport_height.max(1.0);
        let forward = -self.eye_direction();
        let right = forward.cross(Vec3::Z).normalize_or(Vec3::X);
        let up = right.cross(forward);
        self.target += (-right * dx + up * dy) * units_per_pixel;
    }

    /// Zooms from a scroll delta (positive = closer).
    pub fn zoom(&mut self, scroll: f32) {
        self.distance = (self.distance * (-scroll * Self::ZOOM_SPEED).exp())
            .clamp(Self::MIN_DISTANCE, Self::MAX_DISTANCE);
    }

    /// Places the eye on the side given by `direction` (e.g. +Z = top view).
    pub fn look_from(&mut self, direction: Vec3) {
        let d = direction.normalize();
        if d.x.abs() < 1e-4 && d.y.abs() < 1e-4 {
            // Top / bottom view: X to the right, Y up on screen.
            self.yaw = -90_f32.to_radians();
        } else {
            self.yaw = d.y.atan2(d.x);
        }
        self.pitch = d.z.clamp(-1.0, 1.0).asin().clamp(-Self::MAX_PITCH, Self::MAX_PITCH);
    }

    /// Centers the camera on a box.
    pub fn frame(&mut self, bounds: &Aabb) {
        self.target = bounds.center();
        self.distance = (bounds.size().length() * 1.2).clamp(64.0, Self::MAX_DISTANCE);
    }
}

