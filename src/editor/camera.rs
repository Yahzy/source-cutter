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
    /// Vertical field of view (radians). In orthographic mode it sets the zoom level.
    pub fov_y: f32,
    pub orthographic: bool,
    /// Orthographic was turned on by clicking the compass: orbiting goes back to
    /// perspective (Blender's "auto perspective").
    auto_ortho: bool,
}

impl OrbitCamera {
    pub const NEAR: f32 = 1.0;
    pub const FAR: f32 = 200_000.0;
    const MIN_DISTANCE: f32 = 4.0;
    const MAX_DISTANCE: f32 = 60_000.0;
    const MAX_PITCH: f32 = 89.9_f32.to_radians();
    const ORBIT_SPEED: f32 = 0.008;
    const ZOOM_SPEED: f32 = 0.0015;
    /// In orthographic mode the eye is pushed far back so nothing gets clipped.
    const ORTHO_EYE_DISTANCE: f32 = 65_536.0;
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,
            yaw: -45_f32.to_radians(),
            pitch: 30_f32.to_radians(),
            distance: 400.0,
            fov_y: 70_f32.to_radians(),
            orthographic: false,
            auto_ortho: false,
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

    /// Eye position used for rendering (pushed back in orthographic mode).
    pub fn render_eye(&self) -> Vec3 {
        if self.orthographic {
            self.target + self.eye_direction() * Self::ORTHO_EYE_DISTANCE
        } else {
            self.eye()
        }
    }

    pub fn view_matrix(&self) -> Mat4 {
        view::look_at_mat4(self.render_eye(), self.target, Vec3::Z)
    }

    /// Half of the visible height at the target distance, in units.
    pub fn half_height(&self) -> f32 {
        self.distance * (self.fov_y * 0.5).tan()
    }

    pub fn projection_matrix(&self, aspect: f32) -> Mat4 {
        // Depth range [0, 1], which `Projector::ray` relies on.
        if self.orthographic {
            let h = self.half_height();
            let w = h * aspect;
            proj::directx::orthographic(-w, w, -h, h, Self::NEAR, Self::FAR)
        } else {
            proj::directx::perspective(self.fov_y, aspect, Self::NEAR, Self::FAR)
        }
    }

    // --- Views -------------------------------------------------------------

    /// Signed world axis the camera looks from, when in an orthographic axis view.
    pub fn aligned_axis(&self) -> Option<Vec3> {
        if !self.orthographic {
            return None;
        }
        let d = self.eye_direction();
        [Vec3::X, Vec3::Y, Vec3::Z]
            .into_iter()
            .find(|a| d.dot(*a).abs() > 0.9999)
            .map(|a| a * d.dot(a).signum())
    }

    /// Blender-style view name, e.g. "Top Orthographic".
    pub fn view_name(&self) -> String {
        let side = match self.aligned_axis() {
            Some(a) if a == Vec3::Z => "Top",
            Some(a) if a == Vec3::NEG_Z => "Bottom",
            Some(a) if a == Vec3::NEG_Y => "Front",
            Some(a) if a == Vec3::Y => "Back",
            Some(a) if a == Vec3::X => "Right",
            Some(a) if a == Vec3::NEG_X => "Left",
            _ => "User",
        };
        let kind = if self.orthographic { "Orthographic" } else { "Perspective" };
        format!("{side} {kind}")
    }

    /// Switches perspective / orthographic (Blender: numpad 5).
    pub fn toggle_projection(&mut self) {
        self.orthographic = !self.orthographic;
        self.auto_ortho = false;
    }

    // --- Navigation ------------------------------------------------------

    /// Rotates around the target from a mouse delta in pixels.
    pub fn orbit(&mut self, dx: f32, dy: f32) {
        if self.auto_ortho {
            self.orthographic = false;
            self.auto_ortho = false;
        }
        self.yaw -= dx * Self::ORBIT_SPEED;
        self.pitch = (self.pitch + dy * Self::ORBIT_SPEED).clamp(-Self::MAX_PITCH, Self::MAX_PITCH);
    }

    /// Moves the target so the scene follows the mouse.
    pub fn pan(&mut self, dx: f32, dy: f32, viewport_height: f32) {
        let units_per_pixel = 2.0 * self.half_height() / viewport_height.max(1.0);
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

    /// Places the eye on the side given by `direction` (e.g. +Z = top view)
    /// and switches to orthographic, like clicking Blender's compass.
    pub fn look_from(&mut self, direction: Vec3) {
        if !self.orthographic || self.auto_ortho {
            self.orthographic = true;
            self.auto_ortho = true;
        }
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

