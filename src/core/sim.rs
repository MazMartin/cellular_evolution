use super::elements::{Cell, CellConnection, CellId};
use crate::utils::data::Heap;

/// Stores global simulation parameters.
pub struct SimContext {
    pub viscosity: f64,
}

/// Represents the state of the simulation, including all cells and their connections.
pub struct SimulationState {
    pub context: SimContext,
    pub cells: Heap<Cell>,
    pub connections: Vec<CellConnection>,
}

impl SimulationState {
    /// Creates a new simulation state with the given context and initial capacities.
    pub fn new(context: SimContext) -> Self {
        Self {
            context,
            cells: Heap::with_capacity(100),
            connections: Vec::with_capacity(100),
        }
    }

    /// Removes a cell from the simulation by its ID.
    /// Also removes all connections that include the removed cell.
    pub fn remove(&mut self, id: CellId) {
        self.cells.free(id);

        // Efficiently remove all connections pointing to the removed cell.
        let mut i = self.connections.len();
        while i > 0 {
            i -= 1;
            if self.connections[i].points_toward(id) {
                self.connections.swap_remove(i);
            }
        }
    }

    /// Advances the simulation state by a single time step `dt`.
    pub fn tick(&mut self, dt: f64) {
        self.physics_pass(dt);
        // Future passes like `share_resources_pass(dt)` can be added here.
    }
}
