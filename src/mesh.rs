use std::ops::Range;

use glam::Mat4;
use wgpu::util::DeviceExt;
use wgpu::{Queue, RenderPass};

use crate::utils::{common_layout_descriptor, Bindable, UniformBuffer};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x3];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct Geometry {
    vertex_buffer: Option<wgpu::Buffer>,
    model_uniform: Option<UniformBuffer>,
    geometry_size: usize,
}

impl Bindable for Geometry {
    fn layout_desc<'a>() -> wgpu::BindGroupLayoutDescriptor<'a> {
        common_layout_descriptor(Some("mat4_layout_descriptor"))
    }
}

impl Geometry {
    pub fn new(
        data: &[Vertex],
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        mesh_index: u32,
    ) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("vertex{}_buffer", mesh_index)),
            contents: bytemuck::cast_slice(data),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let model = Mat4::IDENTITY.to_cols_array();
        let model_uniform = UniformBuffer::new(
            &model,
            device,
            bind_group_layout,
            &format!("mesh{}", mesh_index),
        );
        Self {
            vertex_buffer: Some(vertex_buffer),
            model_uniform: Some(model_uniform),
            geometry_size: data.len(),
        }
    }

    pub fn update_buffer(&mut self, queue: &Queue, model_matrix: &[f32; 16]) {
        if let Some(uniform) = &self.model_uniform {
            uniform.update_buffer(model_matrix, queue);
        }
    }

    pub fn render(&self, pass: &mut RenderPass<'_>, instances: Range<u32>) {
        if let Some(buffer) = &self.vertex_buffer {
            if let Some(uniform) = &self.model_uniform {
                pass.set_vertex_buffer(0, buffer.slice(..));
                uniform.bind(pass, 1);
                pass.draw(0..self.geometry_size as u32, instances);
            } else {
                todo!()
            }
        } else {
            todo!()
        }
    }
}
