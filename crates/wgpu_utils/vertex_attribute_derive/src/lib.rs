use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, Fields};
use wgpu_utils_base::Attribute;

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
                format: <#ty as Attribute>::format(),
                offset: <#ty as Attribute>::format().size(),
                shader_location: #i as u32,
            }
        }
    });

    let attrs_array_len = fields.len(); // The length of the array

    let gen = quote! {
        impl #name {
            pub const fn attrs() -> &'static [wgpu::VertexAttribute; #attrs_array_len] {
                static ATTRS: [wgpu::VertexAttribute; #attrs_array_len] = [
                    #(#field_types),*
                ];
                &ATTRS
            }
        }

        impl VertexAttributeArray for #name {
            fn desc() -> wgpu::VertexBufferLayout<'static> {
                wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: Self::attrs(),
                }
            }
        }
    };

    gen.into()
}
