use glam::Vec3;

use super::Aabb;

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

    /// Distance along the ray to the horizontal plane `z = height`, if hit in front.
    pub fn intersect_plane_z(&self, height: f32) -> Option<f32> {
        if self.dir.z.abs() < 1e-6 {
            return None;
        }
        let t = (height - self.origin.z) / self.dir.z;
        (t > 0.0).then_some(t)
    }

    /// Distance along the ray to the first hit with the box (slab method).
    pub fn intersect_aabb(&self, aabb: &Aabb) -> Option<f32> {
        let inv = self.dir.recip();
        let t1 = (aabb.min - self.origin) * inv;
        let t2 = (aabb.max - self.origin) * inv;
        let t_near = t1.min(t2).max_element();
        let t_far = t1.max(t2).min_element();

        if t_far < t_near.max(0.0) {
            return None;
        }
        Some(if t_near > 0.0 { t_near } else { t_far })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hits_box_in_front() {
        let ray = Ray { origin: Vec3::new(0.0, 0.0, 100.0), dir: Vec3::NEG_Z };
        let aabb = Aabb::from_corners(Vec3::splat(-16.0), Vec3::splat(16.0));
        assert_eq!(ray.intersect_aabb(&aabb), Some(84.0));
    }

    #[test]
    fn misses_box_behind() {
        let ray = Ray { origin: Vec3::new(0.0, 0.0, 100.0), dir: Vec3::Z };
        let aabb = Aabb::from_corners(Vec3::splat(-16.0), Vec3::splat(16.0));
        assert_eq!(ray.intersect_aabb(&aabb), None);
    }

    #[test]
    fn hits_ground_plane() {
        let ray = Ray { origin: Vec3::new(5.0, 5.0, 10.0), dir: Vec3::new(0.0, 0.6, -0.8) };
        let t = ray.intersect_plane_z(0.0).unwrap();
        assert!((ray.at(t).z).abs() < 1e-5);
    }
}
