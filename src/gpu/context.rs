use std::sync::Arc;
use winit::window::Window;

/// Encapsulates all GPU-related state and functionality using wgpu.
pub(crate) struct GpuContext {
    /// Reference-counted window handle, ensuring proper lifetime management.
    pub window: Arc<Window>,

    /// Logical device interface for interacting with the GPU.
    pub device: wgpu::Device,

    /// Command queue for submitting GPU commands asynchronously.
    pub queue: wgpu::Queue,

    /// Physical size of the window in pixels.
    pub size: winit::dpi::PhysicalSize<u32>,

    /// Surface (swap chain) representing the drawable render target.
    pub surface: wgpu::Surface<'static>,

    /// Format of the textures presented by the surface.
    pub surface_format: wgpu::TextureFormat,
}

impl GpuContext {
    /// Asynchronously creates a new `GpuContext` bound to the given window.
    pub(crate) async fn new(window: Arc<Window>) -> GpuContext {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

        // Request an appropriate adapter (physical GPU).
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("Failed to find a GPU adapter");

        // Request a logical device and command queue from the adapter.
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .expect("Failed to create device and queue");

        let size = window.inner_size();

        // Create the rendering surface linked to the window.
        let surface = instance.create_surface(window.clone())
            .expect("Failed to create surface");

        // Query supported surface formats and pick the first.
        let caps = surface.get_capabilities(&adapter);
        let surface_format = caps.formats[0];

        let context = GpuContext {
            window,
            device,
            queue,
            size,
            surface,
            surface_format,
        };

        // Initial surface configuration.
        context.configure_surface();

        context
    }

    /// Returns a reference to the associated window.
    pub(crate) fn get_window(&self) -> &Window {
        &self.window
    }

    /// Configures the surface with the current size and format.
    fn configure_surface(&self) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            view_formats: vec![self.surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.size.width,
            height: self.size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        self.surface.configure(&self.device, &surface_config);
    }

    /// Handles window resizing by updating the stored size and reconfiguring the surface.
    pub(crate) fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.configure_surface();
    }

    /// Writes a slice of `Pod` data into the given GPU buffer.
    pub fn write_slice_buffer<T: bytemuck::Pod>(&self, buffer: &wgpu::Buffer, data: &[T]) {
        self.queue.write_buffer(buffer, 0, bytemuck::cast_slice(data));
    }
}
