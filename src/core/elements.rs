use super::features::CellType;
use crate::graphics::models::space::SrtTransform;
use crate::physics::objects;
use crate::physics::objects::ObjectData2D;
use crate::utils::vector::Vec2d;
use glam::Vec2;

/// Type alias for identifying a cell.
pub type CellId = usize;

/// Represents a directional connection between two cells.
pub struct CellConnection {
    pub id_a: CellId,
    pub angle_a: f64,

    pub id_b: CellId,
    pub angle_b: f64,
}

impl CellConnection {
    /// Creates a new connection between two cells with specified angles.
    pub fn new(id_a: CellId, angle_a: f64, id_b: CellId, angle_b: f64) -> Self {
        Self {
            id_a,
            angle_a,
            id_b,
            angle_b,
        }
    }

    /// Returns `true` if this connection involves the given cell ID.
    pub fn points_toward(&self, id: CellId) -> bool {
        self.id_a == id || self.id_b == id
    }
}

/// A single cell in a physics-based simulation.
/// It contains physical properties such as position, mass, velocity, and angular data.
#[derive(Clone, Debug)]
pub struct Cell {
    pub force: Vec2d,
    pub mass: f64,
    pub position: Vec2d,
    pub velocity: Vec2d,

    pub torque: f64,
    pub angular_inertia: f64,
    pub angle: f64,
    pub angular_velocity: f64,

    pub size: f64,
    pub typ: CellType,
}

impl Cell {
    /// Creates a new cell at a given position with a given type.
    /// Initializes with default physics and size.
    pub fn new(pos: Vec2d, typ: CellType) -> Self {
        let disk = objects::Disk::from_mass(1.0, 1.0); // Approximate circular object

        Self {
            mass: disk.mass(),
            angular_inertia: disk.rotational_inertia(),

            force: Vec2d::ZERO,
            position: pos,
            velocity: Vec2d::ZERO,
            torque: 0.0,
            angle: 0.0,
            angular_velocity: 0.0,

            size: 1.0,
            typ,
        }
    }

    /// Returns the 2D position as a `Vec2` for rendering.
    pub fn position(&self) -> Vec2 {
        Vec2::new(self.position.x as f32, self.position.y as f32)
    }

    /// Returns the rotation angle as a `f32` in radians.
    pub fn rotation(&self) -> f32 {
        self.angle as f32
    }

    /// Returns the current transform of the cell (position, rotation, scale).
    pub fn get_transform(&self) -> SrtTransform {
        SrtTransform {
            translate: self.position(),
            rotate: self.rotation(),
            scale: Vec2::splat(self.size as f32),
        }
    }
}
