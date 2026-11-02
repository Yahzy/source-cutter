use glam::Vec3;

/// Axis-aligned bounding box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    /// The 12 edges of a box, as pairs of corner indices.
    /// Corner index bits: 1 = max x, 2 = max y, 4 = max z.
    pub const EDGES: [(usize, usize); 12] = [
        (0, 1), (2, 3), (4, 5), (6, 7), // along X
        (0, 2), (1, 3), (4, 6), (5, 7), // along Y
        (0, 4), (1, 5), (2, 6), (3, 7), // along Z
    ];

    /// Builds a box from two opposite corners given in any order.
    pub fn from_corners(a: Vec3, b: Vec3) -> Self {
        Self { min: a.min(b), max: a.max(b) }
    }

    /// Smallest box containing all points (a zero box at the origin if empty).
    pub fn from_points(points: &[Vec3]) -> Self {
        let Some(first) = points.first() else {
            return Self { min: Vec3::ZERO, max: Vec3::ZERO };
        };
        points.iter().fold(Self { min: *first, max: *first }, |b, p| Self {
            min: b.min.min(*p),
            max: b.max.max(*p),
        })
    }

    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// True when the box has no volume (flat or empty on at least one axis).
    pub fn is_degenerate(&self) -> bool {
        self.size().min_element() <= 0.0
    }

    pub fn corner(&self, index: usize) -> Vec3 {
        Vec3::new(
            if index & 1 != 0 { self.max.x } else { self.min.x },
            if index & 2 != 0 { self.max.y } else { self.min.y },
            if index & 4 != 0 { self.max.z } else { self.min.z },
        )
    }

    pub fn corners(&self) -> [Vec3; 8] {
        std::array::from_fn(|i| self.corner(i))
    }
}
