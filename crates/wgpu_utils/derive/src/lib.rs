use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, Fields};

/// Derives the ```VertexAttributeArray``` trait
///
/// Ensures the struct is capable of generating a VertexBufferLayout
/// by calling desc. Requires that each field implement
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
    // We pick each of the fields in our struct
    let fields = match &ast.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields_named) => fields_named.named.clone(),
            Fields::Unnamed(fields_unnamed) => fields_unnamed.unnamed.clone(),
            _ => panic!("#[derive(VertexAttributeArray)] is not supported in unit structs"),
        },
        // TODO: Add enums
        _ => panic!("#[derive(VertexAttributeArray)] is only supported in structs"),
    };

    // Generate the vertex attributes iterator that we'll be inserting in our static array
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

#[proc_macro_derive(BindableGroup)]
pub fn bindable_group_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_bindable_group(&ast)
}

fn impl_bindable_group(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let fields = match &ast.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => fields_named.named.clone(),
            Fields::Unnamed(fields_unnamed) => fields_unnamed.unnamed.clone(),
            _ => panic!("#[derive(BindableGroup)] is not supported in unit structs"),
        },
        _ => panic!("#[derive(BindableGroup)] is only supported for structs"),
    };

    let field_entries = fields.iter().map(|f| {
        let ty = &f.ty;
        quote! {
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: binding_type_of::<#ty>(),
                count: None,
            }
        }
    });

    let entries_array_len = field_entries.len();

    // TODO: add label setup to derive macro
    let gen = quote! {

        impl Bindable for #name {
            fn desc() -> wgpu::BindGroupLayoutDescriptor<'static> {
                static ENTRIES: [wgpu::BindGroupLayoutEntry; #entries_array_len] = [#(#field_entries),*];
                wgpu::BindGroupLayoutDescriptor {
                    entries: &ENTRIES,
                    label: None,
                }
            }
        }
    };

    gen.into()
}
