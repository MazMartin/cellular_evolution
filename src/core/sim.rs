use super::elements::{Cell, CellConnection, CellId};
use crate::utils::data::Heap;
pub struct SimContext {
    pub viscosity: f64,
}

pub struct SimulationState {
    pub context: SimContext,
    pub cells: Heap<Cell>,
    pub connections: Vec<CellConnection>,
}

impl SimulationState {
    pub fn new(context: SimContext) -> SimulationState {
        Self {
            context,
            cells: Heap::with_capacity(100),
            connections: Vec::with_capacity(100),
        }
    }

    pub fn remove(&mut self, id: CellId) {
        self.cells.free(id);

        let mut i = self.connections.len();
        while i > 0 {
            i -= 1;
            if self.connections[i].points_toward(id) {
                self.connections.swap_remove(i);
            }
        }
    }
    
    pub fn tick(&mut self, dt: f64) {
        self.physics_pass(dt);
        
    }
}