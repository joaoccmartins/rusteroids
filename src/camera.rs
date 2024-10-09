use glam::Mat4;
use wgpu::{util::DeviceExt, RenderPass};

const GL_TO_WGPU: Mat4 = Mat4::from_cols(
    glam::vec4(1.0, 0.0, 0.0, 0.0),
    glam::vec4(0.0, 1.0, 0.0, 0.0),
    glam::vec4(0.0, 0.0, 0.5, 0.0),
    glam::vec4(0.0, 0.0, 0.5, 1.0),
);

// TODO: convert this to trait to enable Perspective cameras as well
pub struct OrthoCamera {
    width: u32,
    height: u32,
    buffer: Option<wgpu::Buffer>,
    bind_group: Option<wgpu::BindGroup>,
}

impl OrthoCamera {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            buffer: None,
            bind_group: None,
        }
    }

    pub fn bind_group_layout_desc() -> wgpu::BindGroupLayoutDescriptor<'static> {
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
            label: Some("camera_bind_group_layout"),
        }
    }

    pub fn create_buffer(
        &mut self,
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) {
        self.buffer = Some(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&self.proj_matrix()),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
        );
        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.buffer.as_ref().unwrap().as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        }));
    }

    pub fn update_buffer(&self, queue: &wgpu::Queue) {
        if let Some(buffer) = &self.buffer {
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&self.proj_matrix()));
        } else {
            unreachable!()
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
        if let Some(bind_group) = &self.bind_group {
            pass.set_bind_group(0, bind_group, &[]);
        }
    }
}
