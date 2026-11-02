//! Source Engine unit conventions.
//!
//! In Source, world geometry is measured in "Hammer units".
//! For map/world scale, 1 unit = 1 inch (16 units = 1 foot).

/// Meters per Hammer unit (world scale: 1 unit = 1 inch).
pub const METERS_PER_UNIT: f32 = 0.0254;

/// Smallest grid size available (like Hammer).
pub const GRID_MIN: u32 = 1;
/// Biggest grid size available (like Hammer).
pub const GRID_MAX: u32 = 512;
/// Grid size at startup.
pub const GRID_DEFAULT: u32 = 16;

/// Hammer highlights a grid line every 64 units.
pub const GRID_HIGHLIGHT_STEP: f32 = 64.0;
/// Hammer draws a stronger line every 1024 units.
pub const GRID_MAJOR_STEP: f32 = 1024.0;

/// A Source map spans from -16384 to +16384 on every axis.
pub const MAP_HALF_EXTENT: f32 = 16384.0;

/// Player standing hull (HL2 / Source SDK defaults).
pub const PLAYER_WIDTH: f32 = 32.0;
pub const PLAYER_HEIGHT: f32 = 72.0;

/// Formats a length in units with its metric equivalent, e.g. `64 u (1.63 m)`.
pub fn format_length(units: f32) -> String {
    format!("{} u ({:.2} m)", units, units * METERS_PER_UNIT)
}
