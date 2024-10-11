use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, Fields};
/// Ensures the struct is capable of generating a VertexBufferLayout
/// by calling desc. Requires the existing each field to implement
/// ```ConstFormat``` trait
///
/// Example
/// ```
/// struct MyVec3{
///     x: f32,
///     y: f32,
///     z: f32,
/// }
///
/// const_format_of!(MyVec3 => wgpu::VertexFormat::Float32x3);
///
/// #[derive(VertexAttributeArray)]
/// struct Vertex {
///     pos: MyVec3
///     uv: [f32; 2] // Implements ConstFormat by default
/// }
/// ```
///
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
    let mut previous_type = None;
    let field_types = fields.iter().enumerate().map(|(i, f)| {
        let ty = &f.ty;
        let offset = if let Some(prev) = previous_type {
            quote! {size_of::<#prev>() as u64}
        } else {
            quote! { 0 as u64}
        };
        previous_type = Some(ty);
        quote! {
            wgpu::VertexAttribute {
                format: format_of::<#ty>(),
                offset: #offset,
                shader_location: #i as u32,
            }
        }
    });

    let attrs_array_len = field_types.len();

    let gen = quote! {
        impl VertexAttributeArray for #name {
            fn desc() -> wgpu::VertexBufferLayout<'static> {
                static ATTRIBUTES: [wgpu::VertexAttribute; #attrs_array_len] =  [
                    #(#field_types),*
                ];
                wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &ATTRIBUTES,
                }
            }
        }
    };

    gen.into()
}
