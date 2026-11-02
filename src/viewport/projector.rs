use eframe::egui::{Pos2, Rect, pos2};
use glam::{Mat4, Vec3};

use crate::editor::OrbitCamera;
use crate::math::Ray;

/// Converts between world space and screen space for one frame.
/// Geometry is clipped against the near plane before projection.
pub struct Projector {
    view: Mat4,
    proj: Mat4,
    inv_view_proj: Mat4,
    rect: Rect,
    eye: Vec3,
}

impl Projector {
    pub fn new(camera: &OrbitCamera, rect: Rect) -> Self {
        let view = camera.view_matrix();
        let proj = camera.projection_matrix(rect.width() / rect.height().max(1.0));
        Self { view, proj, inv_view_proj: (proj * view).inverse(), rect, eye: camera.eye() }
    }

    /// Distance from the camera plane (bigger = farther). Used to sort faces.
    pub fn depth(&self, p: Vec3) -> f32 {
        -self.view.transform_point3(p).z
    }

    /// True when a face with this center and normal faces the camera.
    pub fn is_front_facing(&self, center: Vec3, normal: Vec3) -> bool {
        normal.dot(self.eye - center) > 0.0
    }

    /// Rotates a world direction into view space (x right, y up, z towards the viewer).
    pub fn view_direction(&self, dir: Vec3) -> Vec3 {
        self.view.transform_vector3(dir)
    }

    /// Projects a world point. `None` when behind the camera.
    pub fn point(&self, p: Vec3) -> Option<Pos2> {
        let v = self.view.transform_point3(p);
        (v.z < -OrbitCamera::NEAR).then(|| self.view_to_screen(v))
    }

    /// Projects a segment, clipped against the near plane.
    pub fn segment(&self, a: Vec3, b: Vec3) -> Option<[Pos2; 2]> {
        let mut va = self.view.transform_point3(a);
        let mut vb = self.view.transform_point3(b);
        let near = -OrbitCamera::NEAR;

        match (va.z < near, vb.z < near) {
            (false, false) => return None,
            (true, false) => vb = clip_to_near(va, vb, near),
            (false, true) => va = clip_to_near(vb, va, near),
            (true, true) => {}
        }
        Some([self.view_to_screen(va), self.view_to_screen(vb)])
    }

    /// Projects a convex polygon, clipped against the near plane.
    pub fn polygon(&self, points: &[Vec3]) -> Vec<Pos2> {
        let near = -OrbitCamera::NEAR;
        let view: Vec<Vec3> = points.iter().map(|p| self.view.transform_point3(*p)).collect();
        let mut out = Vec::with_capacity(view.len() + 2);

        // Sutherland–Hodgman against a single plane.
        for i in 0..view.len() {
            let cur = view[i];
            let next = view[(i + 1) % view.len()];
            let cur_in = cur.z < near;
            let next_in = next.z < near;
            if cur_in {
                out.push(self.view_to_screen(cur));
            }
            if cur_in != next_in {
                let inside = if cur_in { cur } else { next };
                let outside = if cur_in { next } else { cur };
                out.push(self.view_to_screen(clip_to_near(inside, outside, near)));
            }
        }
        out
    }

    /// World-space ray going through a screen position.
    pub fn ray(&self, screen: Pos2) -> Ray {
        let x = (screen.x - self.rect.left()) / self.rect.width() * 2.0 - 1.0;
        let y = 1.0 - (screen.y - self.rect.top()) / self.rect.height() * 2.0;
        let near = self.inv_view_proj.project_point3(Vec3::new(x, y, 0.0));
        let far = self.inv_view_proj.project_point3(Vec3::new(x, y, 1.0));
        Ray { origin: near, dir: (far - near).normalize() }
    }

    fn view_to_screen(&self, v: Vec3) -> Pos2 {
        let ndc = self.proj.project_point3(v);
        pos2(
            self.rect.left() + (ndc.x + 1.0) * 0.5 * self.rect.width(),
            self.rect.top() + (1.0 - ndc.y) * 0.5 * self.rect.height(),
        )
    }
}

/// Point where the segment `inside → outside` crosses the near plane (view space).
fn clip_to_near(inside: Vec3, outside: Vec3, near: f32) -> Vec3 {
    let t = (near - inside.z) / (outside.z - inside.z);
    inside.lerp(outside, t)
}
