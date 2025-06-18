use crate::core::elements::{Cell, CellConnection};
use crate::core::sim::SimulationState;
use crate::physics::forces::{ForceApplier, ForceAppl, Lever, LinearSpring};
use crate::utils::vector::Vec2d;

impl SimulationState {
    /// Performs one physics step for the entire simulation.
    /// Applies spring constraints, viscous damping, and integrates cell motion.
    pub fn physics_pass(&mut self, dt: f64) {
        // Apply spring forces between all connected cell pairs.
        for connection in self.connections.iter() {
            let (cell_a, cell_b) = self
                .cells
                .get_mut_pair(connection.id_a, connection.id_b);

            // Primary spring connects the cell centers.
            LinearSpring {
                length: 2.0,
                k: 50.0,
            }
                .tick(cell_a, cell_b);

            // Secondary spring connects the edge points (angled offset from center).
            LinearSpring {
                length: 0.0,
                k: 50.0,
            }
                .tick(
                    &mut cell_a.edge_lever(connection.angle_a),
                    &mut cell_b.edge_lever(connection.angle_b),
                );
        }

        // Apply viscous drag and update physics state for each cell.
        for cell in self.cells.flatten_iter_mut() {
            apply_viscous_force(cell, self.context.viscosity);
            cell.apply_force_integrate(dt);
        }
    }
}

/// Applies viscous damping force and torque based on velocity and angular velocity.
fn apply_viscous_force(cell: &mut Cell, viscosity: f64) {
    let force = -cell.velocity * cell.size * viscosity;
    let torque = -cell.angular_velocity * cell.size * viscosity;

    cell.apply_force(force);
    cell.apply_torque(torque);
}

impl Cell {
    /// Returns a lever arm from the center of mass to a rotated edge point on the cell.
    pub fn edge_lever(&mut self, angle: f64) -> Lever<Self> {
        let direction = Vec2d::from_angle(self.angle + angle);
        let application = direction * self.size * 0.5;

        Lever {
            body: self,
            application,
        }
    }

    /// Applies Newtonian motion integration: updates velocity and position based on accumulated forces.
    fn apply_force_integrate(&mut self, dt: f64) {
        // Linear motion
        self.velocity += self.force * dt / self.mass;
        self.position += self.velocity * dt;

        // Angular motion
        self.angular_velocity += self.torque * dt / self.angular_inertia;
        self.angle += self.angular_velocity * dt;

        // Reset accumulated forces and torque
        self.force = Vec2d::ZERO;
        self.torque = 0.0;
    }
}
