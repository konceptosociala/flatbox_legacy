use ash::vk;

pub type ShaderInputAttribute = vk::VertexInputAttributeDescription;
pub type ShaderInputFormat = vk::Format;
pub type ShaderTopology = vk::PrimitiveTopology;

#[derive(Debug, Default, Clone)]
pub struct ShaderInput {
    pub attributes: Vec<ShaderInputAttribute>,
    pub instance_size: usize,
    pub topology: ShaderTopology,
}
