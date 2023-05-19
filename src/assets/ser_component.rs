use crate::ecs::*;

#[typetag::serde(tag = "component")]
pub trait SerializableComponent: Component {
    fn add_into(&self, entity_builder: &mut EntityBuilder);
}
