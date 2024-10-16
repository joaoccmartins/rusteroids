use std::ops::Deref;

use wgpu::util::DeviceExt;

use crate::mesh::Vertex;
use wgpu_utils::Bindable;

/// A bunch of boilerplate code and meshes for now

/// The battleship mesh
pub const WEDGE: &[Vertex] = &[
    Vertex {
        position: [0.0, 20.0],
        color: [1.0, 1.0, 1.0],
    },
    Vertex {
        position: [10.0, -20.0],
        color: [1.0, 1.0, 1.0],
    },
    Vertex {
        position: [0.0, -10.0],
        color: [1.0, 1.0, 1.0],
    },
    Vertex {
        position: [-10.0, -20.0],
        color: [1.0, 1.0, 1.0],
    },
    Vertex {
        position: [0.0, 20.0],
        color: [1.0, 1.0, 1.0],
    },
];

// Create a buffer to be used as a uniform with a bind group
pub fn create_buffer<T>(data: &T, device: &wgpu::Device, label: &str) -> wgpu::Buffer
where
    T: bytemuck::Pod + bytemuck::Zeroable,
{
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        contents: bytemuck::bytes_of(data),
    })
}

pub fn create_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    buffer: &wgpu::Buffer,
    label: &str,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
        label: Some(label),
    })
}

/// A struct to manage both a uniform buffer and its bind group in WebGPU.
/// Directly relates to UniformBinding as the Layout needs to be managed outside of it.
pub struct UniformBuffer {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl UniformBuffer {
    /// Creates a new UniformBuffer struct. Device and Layout already needs to exist.
    pub fn new<T>(
        data: &T,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        label_prefix: &str,
    ) -> Self
    where
        T: bytemuck::Pod + bytemuck::Zeroable,
    {
        let buffer = create_buffer(data, device, &format!("{}_buffer", label_prefix));
        let bind_group = create_bind_group(
            device,
            layout,
            &buffer,
            &format!("{}_bind_group", label_prefix),
        );
        Self { buffer, bind_group }
    }

    /// Updates the buffer content through a pre existing Queue
    pub fn update_buffer<T>(&self, data: &T, queue: &wgpu::Queue)
    where
        T: bytemuck::Pod + bytemuck::Zeroable,
    {
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(data));
    }

    /// Binds self to a particular group in the RenderPipeline
    pub fn bind(&self, pass: &mut wgpu::RenderPass, group_index: u32) {
        pass.set_bind_group(group_index, &self.bind_group, &[]);
    }
}

/// Struct to manage the binding of Uniforms to a RenderPipeline according to Bindable type.
/// Only enables the creation of the RenderPipeline with a specific BindGroup Layout. Buffer
/// is managed via UniformBuffer.
pub struct UniformBinding {
    layout: wgpu::BindGroupLayout,
}

impl UniformBinding {
    pub fn new<T>(device: &wgpu::Device) -> Self
    where
        T: Bindable,
    {
        Self {
            layout: device.create_bind_group_layout(&T::desc()),
        }
    }
}

impl Deref for UniformBinding {
    type Target = wgpu::BindGroupLayout;

    fn deref(&self) -> &Self::Target {
        &self.layout
    }
}
