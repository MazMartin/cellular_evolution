use crate::core::elements::Cell;
use crate::core::sim::SimulationState;
use crate::physics::forces::{ForceApplier, ForceAppl, Lever, LinearSpring};
use crate::utils::vector::Vec2d;

impl SimulationState {
    pub fn physics_pass(&mut self, dt: f64) {
        for connection in self.connections.iter() {
            let (cell_a, cell_b) = self.cells.get_mut_pair(connection.id_a, connection.id_b);

            LinearSpring {
                length: 2.0,
                k: 50.0,
            }
            .tick(cell_a, cell_b);

            LinearSpring {
                length: 0.0,
                k: 50.0,
            }
            .tick(
                &mut cell_a.edge_lever(connection.angle_a),
                &mut cell_b.edge_lever(connection.angle_b),
            );
        }

        for cell in self.cells.flatten_iter_mut() {
            let force = -cell.velocity * cell.size * self.context.viscosity;
            let torque = -cell.angular_velocity * cell.size * self.context.viscosity;

            cell.apply_force(force);
            cell.apply_torque(torque);
        }

        for cell in self.cells.flatten_iter_mut() {
            cell.integrate(dt)
        }
    }
}

impl Cell {
    pub fn edge_lever(&mut self, angle: f64) -> Lever<Self> {
        let application = Vec2d::from_angle(self.angle + angle) * self.size * 0.5;
        Lever {
            body: self,
            application,
        }
    }

    fn integrate(&mut self, dt: f64) {
        self.velocity += self.force * dt / self.mass;
        self.angular_velocity += self.torque * dt / self.angular_inertia;

        self.force = Vec2d::ZERO;
        self.torque = 0.;

        self.position += self.velocity * dt;
        self.angle += self.angular_velocity * dt;
    }
}
