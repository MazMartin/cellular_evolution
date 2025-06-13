use std::ops::Sub;
use crate::core::sim::SimulationState;

type Energy = f32;
type Fat = f32;
pub struct LocalResources {
    energy: Energy,
    fat: Fat,
}

impl Sub for LocalResources {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            energy: self.energy - rhs.energy,
            fat: self.fat - rhs.fat,
        }
    }
}

impl SimulationState {
    pub fn share_resources_pass(&mut self, dt: f64) {
        for connection in self.connections.iter() {
            let (cell_a, cell_b) = self.cells.get_mut_pair(connection.id_a, connection.id_b);
        }
    }
}