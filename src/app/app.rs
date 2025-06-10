use crate::core::sim::{SimContext, SimulationState};
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
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use hecs::World;
use crate::gpu;

mod components {
    use crate::core::sim::{SimContext, SimulationState};
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
    env_node: taffy::NodeId,
}


impl App {
    pub const FPS: u32 = 60;

    pub(crate) fn new() -> Self {
        let mut world = World::new();

        let simulation_context = SimContext { viscosity: 25.0 };
        let simulation_state = Arc::new(Mutex::new(benches::organism_lookn_cells(
            simulation_context,
        )));

        world.spawn((
            components::Simulation {
                state: simulation_state,
            },
            components::PhysicsSystem {
                dt: 1.0 / Self::FPS as f64,
            },
        ));

        let mut tiles = TileViewManager::new();
        let style = Style {
            size: Size {
                width: Dimension::percent(1.0),
                height: Dimension::auto(),
            },
            aspect_ratio: Some(16.0 / 9.0),
            ..Default::default()
        };
        let environment_node = tiles.add_leaf(tiles.root(), style);

        Self {
            gpu_context: None,
            world,
            tiles,
            env_node: environment_node,
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

        let env = SimulationTile::new(Vec2::new(15.0, 10.0), &gpu_context);
        env.init_buffers(&gpu_context.queue);

        self.tiles.add_renderer(self.env_node, env);
        self.tiles.add_renderer(self.env_node, BorderTile::new(&gpu_context));

        self.gpu_context = Some(gpu_context);
        window.request_redraw();
    }

    fn update_and_render(&mut self) {
        if let Some(gpu_ctx) = &mut self.gpu_context {

            for (_, (sim, physics)) in self.world.query::<(&mut components::Simulation, &components::PhysicsSystem)>().iter() {
                sim.state.lock().unwrap().physics_pass(physics.dt);
            }

            let simulation_state = self.world
                .query::<&components::Simulation>()
                .iter()
                .next()
                .map(|(_, sim)| sim.state.clone());

            if let Some(state) = simulation_state {
                self.tiles.load_all(state, &gpu_ctx.queue);

                let mut frame = gpu_ctx.start_frame();
                {
                    let mut render_pass = frame.begin_render_pass();
                    self.tiles.render_all(&mut render_pass);
                }
                gpu_ctx.end_frame(frame);
            }

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