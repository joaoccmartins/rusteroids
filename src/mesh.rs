use std::ops::Range;

use glam::{vec3, Mat4, Vec2};
use wgpu::util::DeviceExt;
use wgpu::RenderPass;

const GL_TO_WGPU: Mat4 = Mat4::from_cols(
    glam::vec4(1.0, 0.0, 0.0, 0.0),
    glam::vec4(0.0, 1.0, 0.0, 0.0),
    glam::vec4(0.0, 0.0, 0.5, 0.0),
    glam::vec4(0.0, 0.0, 0.5, 1.0),
);

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
    pos: Vec2,
    rotation: f32,
}

impl Mesh {
    pub fn new(data: &[Vertex]) -> Self {
        Self {
            data: data.to_vec(),
            vertex_buffer: None,
            rotation: 0.0,
            pos: Vec2::ZERO,
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

    pub fn get_xform(&mut self) -> [f32; 16] {
        let angle = self.rotation;
        let pos = vec3(self.pos.x, self.pos.y, 0.0);
        self.rotation += 1.0_f32.to_radians();
        Mat4::from_rotation_z(angle)
            .mul_mat4(&Mat4::from_translation(pos))
            .mul_mat4(&GL_TO_WGPU)
            .transpose()
            .to_cols_array()
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
