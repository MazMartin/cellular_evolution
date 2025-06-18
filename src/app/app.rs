use crate::core::sim::SimContext;
use crate::graphics::border::BorderTile;
use crate::graphics::layers::SimulationTile;
use crate::testing::benches;
use crate::app::components::Simulation;
use crate::gpu;

use super::tile::TileViewManager;

use glam::{vec2, Vec2};
use std::sync::{Arc, Mutex};
use taffy::{Dimension, Size, Style};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

/// Main application struct managing GPU, tile layout, and simulation state.
pub struct App {
    gpu_context: Option<gpu::context::GpuContext>,
    tile_manager: TileViewManager,
    primary_simulation: Simulation,
}

impl App {
    /// Target frames per second.
    const TARGET_FPS: f32 = 60.0;

    /// Creates a new instance of the application with default simulation and tile layout.
    pub fn new() -> Self {
        let mut tile_manager = TileViewManager::new();

        // Initialize simulation state with custom viscosity.
        let sim_context = SimContext { viscosity: 25.0 };
        let initial_state = Arc::new(Mutex::new(benches::organism_lookn_cells(sim_context)));

        // Define UI style for the main simulation tile.
        let style = Style {
            size: Size {
                width: Dimension::percent(0.8),
                height: Dimension::auto(),
            },
            aspect_ratio: Some(16.0 / 9.0),
            ..Default::default()
        };

        let sim_tile_node = tile_manager.add_leaf(tile_manager.root(), style);

        Self {
            gpu_context: None,
            tile_manager,
            primary_simulation: Simulation {
                state: initial_state,
                tile: Some(sim_tile_node),
            },
        }
    }

    /// Initializes the GPU context and attaches renderers for the simulation.
    fn init_gpu(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .expect("Failed to create window"),
        );

        let gpu_context = pollster::block_on(gpu::context::GpuContext::new(window.clone()));

        self.tile_manager.resize(vec2(
            gpu_context.size.width as f32,
            gpu_context.size.height as f32,
        ));

        // Attach renderers to the simulation tile.
        if let Some(sim_tile_node) = self.primary_simulation.tile {
            self.tile_manager.add_renderer(
                sim_tile_node,
                SimulationTile::new(vec2(15.0, 10.0), &gpu_context),
                &gpu_context.queue,
            );
            self.tile_manager.add_renderer(
                sim_tile_node,
                BorderTile::new(&gpu_context),
                &gpu_context.queue,
            );
        }

        self.gpu_context = Some(gpu_context);
        window.request_redraw();
    }

    /// Updates the simulation and renders all tiles to the screen.
    fn update_and_render(&mut self) {
        // Advance the simulation.
        self.primary_simulation
            .state
            .lock()
            .unwrap()
            .tick((1.0 / Self::TARGET_FPS) as f64);

        // If GPU is available, load data and render.
        if let Some(gpu_context) = &mut self.gpu_context {
            self.tile_manager
                .load_all(self.primary_simulation.state.clone(), &gpu_context.queue);

            let mut frame = gpu_context.start_frame();
            {
                let mut render_pass = frame.begin_render_pass();
                self.tile_manager.render_all(&mut render_pass);
            }
            gpu_context.end_frame(frame);

            gpu_context.get_window().request_redraw();
        }
    }

    /// Handles window resizing and updates the GPU and tile layout accordingly.
    fn handle_resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if let Some(gpu_context) = &mut self.gpu_context {
            gpu_context.resize(new_size);
            self.tile_manager.resize(vec2(
                gpu_context.size.width as f32,
                gpu_context.size.height as f32,
            ));
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.init_gpu(event_loop);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Close requested. Exiting application.");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.update_and_render();
            }
            WindowEvent::Resized(new_size) => {
                self.handle_resize(new_size);
            }
            _ => {}
        }
    }

    fn suspended(&mut self, _: &ActiveEventLoop) {
        // Currently no action taken on suspend.
    }
}
