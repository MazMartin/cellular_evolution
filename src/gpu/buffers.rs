use crate::gpu::context::GpuContext;
use wgpu::{BindGroup, BindGroupLayout, ShaderStages};
use std::mem::size_of;

/// A typed wrapper around a `wgpu::Buffer`, used for storage or uniform buffers.
pub struct GpuBuffer<T> {
    /// Human-readable label used in debugging.
    pub label: &'static str,

    /// The raw GPU buffer.
    pub buffer: wgpu::Buffer,

    /// The usage flags for the buffer (e.g., storage, uniform).
    pub usage: wgpu::BufferUsages,

    /// Number of elements of type `T` the buffer was allocated for.
    pub len: usize,

    /// Marker to retain type information without storing data.
    _marker: std::marker::PhantomData<T>,
}

/// Describes how a buffer will be used in a bind group.
#[derive(Clone, Copy)]
pub enum BufferKind {
    /// Storage buffer, optionally marked as read-only.
    Storage { read_only: bool },

    /// Uniform buffer.
    Uniform,
}

/// Specifies shader stage visibility and buffer type for binding.
#[derive(Clone, Copy)]
pub struct BindInfo {
    pub visibility: ShaderStages,
    pub kind: BufferKind,
}

impl GpuContext {
    /// Creates a new GPU buffer of type `T` with the given usage, label, and length.
    pub fn create_buffer<T>(
        &self,
        usage: wgpu::BufferUsages,
        label: &'static str,
        len: usize,
    ) -> GpuBuffer<T> {
        let size = (size_of::<T>() * len) as wgpu::BufferAddress;

        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("{label} - Buffer")),
            size,
            usage,
            mapped_at_creation: false,
        });

        GpuBuffer {
            label,
            buffer,
            usage,
            len,
            _marker: std::marker::PhantomData,
        }
    }

    /// Creates a `BindGroupLayout` and `BindGroup` from a list of buffers and their `BindInfo`.
    pub fn create_bind_data(
        &self,
        bindings: &[(&wgpu::Buffer, BindInfo)],
    ) -> (BindGroupLayout, BindGroup) {
        let layout_entries: Vec<_> = bindings
            .iter()
            .enumerate()
            .map(|(i, (_, info))| wgpu::BindGroupLayoutEntry {
                binding: i as u32,
                visibility: info.visibility,
                ty: match info.kind {
                    BufferKind::Storage { read_only } => wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    BufferKind::Uniform => wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                },
                count: None,
            })
            .collect();

        let layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("auto-layout"),
            entries: &layout_entries,
        });

        let group_entries: Vec<_> = bindings
            .iter()
            .enumerate()
            .map(|(i, (buffer, _))| wgpu::BindGroupEntry {
                binding: i as u32,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer,
                    offset: 0,
                    size: None,
                }),
            })
            .collect();

        let group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("auto-group"),
            layout: &layout,
            entries: &group_entries,
        });

        (layout, group)
    }
}

impl<T: bytemuck::Pod> GpuBuffer<T> {
    /// Creates a bind group for this buffer using an existing layout and binding index.
    pub fn create_bind_group(
        &self,
        device: &wgpu::Device,
        layout: &BindGroupLayout,
        binding: u32,
    ) -> BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding,
                resource: self.buffer.as_entire_binding(),
            }],
            label: Some(&format!("{} - Bind Group", self.label)),
        })
    }

    /// Writes a single value of type `T` into the GPU buffer.
    /// Panics if the buffer was created for more than one element.
    pub fn write(&self, queue: &wgpu::Queue, data: &T) {
        debug_assert!(
            self.len == 1,
            "Calling write() on a buffer sized for more than one element"
        );
        let bytes = bytemuck::bytes_of(data);
        queue.write_buffer(&self.buffer, 0, bytes);
    }

    /// Writes a slice of `T` into the GPU buffer.
    /// Panics if the data length exceeds the allocated buffer size.
    pub fn write_array(&self, queue: &wgpu::Queue, data: &[T]) {
        assert!(
            data.len() <= self.len,
            "write_array: data length ({}) exceeds buffer capacity ({})",
            data.len(),
            self.len
        );
        let bytes = bytemuck::cast_slice(data);
        queue.write_buffer(&self.buffer, 0, bytes);
    }
}
