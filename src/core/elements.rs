use super::features::CellType;
use crate::graphics::models::space::SrtTransform;
use crate::physics::objects;
use crate::physics::objects::ObjectData2D;
use crate::utils::vector::Vec2d;
use glam::Vec2;

pub type CellId = usize;

pub struct CellConnection {
    pub id_a: CellId,
    pub angle_a: f64,

    pub id_b: CellId,
    pub angle_b: f64,
}

impl CellConnection {
    pub fn new(id_a: CellId, angle_a: f64, id_b: CellId, angle_b: f64) -> Self {
        Self {
            id_a,
            angle_a,

            id_b,
            angle_b,
        }
    }

    pub fn points_toward(&self, id: CellId) -> bool {
        self.id_a == id || self.id_b == id
    }
}

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
    pub fn new(pos: Vec2d, typ: CellType) -> Self {
        let disk_approx = objects::Disk::from_mass(1.0, 1.0);

        Self {
            mass: disk_approx.mass(),
            angular_inertia: disk_approx.rotational_inertia(),

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

    pub fn position(&self) -> Vec2 {
        Vec2::new(self.position.x as f32, self.position.y as f32)
    }

    pub fn rotation(&self) -> f32 {
        self.angle as f32
    }

    pub fn get_transform(&self) -> SrtTransform {
        SrtTransform {
            translate: self.position(),
            rotate: self.rotation(),
            scale: Vec2::splat(self.size as f32),
        }
    }
}
