use crate::core::sim::{SimContext};
use crate::graphics::border::BorderTile;
use crate::graphics::layers::SimulationTile;
use crate::app::tile::TileViewManager;
use crate::testing::benches;

use glam::{vec2, Vec2};
use std::sync::{Arc, Mutex};
use taffy::{Dimension, Size, Style};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop},
    window::{Window, WindowId},
};

use hecs::World;
use crate::gpu;

mod components {
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
}

pub struct App {
    gpu_context: Option<gpu::context::GpuContext>,
    world: World,
    tiles: TileViewManager,
    env_nodes: Vec<taffy::NodeId>, // Changed to support multiple environments
}

impl App {
    const FPS: f32 = 60.;
    pub fn new() -> Self {
        let mut world = World::new();
        let mut tiles = TileViewManager::new();
        
        let sim_context1 = SimContext { viscosity: 25.0 };
        let sim_context2 = SimContext { viscosity: 1.0 };

        let simulation_state1 = Arc::new(Mutex::new(benches::organism_lookn_cells(sim_context1)));
        let simulation_state2 = Arc::new(Mutex::new(benches::organism_lookn_cells(sim_context2)));

        world.spawn((
            components::Simulation {
                state: simulation_state1,
            },
            components::PhysicsSystem {
                dt: 1.0 / Self::FPS as f64,
            },
        ));

        world.spawn((
            components::Simulation {
                state: simulation_state2,
            },
            components::PhysicsSystem {
                dt: 1.0 / Self::FPS as f64,
            },
        ));


        let mut env_nodes = Vec::new();

        for _ in 0..2 {
            let style = Style {
                size: Size {
                    width: Dimension::percent(0.5),
                    height: Dimension::auto(),
                },
                aspect_ratio: Some(16.0 / 9.0),
                ..Default::default()
            };
            let node = tiles.add_leaf(tiles.root(), style);
            env_nodes.push(node);
        }

        Self {
            gpu_context: None,
            world,
            tiles,
            env_nodes,
        }
    }

    fn init_gpu(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
        let gpu_context = pollster::block_on(gpu::context::GpuContext::new(window.clone()));

        self.world.spawn((components::RenderSystem {
            queue: Arc::new(gpu_context.queue.clone()),
        },));

        self.tiles.resize(Vec2::new(
            gpu_context.size.width as f32,
            gpu_context.size.height as f32,
        ));
        
        for node in &self.env_nodes {
            let env = SimulationTile::new(Vec2::new(15.0, 10.0), &gpu_context);
            env.init_buffers(&gpu_context.queue);

            self.tiles.add_renderer(*node, env);
            self.tiles.add_renderer(*node, BorderTile::new(&gpu_context));
        }

        self.gpu_context = Some(gpu_context);
        window.request_redraw();
    }

    fn update_and_render(&mut self) {
        if let Some(gpu_ctx) = &mut self.gpu_context {

            for (_, (sim, physics)) in self.world.query::<(&mut components::Simulation, &components::PhysicsSystem)>().iter() {
                sim.state.lock().unwrap().physics_pass(physics.dt);
            }


            let simulation_states: Vec<_> = self.world
                .query::<&components::Simulation>()
                .iter()
                .map(|(_, sim)| sim.state.clone())
                .collect();


            for (node, state) in self.env_nodes.iter().zip(simulation_states.iter()) {
                self.tiles.load_all(state.clone(), &gpu_ctx.queue);
            }


            let mut frame = gpu_ctx.start_frame();
            {
                let mut render_pass = frame.begin_render_pass();
                self.tiles.render_all(&mut render_pass);
            }
            gpu_ctx.end_frame(frame);

            gpu_ctx.get_window().request_redraw();
        }
    }


    fn handle_resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        if let Some(gpu_ctx) = &mut self.gpu_context {
            gpu_ctx.resize(size);
            self.tiles.resize(vec2(gpu_ctx.size.width as f32, gpu_ctx.size.height as f32));
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.init_gpu(event_loop);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping application.");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.update_and_render();
            }
            WindowEvent::Resized(size) => {
                self.handle_resize(size);
            }
            _ => {}
        }
    }

    fn suspended(&mut self, _: &ActiveEventLoop) {}
}