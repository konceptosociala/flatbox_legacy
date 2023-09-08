use crate::prelude::*;

#[typetag::serde(tag = "component")]
pub trait SerializableComponent: Component {
    fn add_into(&self, entity_builder: &mut EntityBuilder);
}

/// Macro for implementing [`SerializableComponent`] trait for multiple types, that implement [`Clone`] trait; for using in [`Scene`]'s. Use to avoid boilerplate
/// 
/// # Usage example
/// 
/// ```rust
/// #[derive(Clone)]
/// struct ComponentA;
/// 
/// #[derive(Clone)]
/// struct ComponentB;
/// 
/// #[derive(Clone)]
/// struct ComponentC;
/// 
/// impl_ser_component!(ComponentA, ComponentB, ComponentC);
/// 
/// ```
/// 
#[macro_export]
macro_rules! impl_ser_component {
    ($($comp:ty),+) => {
        $(
            #[typetag::serde]
            impl SerializableComponent for $comp {
                fn add_into(&self, entity_builder: &mut EntityBuilder) {
                    entity_builder.add(self.clone());
                }
            }
        )+
    }
}

impl_ser_component!(
    bool, u8, i8, u16, i16, u32, i32, u64, i64, usize, isize,
    BodyHandle, Timer, Transform, AssetHandle<'S'>
);

#[cfg(feature = "render")]
impl_ser_component!(
    Camera, DirectionalLight, Model, PointLight, 
    AssetHandle<'T'>, AssetHandle<'M'>
);
