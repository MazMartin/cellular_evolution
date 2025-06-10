use super::cpu::Primitive;
use glam::{Mat4, Vec2};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GpuVertex([f32; 2]);

unsafe impl bytemuck::Pod for GpuVertex {}
unsafe impl bytemuck::Zeroable for GpuVertex {}

impl GpuVertex {
    pub fn new(Vec2 { x, y }: Vec2) -> Self {
        Self([x, y])
    }

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<GpuVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array!(0 => Float32x2),
        }
    }
}

impl From<Vec2> for GpuVertex {
    fn from(vec: Vec2) -> Self {
        Self([vec.x, vec.y])
    }
}

pub fn mat4_to_gpu_mat(mat: Mat4) -> [[f32; 4]; 4] {
    mat.to_cols_array_2d()
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GpuQuadRenderInstance {
    pub aabb_center: [f32; 2],
    pub aabb_half: [f32; 2],
    pub start_i: u32,
    pub end_i: u32,
}

unsafe impl bytemuck::Pod for GpuQuadRenderInstance {}
unsafe impl bytemuck::Zeroable for GpuQuadRenderInstance {}

impl GpuQuadRenderInstance {
    const ATTRIBS: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
            5 => Float32x2, 6 => Float32x2,
            7 => Uint32, 8 => Uint32];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<GpuQuadRenderInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct GpuPrimitive {
    unit_projection: [[f32; 4]; 4],
    color: [f32; 4],
    shape: u32,
    _padding: [u32; 3],
}

unsafe impl bytemuck::Pod for GpuPrimitive {}
unsafe impl bytemuck::Zeroable for GpuPrimitive {}

impl From<Primitive> for GpuPrimitive {
    fn from(p: Primitive) -> Self {
        let transform = p.transform;
        let color = [
            p.color.r as f32 / 255.0,
            p.color.g as f32 / 255.0,
            p.color.b as f32 / 255.0,
            p.color.a as f32 / 255.0,
        ];
        let shape = p.shape as u32;

        GpuPrimitive {
            unit_projection: mat4_to_gpu_mat(transform.to_mat4().inverse()),
            color,
            shape,
            _padding: [0, 0, 0],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GpuPrimitiveIndex {
    pub(crate) index: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
}

unsafe impl bytemuck::Pod for GpuPrimitiveIndex {}
unsafe impl bytemuck::Zeroable for GpuPrimitiveIndex {}

impl From<usize> for GpuPrimitiveIndex {
    fn from(i: usize) -> Self {
        GpuPrimitiveIndex {
            index: i as u32,
            _pad0: 0,
            _pad1: 0,
            _pad2: 0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BorderInfoUniform {
    pub size: [f32; 2],
    pub width: f32,
    _pad: [f32; 1],
}

impl BorderInfoUniform {
    pub fn new(size: Vec2, width: f32) -> Self {
        Self {
            size: [size.x, size.y],
            width,
            _pad: [0.0],
        }
    }
}
