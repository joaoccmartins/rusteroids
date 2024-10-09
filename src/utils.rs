use wgpu::util::DeviceExt;

use crate::mesh::Vertex;

/// A bunch of boilerplate code and meshes for now

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

pub fn uniform_layout_descriptor(label: &str) -> wgpu::BindGroupLayoutDescriptor {
    wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: Some(label),
    }
}

// Create a buffer to be used as a uniform with a bind group
pub fn create_buffer<T>(data: T, device: &wgpu::Device, label: &str) -> wgpu::Buffer
where
    T: bytemuck::Pod + bytemuck::Zeroable,
{
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        contents: bytemuck::bytes_of(&data),
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
