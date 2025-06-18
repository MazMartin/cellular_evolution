use crate::gpu::context::GpuContext;
use glam::Vec2;
use std::sync::{Arc, Mutex};
use wgpu::RenderPass;
use crate::core::sim::SimulationState;

/// Holds the data needed to render a single frame,
/// including the texture to draw to, command encoder, and view.
pub struct FrameContext {
    pub surface_texture: wgpu::SurfaceTexture,
    pub encoder: wgpu::CommandEncoder,
    pub view: wgpu::TextureView,
}

impl FrameContext {
    /// Starts a render pass that clears the frame to black.
    pub fn begin_render_pass(&mut self) -> RenderPass {
        self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        })
    }
}

impl GpuContext {
    /// Prepares GPU for a new frame by acquiring the next texture and creating a command encoder.
    pub fn start_frame(&mut self) -> FrameContext {
        let surface_texture = self
            .surface
            .get_current_texture()
            .expect("failed to acquire next swapchain texture");
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                // Use sRGB format to ensure correct gamma.
                format: Some(self.surface_format.add_srgb_suffix()),
                ..Default::default()
            });

        let encoder = self.device.create_command_encoder(&Default::default());

        FrameContext {
            surface_texture,
            encoder,
            view: texture_view,
        }
    }

    /// Submits the recorded commands and presents the frame.
    pub fn end_frame(&mut self, frame: FrameContext) {
        self.queue.submit(std::iter::once(frame.encoder.finish()));
        self.window.pre_present_notify();
        frame.surface_texture.present();
    }
}

/// Interface for rendering tiles.
///
/// Responsible for initialization, resizing, updating GPU data,
/// and issuing draw calls during render passes.
pub trait TileRenderer {
    /// Called once to initialize the renderer.
    fn init(&self, queue: &wgpu::Queue);
    
    /// Called when the viewport or target size changes
    fn resize(&mut self, size: Vec2, queue: &wgpu::Queue);
    
    /// Updates render data based on simulation state.
    fn update_render_data(&mut self, state: Arc<Mutex<SimulationState>>, queue: &wgpu::Queue);

    /// Encodes commands to render on the render pass.
    fn render_pipeline<'a>(&'a self, render_pass: &mut RenderPass<'a>);
}
