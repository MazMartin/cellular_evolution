use crate::core::sim::{SimulationState};
use std::sync::{Arc, Mutex};
use wgpu;

pub struct Simulation {
    pub state: Arc<Mutex<SimulationState>>,
}

pub struct RenderSystem {
    pub queue: Arc<wgpu::Queue>,
}

pub struct PhysicsSystem {
    pub dt: f64,
}