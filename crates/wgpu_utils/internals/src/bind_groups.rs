/// TODO: get better name
/// TODO: expand to other BindingTypes
pub enum BindType {
    Uniform,
}

/// Defines the default Binding Type of a particular type.
/// By defining it using
/// ``````
/// const_binding_type_of!(MyMat4 => BindingType::Buffer)
/// ```
///
/// You can then use its binding type with the following call
/// ```
/// let binding_type: wgpu::VertexFormat = binding_type_of::<MyMat4>();
/// ```
///
pub trait ConstBindingType {
    const TYPE: BindType;
}

pub const fn binding_type_of<T: ConstBindingType>() -> wgpu::BindingType {
    match T::TYPE {
        BindType::Uniform => wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        _ => todo!(),
    }
}

/// TODO: simplify ```$binding_type:expr``` requirements
macro_rules! const_binding_type_of {
    ($T:ty => $binding_type:expr) => {
        impl ConstBindingType for $T {
            const TYPE: BindType = $binding_type;
        }
    };
}

const_binding_type_of!([f32; 16] => BindType::Uniform);
