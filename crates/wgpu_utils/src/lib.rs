pub use vertex_attribute_derive::VertexAttributeArray;

pub trait VertexAttributeArray {
    /// Generates a VertexBufferLayout to be used in a RenderPipeline
    /// TODO: refactor static out of it.
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}
