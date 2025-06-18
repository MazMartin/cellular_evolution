use crate::core::elements::Cell;
use crate::utils::vector::Vec2d;

/// Trait for objects that can have forces and torques applied to them,
/// and can provide their position.
pub trait ForceAppl {
    fn apply_force(&mut self, force: Vec2d);
    fn apply_torque(&mut self, torque: f64);
    fn pos(&self) -> Vec2d;
}

/// Trait for objects that apply forces between two ForceAppl instances.
pub trait ForceApplier<T: ForceAppl> {
    fn tick(&mut self, a: &mut T, b: &mut T);
}

/// Represents a lever applying force and torque at a specific application point.
pub struct Lever<'a, T: ForceAppl> {
    pub body: &'a mut T,
    pub application: Vec2d,
}

impl<'a, T: ForceAppl> ForceAppl for Lever<'a, T> {
    /// Applies a force to the lever's body and calculates the resulting torque.
    fn apply_force(&mut self, force: Vec2d) {
        self.body.apply_force(force);
        let torque = self.application.perp_dot(force);
        self.body.apply_torque(torque);
    }

    /// Applies torque, converting it to an equivalent force if possible.
    fn apply_torque(&mut self, torque: f64) {
        let r_mag = self.application.length();
        if r_mag > 1e-10 {
            let force_direction = self.application.perp() / r_mag;
            let force_magnitude = torque / r_mag;
            let force = force_direction * force_magnitude;
            self.body.apply_force(force);
        } else {
            self.body.apply_torque(torque);
        }
    }

    /// Returns the position where force is applied.
    fn pos(&self) -> Vec2d {
        self.body.pos() + self.application
    }
}

/// A linear spring applying forces between two ForceAppl objects,
/// based on Hooke's law.
pub struct LinearSpring {
    pub length: f64,
    pub k: f64,
}

impl<T: ForceAppl> ForceApplier<T> for LinearSpring {
    /// Updates forces on two objects based on their distance and spring parameters.
    fn tick(&mut self, a: &mut T, b: &mut T) {
        let delta = b.pos() - a.pos();
        let stretch = delta.length() - self.length;
        let force_mag = -self.k * stretch;
        let force_dir = delta.normalize();
        let force = force_dir * force_mag;

        a.apply_force(force * -1.0);
        b.apply_force(force);
    }
}

impl ForceAppl for Cell {
    /// Adds force to the cell's force accumulator.
    fn apply_force(&mut self, force: Vec2d) {
        self.force += force;
    }
    /// Adds torque to the cell's torque accumulator.
    fn apply_torque(&mut self, torque: f64) {
        self.torque += torque;
    }
    /// Returns the cell's current position.
    fn pos(&self) -> Vec2d {
        self.position
    }
}
