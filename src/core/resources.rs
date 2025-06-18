use std::ops::Sub;
use crate::core::sim::SimulationState;

/// Type alias representing units of energy (abstract scale).
type Energy = f32;

/// Type alias representing units of stored fat (abstract scale).
type Fat = f32;

/// Represents localized, shareable resources stored in a cell.
#[derive(Clone, Copy, Debug, Default)]
pub struct LocalResources {
    energy: Energy,
    fat: Fat,
}

impl Sub for LocalResources {
    type Output = Self;

    /// Subtracts one resource set from another, field-by-field.
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            energy: self.energy - rhs.energy,
            fat: self.fat - rhs.fat,
        }
    }
}

impl SimulationState {
    /// Placeholder for resource-sharing logic between connected cells.
    /// Will compute transfer of energy/fat through `CellConnection`s over time `dt`.
    pub fn share_resources_pass(&mut self, dt: f64) {
        for connection in self.connections.iter() {
            let (cell_a, cell_b) = self.cells.get_mut_pair(connection.id_a, connection.id_b);

            // TODO: Implement transfer of `LocalResources` between cell_a and cell_b
            // based on concentration gradients, diffusion, or control logic.
        }
    }
}
