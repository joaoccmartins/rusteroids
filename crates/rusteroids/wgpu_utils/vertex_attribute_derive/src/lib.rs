use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(VertexAttributeArray)]
pub fn vertex_attribute_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_vertex_attribute(&ast)
}

fn impl_vertex_attribute(_ast: &syn::DeriveInput) -> TokenStream {
    let gen = quote! {
        impl VertexAttributeArray for Vertex {
            const ATTRIBS: [wgpu::VertexAttribute; 2] =
                wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x3];
            fn desc() -> wgpu::VertexBufferLayout<'static> {
                wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &Self::ATTRIBS,
                }
            }
        }
    };
    gen.into()
}
