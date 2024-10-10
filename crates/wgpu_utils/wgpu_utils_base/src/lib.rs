pub trait VertexAttributeArray {
    /// Generates a VertexBufferLayout to be used in a RenderPipeline
    /// TODO: refactor static out of it.
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

pub trait Attribute {
    fn format() -> wgpu::VertexFormat;
}

impl Attribute for [f32; 2] {
    fn format() -> wgpu::VertexFormat {
        wgpu::VertexFormat::Float32x2
    }
}

impl Attribute for [f32; 3] {
    fn format() -> wgpu::VertexFormat {
        wgpu::VertexFormat::Float32x3
    }
}
