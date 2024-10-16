pub use internals::format_of;
pub use wgpu_utils_derive::VertexAttributeArray;

pub trait VertexAttributeArray {
    /// Generates a VertexBufferLayout to be used in a RenderPipeline
    /// TODO: refactor static out of it.
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

pub trait Bindable {
    /// Generates a BindGroupDescriptor to be used in a RenderPipeline
    fn desc(label: Option<&str>) -> wgpu::BindGroupLayoutDescriptor<'static>;
}
