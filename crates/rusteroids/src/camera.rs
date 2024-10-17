use glam::Mat4;
use wgpu::RenderPass;
use wgpu_utils::{binding_type_of, Bindable, BindableGroup};

use crate::utils::UniformBuffer;

const GL_TO_WGPU: Mat4 = Mat4::from_cols(
    glam::vec4(1.0, 0.0, 0.0, 0.0),
    glam::vec4(0.0, 1.0, 0.0, 0.0),
    glam::vec4(0.0, 0.0, 0.5, 0.0),
    glam::vec4(0.0, 0.0, 0.5, 1.0),
);

#[derive(BindableGroup, Clone)]
pub struct MyMat4 {
    data: [f32; 16],
}

// TODO: convert this to trait to enable Perspective cameras as well
pub struct OrthoCamera {
    width: u32,
    height: u32,
    uniform: Option<UniformBuffer>,
}

impl OrthoCamera {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            uniform: None,
        }
    }

    pub fn setup(&mut self, device: &wgpu::Device, bind_group_layout: &wgpu::BindGroupLayout) {
        self.uniform = Some(UniformBuffer::new(
            &self.proj_matrix(),
            device,
            bind_group_layout,
            "camera",
        ))
    }

    pub fn update_buffer(&self, queue: &wgpu::Queue) {
        if let Some(uniform) = &self.uniform {
            uniform.update_buffer(&self.proj_matrix(), queue);
        }
    }

    fn proj_matrix(&self) -> [f32; 16] {
        let (half_width, half_height) = ((self.width / 2) as f32, (self.height / 2) as f32);
        Mat4::orthographic_lh(
            -half_width,
            half_width,
            -half_height,
            half_height,
            0.01,
            1000.0,
        )
        .mul_mat4(&GL_TO_WGPU)
        .transpose()
        .to_cols_array()
    }

    pub fn resize(&mut self, width: u32, height: u32, queue: &wgpu::Queue) {
        if self.width != width || self.height != height {
            self.width = width;
            self.height = height;
            self.update_buffer(queue);
        }
    }

    pub fn bind_group(&self, pass: &mut RenderPass<'_>) {
        if let Some(uniform) = &self.uniform {
            uniform.bind(pass, 0);
        }
    }
}
