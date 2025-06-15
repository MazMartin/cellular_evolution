use super::loaders::EnvironmentRenderLoader;
use super::models::{gpu::*, space::*};
use super::renderer::TileRenderer;
use crate::core::sim::SimulationState;
use crate::gpu::buffers::{BindInfo, BufferKind, GpuBuffer};
use crate::gpu::context::GpuContext;
use glam::{Vec2, vec2};
use std::sync::{Arc, Mutex};
use crate::combine_code;

pub struct SimulationTile {
    worldspace: AABB,
    camera: SrtTransform,

    pipeline: wgpu::RenderPipeline, // Defines the shader pipeline

    loader: EnvironmentRenderLoader,

    // Buffers
    vert_buff: GpuBuffer<GpuVertex>,
    render_instance_buff: GpuBuffer<GpuQuadRenderInstance>,
    primitive_index_buff: GpuBuffer<GpuPrimitiveIndex>,
    primitive_buff: GpuBuffer<GpuPrimitive>,
    projection_buff: GpuBuffer<[[f32; 4]; 4]>,

    // Buffer Data
    instance_count: u32,

    // Buffer Binds
    cell_data_bind: wgpu::BindGroup,
    projection_bind: wgpu::BindGroup,
}

impl SimulationTile {
    pub(crate) fn new(size: Vec2, context: &GpuContext) -> Self {
        let worldspace = AABB::from_wh(size);
        
        let shader = context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Environment Shader"),
            source: wgpu::ShaderSource::Wgsl(combine_code!(
            "../shaders/primitive_ren.wgsl",
            "../shaders/primitive_utils.wgsl").into()),
        });

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

        let render_pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&projection_layout, &cell_data_layout],
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_main"), // entry for the vertex shader
                        buffers: &[GpuVertex::desc(), GpuQuadRenderInstance::desc()],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_main"), // entry for the fragment shader
                        targets: &[Some(wgpu::ColorTargetState {
                            format: context.surface_format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL, // write rgba
                        })],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    }),

                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw, // 2.
                        cull_mode: Some(wgpu::Face::Back),
                        // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                        polygon_mode: wgpu::PolygonMode::Fill,
                        // Requires Features::DEPTH_CLIP_CONTROL
                        unclipped_depth: false,
                        // Requires Features::CONSERVATIVE_RASTERIZATION
                        conservative: false,
                    },

                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,                         // no MSAA (multi-sample anti-aliasing)
                        mask: !0,                         // enables all samples
                        alpha_to_coverage_enabled: false, // disabled (used for transparency masking)
                    },
                    multiview: None,
                    cache: None,
                });

        Self {
            worldspace,
            camera: SrtTransform::default(),

            pipeline: render_pipeline, // Defines the shader pipeline

            loader: EnvironmentRenderLoader::new(),

            // Buffers
            vert_buff,
            render_instance_buff,
            primitive_index_buff,
            primitive_buff,
            projection_buff,

            // Buffer Data
            instance_count: 0,

            // Buffer Binds
            cell_data_bind,
            projection_bind,
        }
    }
}

impl TileRenderer for SimulationTile {
    fn init(&self, queue: &wgpu::Queue) {
        self.vert_buff
            .write_array(&queue, &AABB::UNIT.corners().ccw_mesh());
        self.projection_buff
            .write(&queue, &mat4_to_gpu_mat(self.camera.to_mat4().inverse()))
    }
    fn resize(&mut self, size: Vec2, queue: &wgpu::Queue) {
        let aspect = size.x / size.y;
        let zoom = 10.0;
        let center = vec2(0., 0.);

        self.camera = SrtTransform {
            translate: center,
            rotate: 0.0,
            scale: vec2(zoom, zoom / aspect),
        };

        self.projection_buff
            .write(&queue, &mat4_to_gpu_mat(self.camera.to_mat4().inverse()))
    }

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

    fn render_pipeline(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.projection_bind, &[]);
        render_pass.set_bind_group(1, &self.cell_data_bind, &[]);

        render_pass.set_vertex_buffer(0, self.vert_buff.buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.render_instance_buff.buffer.slice(..));

        render_pass.draw(0..6, 0..self.instance_count);
    }
}
