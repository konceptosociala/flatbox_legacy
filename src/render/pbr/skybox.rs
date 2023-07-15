use serde::{Serialize, Deserialize};
use vk_shader_macros::include_glsl;
use ash::vk;

use crate::render::{
    pbr::{
        texture::Texture,
        material::Material,
    },
    backend::shader::*,
};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SkyBox(pub Texture);

impl SkyBox {
    pub fn descriptor_image_info(&self) -> Option<vk::DescriptorImageInfo> {
        if let (Some(image_view), Some(sampler)) = (self.0.imageview, self.0.sampler) {
            Some(
                vk::DescriptorImageInfo {
                    image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                    image_view,
                    sampler,
                    ..Default::default()
                }
            )
        } else {
            None
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct SkyBoxMat;

#[typetag::serde]
impl Material for SkyBoxMat {
    fn vertex() ->  &'static [u32] {
        include_glsl!(
            "./src/shaders/skybox.vs", 
            kind: vert,
        )
    }

    fn fragment() ->  &'static [u32] {
        include_glsl!(
            "./src/shaders/skybox.fs",
            kind: frag,
        )
    }

    fn input() -> ShaderInput {
        ShaderInput { 
            attributes: vec![], 
            instance_size: 0, 
            topology: ShaderTopology::TRIANGLE_LIST,
        }
    }
}