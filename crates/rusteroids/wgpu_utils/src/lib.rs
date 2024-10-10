pub use vertex_attribute_derive::VertexAttribute;

pub trait VertexAttribute {
    const ATTRIBS: [wgpu::VertexAttribute; 2];
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}
