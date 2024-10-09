use std::{
    iter,
    ops::{Deref, DerefMut},
};

use winit::{event::*, window::Window};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::{camera::OrthoCamera, utils::Gadget};
use crate::{
    mesh::{Geometry, Vertex},
    utils::UniformBinding,
};

pub struct Context<'a> {
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    window: &'a Window,
}

impl<'a> Context<'a> {
    pub async fn new(window: &'a Window) -> Context<'a> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            // TODO: Change to WebGPU
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    memory_hints: Default::default(),
                },
                // Some(&std::path::Path::new("trace")), // Trace path
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
        };

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
        }
    }

    pub fn window(&self) -> &Window {
        self.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn get_size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }
}

pub struct Renderer<'a> {
    meshes: Vec<Geometry>,
    camera: OrthoCamera,
    gadget: Gadget,
    pub(crate) model_matrix_binding: UniformBinding,
    pub(crate) context: Context<'a>,
}

impl<'a> Renderer<'a> {
    pub async fn new(window: &'a Window) -> Renderer<'a> {
        let context = Context::<'a>::new(window).await;
        // Create camera_bind_group_layout
        let camera_binding = UniformBinding::new::<OrthoCamera>(&context.device);
        let mut camera = OrthoCamera::new(context.size.width, context.size.height);
        camera.setup(&context.device, &camera_binding);

        // Create mesh_bind_group_layout
        let model_matrix_binding = UniformBinding::new::<Geometry>(&context.device);
        let gadget = Gadget::from(
            wgpu::include_wgsl!("shader.wgsl"),
            Vertex::desc(),
            &[&camera_binding, &model_matrix_binding],
            &context.device,
            context.config.format,
        );

        Self {
            meshes: Vec::new(),
            camera,
            gadget,
            model_matrix_binding,
            context,
        }
    }

    pub fn update(&mut self, model: &[f32; 16]) {
        let context = &self.context;
        for mesh in &mut self.meshes {
            mesh.update_buffer(&context.queue, model)
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.context.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.gadget);
            self.camera.bind_group(&mut render_pass);
            self.meshes.iter_mut().for_each(|mesh| {
                mesh.render(&mut render_pass, 0..1);
            });
        }
        self.context.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    #[allow(unused_variables)]
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.context.resize(new_size);
            self.camera
                .resize(new_size.width, new_size.height, &self.context.queue);
        }
    }

    pub fn add_mesh(&mut self, mesh: &[Vertex]) {
        let mut mesh = Geometry::new(mesh);
        mesh.setup(
            &self.context.device,
            &self.model_matrix_binding,
            self.meshes.len() as u32,
        );
        self.meshes.push(mesh);
    }
}

impl<'a> Deref for Renderer<'a> {
    type Target = Context<'a>;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

// Not sure we should be exposing Context as mutable in here
// TODO: Review
impl<'a> DerefMut for Renderer<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.context
    }
}
