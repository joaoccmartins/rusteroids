pub trait ConstFormat {
    const FORMAT: wgpu::VertexFormat;
}

pub const fn format_of<T: ConstFormat>() -> wgpu::VertexFormat {
    T::FORMAT
}
/// Defines the default VertexFormat of a particular type.
/// By defining it using
/// ``````
/// const_format_of!(MyVec3 => VertexFormat::Float32x3)
/// ```
///
/// You can then use its format with the following call
/// ```
/// let vertex_format: wgpu::VertexFormat = format_of::<MyVec3>();
/// ```
///
/// And use this in conjunction with VertexAttributeArray macro
/// ```
/// #[derive(VertexAttributeArray)]
/// struct MyVertexBuffer{
///     pos: MyVec3,
/// }
macro_rules! const_format_of {
    ($T:ty => $format:expr) => {
        impl ConstFormat for $T {
            const FORMAT: wgpu::VertexFormat = $format;
        }
    };
}

const_format_of!(f32 => wgpu::VertexFormat::Float32);
const_format_of!([f32; 2] => wgpu::VertexFormat::Float32x2);
const_format_of!([f32; 3] => wgpu::VertexFormat::Float32x3);
const_format_of!([f32; 4] => wgpu::VertexFormat::Float32x4);
const_format_of!(u32 => wgpu::VertexFormat::Uint32);
const_format_of!([u32; 2] => wgpu::VertexFormat::Uint32x2);
const_format_of!([u32; 3] => wgpu::VertexFormat::Uint32x3);
