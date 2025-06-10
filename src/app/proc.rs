use std::sync::{Arc, Mutex};
use crate::graphics::renderer::TileRenderer;

type AState = f32;

enum ProcMessage {
    SpawnTile(Box<dyn TileRenderer>),
}

trait Process {
    fn poll(&mut self, messages: &mut Vec<ProcMessage>);
}

struct AProcess {
    state_pointer: Arc<Mutex<AState>>,
}

impl AProcess {
    fn new() -> Self {
        Self {
            state_pointer: Arc::new(Mutex::new(0.0)),
        }
    }
}

impl Process for AProcess {
    fn poll(&mut self, messages: &mut Vec<ProcMessage>) {
        messages.push(ProcMessage::SpawnTile(Box::new(ATileRenderer {
            state_pointer: Arc::clone(&self.state_pointer),
        })));
    }
}

struct ATileRenderer {
    state_pointer: Arc<Mutex<AState>>,
}

impl TileRenderer for ATileRenderer {
    // implement required methods here
}
