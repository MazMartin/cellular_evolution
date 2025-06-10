use crate::core::elements::Cell;
use crate::utils::vector::Vec2d;

pub trait ForceAppl {
    fn apply_force(&mut self, force: Vec2d);
    fn apply_torque(&mut self, torque: f64);
    fn pos(&self) -> Vec2d;
}

pub trait ForceApplier<T: ForceAppl> {
    fn tick(&mut self, a: &mut T, b: &mut T);
}

pub struct Lever<'a, T: ForceAppl> {
    pub body: &'a mut T,
    pub application: Vec2d,
}

impl<'a, T: ForceAppl> ForceAppl for Lever<'a, T> {
    fn apply_force(&mut self, force: Vec2d) {
        self.body.apply_force(force);
        let torque = self.application.perp_dot(force);
        self.body.apply_torque(torque);
    }

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

    fn pos(&self) -> Vec2d {
        self.body.pos() + self.application
    }
}

pub struct LinearSpring {
    pub length: f64,
    pub k: f64,
}

impl<T: ForceAppl> ForceApplier<T> for LinearSpring {
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
    fn apply_force(&mut self, force: Vec2d) {
        self.force += force;
    }
    fn apply_torque(&mut self, torque: f64) {
        self.torque += torque;
    }
    fn pos(&self) -> Vec2d {
        self.position
    }
}
