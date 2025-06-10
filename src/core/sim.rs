use super::elements::{Cell, CellConnection, CellId};
use crate::utils::data::Heap;
pub struct SimContext {
    pub viscosity: f64,
}

pub struct AppContext {
    pub context: SimContext,
    pub cells: Heap<Cell>,
    pub connections: Vec<CellConnection>,
}

impl AppContext {
    pub fn new(context: SimContext) -> AppContext {
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
}