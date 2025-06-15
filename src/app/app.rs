use crate::core::sim::{SimContext};
use crate::graphics::border::BorderTile;
use crate::graphics::layers::SimulationTile;
use super::tile::TileViewManager;
use crate::testing::benches;
use super::components;

use glam::{vec2, Vec2};
use std::sync::{Arc, Mutex};
use taffy::{Dimension, Size, Style};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop},
    window::{Window, WindowId},
};
use crate::app::components::Simulation;
use crate::gpu;

pub struct App {
    gpu_context: Option<gpu::context::GpuContext>,
    tiles: TileViewManager,
    main_sim: Simulation,
}

impl App {
    const FPS: f32 = 60.;
    pub fn new() -> Self {
        let mut tiles = TileViewManager::new();
        let sim_context = SimContext { viscosity: 25.0 };
        let simulation_state = Arc::new(Mutex::new(benches::organism_lookn_cells(sim_context)));
        
        let style = Style {
            size: Size {
                width: Dimension::percent(0.8),
                height: Dimension::auto(),
            },
            aspect_ratio: Some(16.0 / 9.0),
            ..Default::default()
        };
        
        let node = tiles.add_leaf(tiles.root(), style);

        Self {
            gpu_context: None,
            tiles,
            main_sim: Simulation {
                state: simulation_state,
                tile: Some(node),
            }
        }
    }

    fn init_gpu(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
        let gpu_context = pollster::block_on(gpu::context::GpuContext::new(window.clone()));

        self.tiles.resize(Vec2::new(
            gpu_context.size.width as f32,
            gpu_context.size.height as f32,
        ));
        
        self.tiles.add_renderer(self.main_sim.tile.unwrap(), SimulationTile::new(Vec2::new(15.0, 10.0), &gpu_context), &gpu_context.queue);
        self.tiles.add_renderer(self.main_sim.tile.unwrap(), BorderTile::new(&gpu_context), &gpu_context.queue);
        
        self.gpu_context = Some(gpu_context);
        window.request_redraw();
    }

    fn update_and_render(&mut self) {
        self.main_sim.state.lock().unwrap().tick((1.0 / Self::FPS) as f64);
        if let Some(gpu_context) = &mut self.gpu_context {
            self.tiles.load_all(self.main_sim.state.clone(), &gpu_context.queue);
            
            let mut frame = gpu_context.start_frame();
            {
                let mut render_pass = frame.begin_render_pass();
                self.tiles.render_all(&mut render_pass);
            }
            gpu_context.end_frame(frame);

            gpu_context.get_window().request_redraw();
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