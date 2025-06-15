use crate::core::sim::{SimulationState};
use std::sync::{Arc, Mutex};
use taffy::NodeId;

pub struct Simulation {
    pub state: Arc<Mutex<SimulationState>>,
    pub tile: Option<NodeId>,
}