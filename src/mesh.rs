use std::ops::Range;

use glam::Mat4;
use wgpu::util::DeviceExt;
use wgpu::RenderPass;

use crate::renderer::Context;
use crate::utils;

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

pub struct Mesh {
    data: Vec<Vertex>,
    vertex_buffer: Option<wgpu::Buffer>,
    model_buffer: Option<wgpu::Buffer>,
    bind_group: Option<wgpu::BindGroup>,
}

impl Mesh {
    pub fn new(data: &[Vertex]) -> Self {
        Self {
            data: data.to_vec(),
            vertex_buffer: None,
            model_buffer: None,
            bind_group: None,
        }
    }

    pub fn bind_group_layout_desc() -> wgpu::BindGroupLayoutDescriptor<'static> {
        utils::uniform_layout_descriptor("mesh_bind_group_layout")
    }

    pub fn update(&mut self, context: &Context, model_matrix: &[f32; 16]) {
        if let Some(buffer) = &self.model_buffer {
            context
                .queue
                .write_buffer(buffer, 0, bytemuck::cast_slice(model_matrix));
        }
    }

    pub fn render(&self, pass: &mut RenderPass<'_>, instances: Range<u32>) {
        if let Some(buffer) = &self.vertex_buffer {
            if let Some(bind_group) = &self.bind_group {
                pass.set_bind_group(1, bind_group, &[]);
                pass.set_vertex_buffer(0, buffer.slice(..));
                pass.draw(0..self.data.len() as u32, instances);
            } else {
                todo!()
            }
        } else {
            todo!()
        }
    }

    pub(crate) fn create_buffer(
        &mut self,
        device: &wgpu::Device,
        bind_group_layou: &wgpu::BindGroupLayout,
        mesh_index: u32,
    ) {
        self.vertex_buffer = Some(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("vertex{}_buffer", mesh_index)),
                contents: bytemuck::cast_slice(&self.data),
                usage: wgpu::BufferUsages::VERTEX,
            }),
        );
        let model = Mat4::IDENTITY.to_cols_array();
        let label = &format!("mesh{}_buffer", mesh_index);
        self.model_buffer = Some(utils::create_buffer(model, device, label));

        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layou,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.model_buffer.as_ref().unwrap().as_entire_binding(),
            }],
            label: Some(&format!("mesh{}_bind_group", mesh_index)),
        }));
    }
}
