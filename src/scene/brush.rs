use crate::math::Aabb;

/// Stable identifier of a brush inside a scene.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BrushId(pub u32);

/// A solid piece of geometry. For now a brush is a simple box;
/// later it can become a convex polyhedron defined by planes (like Hammer).
#[derive(Clone, Debug)]
pub struct Brush {
    pub id: BrushId,
    pub name: String,
    pub bounds: Aabb,
    /// Display color (RGB) used in the viewport.
    pub color: [u8; 3],
}

/// Muted colors given to new brushes, in rotation.
const PALETTE: [[u8; 3]; 6] = [
    [150, 150, 160],
    [165, 140, 115],
    [120, 150, 135],
    [140, 130, 165],
    [170, 160, 120],
    [120, 140, 165],
];

impl Brush {
    pub fn new(id: BrushId, bounds: Aabb) -> Self {
        Self {
            id,
            name: format!("Brush {}", id.0),
            bounds,
            color: PALETTE[id.0 as usize % PALETTE.len()],
        }
    }
}
