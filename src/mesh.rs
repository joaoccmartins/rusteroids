use std::ops::Range;

use wgpu::util::DeviceExt;
use wgpu::RenderPass;

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
}

impl Mesh {
    pub fn new(data: &[Vertex]) -> Self {
        Self {
            data: data.to_vec(),
            vertex_buffer: None,
        }
    }

    pub fn render(&self, pass: &mut RenderPass<'_>, instances: Range<u32>) {
        if let Some(buffer) = &self.vertex_buffer {
            pass.set_vertex_buffer(0, buffer.slice(..));
            pass.draw(0..self.data.len() as u32, instances);
        } else {
            todo!()
        }
    }

    pub(crate) fn create_buffer(&mut self, device: &wgpu::Device) {
        self.vertex_buffer = Some(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&self.data),
                usage: wgpu::BufferUsages::VERTEX,
            }),
        );
    }
}
