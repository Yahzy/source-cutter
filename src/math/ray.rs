use glam::Vec3;

/// A half-line used for picking (mouse → world).
#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub origin: Vec3,
    /// Normalized direction.
    pub dir: Vec3,
}

impl Ray {
    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + self.dir * t
    }

    /// Distance along the ray to the plane through `point` with `normal`, if hit in front.
    pub fn intersect_plane(&self, point: Vec3, normal: Vec3) -> Option<f32> {
        let denom = self.dir.dot(normal);
        if denom.abs() < 1e-6 {
            return None;
        }
        let t = (point - self.origin).dot(normal) / denom;
        (t > 0.0).then_some(t)
    }

    /// Distance to a triangle, both sides (Möller–Trumbore).
    pub fn intersect_triangle(&self, a: Vec3, b: Vec3, c: Vec3) -> Option<f32> {
        let e1 = b - a;
        let e2 = c - a;
        let p = self.dir.cross(e2);
        let det = e1.dot(p);
        if det.abs() < 1e-8 {
            return None;
        }
        let inv_det = 1.0 / det;
        let s = self.origin - a;
        let u = s.dot(p) * inv_det;
        if !(0.0..=1.0).contains(&u) {
            return None;
        }
        let q = s.cross(e1);
        let v = self.dir.dot(q) * inv_det;
        if v < 0.0 || u + v > 1.0 {
            return None;
        }
        let t = e2.dot(q) * inv_det;
        (t > 0.0).then_some(t)
    }

    /// Parameter `s` of the point on the line `origin + s * axis` closest to this ray.
    /// Used to drag things along an axis with the mouse.
    pub fn closest_on_line(&self, origin: Vec3, axis: Vec3) -> Option<f32> {
        let b = axis.dot(self.dir);
        let denom = axis.length_squared() - b * b;
        if denom.abs() < 1e-6 {
            return None; // Ray parallel to the axis.
        }
        let w = origin - self.origin;
        let d = axis.dot(w);
        let e = self.dir.dot(w);
        Some((b * e - d) / denom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hits_ground_plane() {
        let ray = Ray { origin: Vec3::new(5.0, 5.0, 10.0), dir: Vec3::new(0.0, 0.6, -0.8) };
        let t = ray.intersect_plane(Vec3::ZERO, Vec3::Z).unwrap();
        assert!((ray.at(t).z).abs() < 1e-5);
    }

    #[test]
    fn closest_point_on_axis() {
        // Ray looking down at x = 10: closest point on the X axis is s = 10.
        let ray = Ray { origin: Vec3::new(10.0, 0.0, 50.0), dir: Vec3::NEG_Z };
        let s = ray.closest_on_line(Vec3::ZERO, Vec3::X).unwrap();
        assert!((s - 10.0).abs() < 1e-4);
    }
}
