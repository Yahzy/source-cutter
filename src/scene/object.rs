use super::Mesh;

/// Stable identifier of an object inside a scene.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObjectId(pub u32);

/// A named mesh. Vertices are stored in world space, like Hammer brushes.
#[derive(Clone, Debug, PartialEq)]
pub struct Object {
    pub id: ObjectId,
    pub name: String,
    pub mesh: Mesh,
    /// Display color (RGB) used in the viewport.
    pub color: [u8; 3],
}

/// Muted colors given to new objects, in rotation.
const PALETTE: [[u8; 3]; 6] = [
    [150, 150, 160],
    [165, 140, 115],
    [120, 150, 135],
    [140, 130, 165],
    [170, 160, 120],
    [120, 140, 165],
];

impl Object {
    pub fn new(id: ObjectId, name: String, mesh: Mesh) -> Self {
        Self { id, name, mesh, color: PALETTE[id.0 as usize % PALETTE.len()] }
    }
}
