use crate::prelude::*;

#[typetag::serde(tag = "component")]
pub trait SerializableComponent: Component {
    fn add_into(&self, entity_builder: &mut EntityBuilder);
}

#[macro_export]
macro_rules! impl_ser_component {
    ($($comp:ident),*) => {
        $(
            #[typetag::serde]
            impl SerializableComponent for $comp {
                fn add_into(&self, entity_builder: &mut EntityBuilder) {
                    entity_builder.add(self.clone());
                }
            }
        )*
    }
}

impl_ser_component!(
    bool, u8, i8, u16, i16, u32, i32, u64, i64, usize, isize,
    AssetHandle, BodyHandle, Camera, DirectionalLight, Mesh, PointLight, Timer, Transform
);
