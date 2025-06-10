use std::sync::Arc;
use winit::window::Window;
pub(crate) struct GpuContext {
    // A structure defining everything needed to operate the GPU via WGPU
    pub window: Arc<Window>, // The Arc type will track all references and only destroy when all references exit
    pub device: wgpu::Device, // Represents a logical device that can interact with the GPU
    pub queue: wgpu::Queue,  // Lets the CPU queue asynchronous commands to the GPU
    pub size: winit::dpi::PhysicalSize<u32>, // Stores the window's size in physical pixels.
    pub surface: wgpu::Surface<'static>, // Represents the swap chain surface where the final rendered image is presented
    pub surface_format: wgpu::TextureFormat, // Defines the format (color encoding and bit depth) of the surface’s textures
}

impl GpuContext {
    pub(crate) async fn new(window: Arc<Window>) -> GpuContext {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .unwrap();

        let size = window.inner_size();

        let surface = instance.create_surface(window.clone()).unwrap();
        let cap = surface.get_capabilities(&adapter);
        let surface_format = cap.formats[0];

        let state = GpuContext {
            window,
            device,
            queue,
            size,
            surface,
            surface_format,
        };

        // Configure surface for the first time
        state.configure_surface();

        state
    }

    pub(crate) fn get_window(&self) -> &Window {
        &self.window
    }

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

    pub(crate) fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.configure_surface();
    }

    pub fn write_slice_buffer<T: bytemuck::Pod>(&self, buffer: &wgpu::Buffer, data: &[T]) {
        self.queue
            .write_buffer(&buffer, 0, bytemuck::cast_slice(&data));
    }
}
