use internals::format_of;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, Fields};
/// Ensures the struct is capable of generating a VertexBufferLayout
/// by calling desc. Requires the existing of a ```const ATTR: [wgpu::VertexAttribute; N]```
///
/// Example
/// ```
/// struct Vertex {
///     pos: [f32; 3]
///     uv:  [f32; 2]
/// }
///
/// impl Vertex{
///     const ATTR: [wgpu::VertexAttribute; 2] = wpug::vertex_attr_array![0 => Float32x3; 1 Float32x2];
/// }
/// ```
///
/// TODO: make a bind_to_group(binding: u32, bind_group: &BindGroup, offsets: &[DynamicOffset])
/// TODO: refactor ATTR requirement out
#[proc_macro_derive(VertexAttributeArray)]
pub fn vertex_attribute_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_vertex_attribute(&ast)
}

fn impl_vertex_attribute(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let fields = match &ast.data {
        Data::Struct(data) => {
            if let Fields::Named(named_fields) = &data.fields {
                named_fields.named.clone()
            } else {
                panic!("#[derive(VertexAttributeArray)] is only supported on structs with named fields");
            }
        }
        // TODO: Add enums
        _ => panic!("#[derive(VertexAttributeArray)] is only supported on structs"),
    };

    // Generate the vertex attributes
    let field_types = fields.iter().enumerate().map(|(i, f)| {
        let ty = &f.ty;
        quote! {
            wgpu::VertexAttribute {
                format: format_of::<#ty>(),
                offset: size_of::<#ty>() as u64,
                shader_location: #i as u32,
            }
        }
    });

    let attrs_array_len = field_types.len();

    let gen = quote! {
        impl VertexAttributeArray for #name {
            fn desc() -> wgpu::VertexBufferLayout<'static> {
                static attr: [wgpu::VertexAttribute; #attrs_array_len] =  [
                    #(#field_types),*
                ];
                wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &attr,
                }
            }
        }
    };

    gen.into()
}
