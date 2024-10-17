use std::{iter, ops::Deref};

use std::collections::HashMap;
use winit::{event::*, window::Window};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::camera::{MyMat4, OrthoCamera};
use crate::mesh::{Geometry, Vertex};

use wgpu_utils::{Bindable, VertexAttributeArray};

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

    pub fn get_queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn get_device(&self) -> &wgpu::Device {
        &self.device
    }
}

/// A gadget to bind a Shader, VertexBuffer and Bindgroups into a RenderPipeline
pub struct Gadget {
    pipeline: wgpu::RenderPipeline,
}

impl Gadget {
    pub fn from(
        shader_src: wgpu::ShaderModuleDescriptor,
        vertex_layout: wgpu::VertexBufferLayout,
        uniforms: &[&wgpu::BindGroupLayout],
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(shader_src);

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: uniforms,
                push_constant_ranges: &[],
            });

        Self {
            pipeline: device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[vertex_layout],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::LineStrip,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            }),
        }
    }
}

impl Deref for Gadget {
    type Target = wgpu::RenderPipeline;

    fn deref(&self) -> &Self::Target {
        &self.pipeline
    }
}

/// A renderer struct, binding Gadgets to multiple Uniform Bindings,
/// enabling the rendering of Geometries. Effectively manages the
/// rendering of Geometries with differing Gadgets (representing WebGPU
/// pipelines).
/// TODO: enable the usage of multiple Gadgets to render different Meshes
pub struct Renderer<'a> {
    camera: OrthoCamera,
    gadget: Gadget,
    uniforms: HashMap<&'a str, wgpu::BindGroupLayout>,
    context: Context<'a>,
}

impl<'a> Renderer<'a> {
    pub async fn new(window: &'a Window) -> Renderer<'a> {
        let context = Context::<'a>::new(window).await;
        let mut uniforms: HashMap<_, _> = HashMap::new();

        // Create camera_bind_group_layout
        let camera_binding = context.device.create_bind_group_layout(&MyMat4::desc());
        let mut camera = OrthoCamera::new(context.size.width, context.size.height);
        camera.setup(&context.device, &camera_binding);

        // Create mesh_bind_group_layout
        let model_matrix_binding = context.device.create_bind_group_layout(&MyMat4::desc());

        // Create a gadget for rendering with a camera and model matrix, using Vertex as
        // the geometry buffer
        let gadget = Gadget::from(
            wgpu::include_wgsl!("shader.wgsl"),
            Vertex::desc(),
            &[&camera_binding, &model_matrix_binding],
            &context.device,
            context.config.format,
        );

        // Add UniformBindings to the uniform map
        uniforms.insert("camera", camera_binding);
        uniforms.insert("model", model_matrix_binding);

        Self {
            camera,
            gadget,
            uniforms,
            context,
        }
    }

    /// Get an uniform that was used during the creation of one of the Gadgets
    pub fn get_uniform_binding(&self, uniform_name: &str) -> &wgpu::BindGroupLayout {
        if let Some(uniform_binding) = self.uniforms.get(uniform_name) {
            uniform_binding
        } else {
            unreachable!()
        }
    }

    /// Renders meshes using the single Gadgets
    /// TODO: enable multiple Gadgets and different runs of render for the same pass
    pub fn render(&mut self, meshes: &[Geometry]) -> Result<(), wgpu::SurfaceError> {
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
            meshes.iter().for_each(|mesh| {
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
}

impl<'a> Deref for Renderer<'a> {
    type Target = Context<'a>;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}
