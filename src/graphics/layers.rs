use super::loaders::EnvironmentRenderLoader;
use super::models::{gpu::*, space::*};
use super::renderer::TileRenderer;
use crate::core::sim::SimulationState;
use crate::gpu::buffers::{BindInfo, BufferKind, GpuBuffer};
use crate::gpu::context::GpuContext;
use glam::{Vec2, vec2};
use std::sync::{Arc, Mutex};
use crate::combine_code;

/// A tile responsible for rendering the simulation environment.
///
/// This struct manages GPU buffers and a pipeline for rendering primitives
/// that represent simulation entities. It tracks a world-space AABB,
/// a camera transform, and maintains buffers for vertex data, instance
/// data, primitive data, and uniform data.
///
/// The rendering pipeline uses WGSL shaders combined from multiple shader files,
/// and uses instanced rendering of quads to represent simulation objects.
pub struct SimulationTile {
    /// Axis-aligned bounding box defining the simulation world space for this tile.
    worldspace: AABB,

    /// Camera transform representing translation, rotation, and scale.
    camera: SrtTransform,

    /// The GPU render pipeline configured with shaders and fixed-function state.
    pipeline: wgpu::RenderPipeline,

    /// Loader responsible for preparing simulation data into GPU-friendly buffers.
    loader: EnvironmentRenderLoader,

    // GPU Buffers for vertex data, instances, primitives, and uniforms:
    vert_buff: GpuBuffer<GpuVertex>,
    render_instance_buff: GpuBuffer<GpuQuadRenderInstance>,
    primitive_index_buff: GpuBuffer<GpuPrimitiveIndex>,
    primitive_buff: GpuBuffer<GpuPrimitive>,
    projection_buff: GpuBuffer<[[f32; 4]; 4]>,

    /// Number of instances to render in the current frame.
    instance_count: u32,

    // Bind groups for uniform and storage buffers passed to shaders:
    cell_data_bind: wgpu::BindGroup,
    projection_bind: wgpu::BindGroup,
}

impl SimulationTile {
    /// Constructs a new `SimulationTile` with specified size and GPU context.
    ///
    /// This initializes all GPU buffers, compiles shaders, sets up pipeline layout,
    /// and prepares bind groups for uniform and storage buffers.
    pub(crate) fn new(size: Vec2, context: &GpuContext) -> Self {
        let worldspace = AABB::from_wh(size);

        let shader = context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Environment Shader"),
            source: wgpu::ShaderSource::Wgsl(combine_code!(
                "../shaders/primitive_ren.wgsl",
                "../shaders/primitive_utils.wgsl"
            ).into()),
        });

        // Create GPU buffers with usage flags appropriate for vertex, uniform, or storage data.
        let projection_buff = context.create_buffer(
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            "Projection Uniform",
            1,
        );
        let vert_buff = context.create_buffer(
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            "Unit Verts",
            6,
        );
        let render_instance_buff = context.create_buffer(
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            "Render Pack Instances",
            100,
        );

        let primitive_index_buff = context.create_buffer(
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            "Primitive Index Storage",
            100,
        );
        let primitive_buff = context.create_buffer(
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            "Primitive Storage",
            100,
        );

        // Create bind groups and layouts for uniform and storage buffers.
        let (projection_layout, projection_bind) = context.create_bind_data(&[(
            &projection_buff.buffer,
            BindInfo {
                visibility: wgpu::ShaderStages::VERTEX,
                kind: BufferKind::Uniform,
            },
        )]);

        let (cell_data_layout, cell_data_bind) = context.create_bind_data(&[
            (
                &primitive_index_buff.buffer,
                BindInfo {
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    kind: BufferKind::Storage { read_only: true },
                },
            ),
            (
                &primitive_buff.buffer,
                BindInfo {
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    kind: BufferKind::Storage { read_only: true },
                },
            ),
        ]);

        // Create the pipeline layout referencing the bind group layouts.
        let render_pipeline_layout =
            context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&projection_layout, &cell_data_layout],
                push_constant_ranges: &[],
            });

        // Create the render pipeline specifying shaders, vertex layouts, and rasterization state.
        let render_pipeline =
            context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"), // Vertex shader entry
                    buffers: &[GpuVertex::desc(), GpuQuadRenderInstance::desc()],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"), // Fragment shader entry
                    targets: &[Some(wgpu::ColorTargetState {
                        format: context.surface_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),

                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },

                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            });

        Self {
            worldspace,
            camera: SrtTransform::default(),

            pipeline: render_pipeline,

            loader: EnvironmentRenderLoader::new(),

            vert_buff,
            render_instance_buff,
            primitive_index_buff,
            primitive_buff,
            projection_buff,

            instance_count: 0,

            cell_data_bind,
            projection_bind,
        }
    }
}

impl TileRenderer for SimulationTile {
    /// Called once to initialize the renderer.
    fn init(&self, queue: &wgpu::Queue) {
        self.vert_buff
            .write_array(&queue, &AABB::UNIT.corners().ccw_mesh());
        self.projection_buff
            .write(&queue, &mat4_to_gpu_mat(self.camera.to_mat4().inverse()))
    }

    /// Called when the viewport or target size changes
    fn resize(&mut self, size: Vec2, queue: &wgpu::Queue) {
        let aspect = size.x / size.y;
        let zoom = 10.0;
        let center = vec2(0., 0.);

        // Update camera transform to keep aspect ratio and zoom
        self.camera = SrtTransform {
            translate: center,
            rotate: 0.0,
            scale: vec2(zoom, zoom / aspect),
        };

        // Upload updated projection matrix to uniform buffer
        self.projection_buff
            .write(&queue, &mat4_to_gpu_mat(self.camera.to_mat4().inverse()))
    }

    /// Updates render data based on simulation state.
    fn update_render_data(&mut self, state: Arc<Mutex<SimulationState>>, queue: &wgpu::Queue) {
        self.loader.run(state);

        self.instance_count = self.loader.gpu_render_instances.len() as u32;
        self.primitive_buff
            .write_array(&queue, &self.loader.gpu_primitives);
        self.primitive_index_buff
            .write_array(&queue, &self.loader.gpu_primitive_indices);
        self.render_instance_buff
            .write_array(&queue, &self.loader.gpu_render_instances);
    }

    /// Encodes commands to render on the render pass.
    fn render_pipeline(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.projection_bind, &[]);
        render_pass.set_bind_group(1, &self.cell_data_bind, &[]);

        render_pass.set_vertex_buffer(0, self.vert_buff.buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.render_instance_buff.buffer.slice(..));

        render_pass.draw(0..6, 0..self.instance_count);
    }
}
