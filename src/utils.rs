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

pub fn uniform_layout_descriptor<'a>(label: &'a str) -> wgpu::BindGroupLayoutDescriptor<'a> {
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
