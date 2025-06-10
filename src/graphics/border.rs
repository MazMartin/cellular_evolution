use std::sync::{Arc, Mutex};
use crate::combine_code;
use crate::gpu::buffers::{BindInfo, BufferKind, GpuBuffer};
use crate::gpu::context::GpuContext;
use super::models::{gpu::*, space::*};
use super::renderer::TileRenderer;

use glam::Vec2;
use wgpu::{BindGroup, ShaderStages};
use crate::core::sim::SimulationState;

pub struct BorderTile {
    pipeline: wgpu::RenderPipeline,
    vert_buff: GpuBuffer<GpuVertex>,
    info_buff: GpuBuffer<BorderInfoUniform>,
    info_bind: BindGroup,
}

impl BorderTile {
    pub fn new(context: &GpuContext) -> Self {
        let shader = context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Border Shader"),
            source: wgpu::ShaderSource::Wgsl(combine_code!(
                "../shaders/border.wgsl"
            ).into()),
        });

        let vert_buff = context.create_buffer(
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            "Border Vertices",
            24,
        );

        let info_buff = context.create_buffer::<BorderInfoUniform>(
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            "Border Info",
            1,
        );

        let (info_layout, info_bind) = context.create_bind_data(&[(
            &info_buff.buffer,
            BindInfo {
                visibility: ShaderStages::VERTEX_FRAGMENT,
                kind: BufferKind::Uniform,
            },
        )]);

        let pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Border Pipeline Layout"),
            bind_group_layouts: &[&info_layout],
            push_constant_ranges: &[],
        });

        let pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Border Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[GpuVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
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
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self { pipeline, vert_buff, info_buff, info_bind }
    }
    
    fn generate_border_mesh(aabb: AABB, width: f32) -> [GpuVertex; 24] {
        let inner = aabb.add_padding(-width).corners();
        let outer = aabb.add_padding(0.0).corners();

        let v = |pos: Vec2| GpuVertex::new(pos);

        [
            // Top
            v(outer.tl), v(outer.tr), v(inner.tr),
            v(inner.tr), v(inner.tl), v(outer.tl),

            // Right
            v(outer.tr), v(outer.br), v(inner.br),
            v(inner.br), v(inner.tr), v(outer.tr),

            // Bottom
            v(outer.br), v(outer.bl), v(inner.bl),
            v(inner.bl), v(inner.br), v(outer.br),

            // Left
            v(outer.bl), v(outer.tl), v(inner.tl),
            v(inner.tl), v(inner.bl), v(outer.bl),
        ]
    }
}

impl TileRenderer for BorderTile {
    fn resize(&mut self, size: Vec2, queue: &wgpu::Queue) {
        let aabb = AABB::new(Vec2::ZERO, size * 0.5);
        let vertices = Self::generate_border_mesh(aabb, 20.0);
        self.vert_buff.write_array(queue, &vertices);
        self.info_buff.write(queue, &BorderInfoUniform::new(size, 20.0));
    }

    fn update_render_data(&mut self, _: Arc<Mutex<SimulationState>>, _: &wgpu::Queue) {
        // Border doesn't need state updates
    }

    fn render_pipeline(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.info_bind, &[]);
        render_pass.set_vertex_buffer(0, self.vert_buff.buffer.slice(..));
        render_pass.draw(0..24, 0..1);
    }
}
