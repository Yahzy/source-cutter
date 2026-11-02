use glam::Vec3;

/// Axis-aligned bounding box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

/// One face of a box: its 4 corner indices (cyclic order) and outward normal.
pub struct AabbFace {
    pub corners: [usize; 4],
    pub normal: Vec3,
}

impl Aabb {
    /// The 6 faces of a box. Corner index bits: 1 = max x, 2 = max y, 4 = max z.
    pub const FACES: [AabbFace; 6] = [
        AabbFace { corners: [0, 2, 6, 4], normal: Vec3::NEG_X },
        AabbFace { corners: [1, 3, 7, 5], normal: Vec3::X },
        AabbFace { corners: [0, 1, 5, 4], normal: Vec3::NEG_Y },
        AabbFace { corners: [2, 3, 7, 6], normal: Vec3::Y },
        AabbFace { corners: [0, 1, 3, 2], normal: Vec3::NEG_Z },
        AabbFace { corners: [4, 5, 7, 6], normal: Vec3::Z },
    ];

    /// The 12 edges of a box, as pairs of corner indices.
    pub const EDGES: [(usize, usize); 12] = [
        (0, 1), (2, 3), (4, 5), (6, 7), // along X
        (0, 2), (1, 3), (4, 6), (5, 7), // along Y
        (0, 4), (1, 5), (2, 6), (3, 7), // along Z
    ];

    /// Builds a box from two opposite corners given in any order.
    pub fn from_corners(a: Vec3, b: Vec3) -> Self {
        Self { min: a.min(b), max: a.max(b) }
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
