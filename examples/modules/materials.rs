use despero::prelude::*;
use ash::vk;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MyMaterial {
    pub colour: [f32; 3],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TexMaterial {
    pub texture_id: u32,
}

impl Material for MyMaterial {
    fn pipeline(renderer: &Renderer) -> Pipeline {
        let vertex_shader = vk::ShaderModuleCreateInfo::builder()
            .code(vk_shader_macros::include_glsl!(
                "./shaders/vertex_simple.glsl", 
                kind: vert,
            ));
        
        let fragment_shader = vk::ShaderModuleCreateInfo::builder()
            .code(vk_shader_macros::include_glsl!(
                "./shaders/fragment_simple.glsl",
                kind: frag,
            ));
            
        let instance_attributes = vec![
            ShaderInputAttribute {
                binding: 1,
                location: 3,
                offset: 0,
                format: ShaderInputFormat::R32G32B32_SFLOAT,
            },
        ];
        
        Pipeline::init(
            &renderer,
            &vertex_shader,
            &fragment_shader,
            instance_attributes,
            12,
            vk::PrimitiveTopology::TRIANGLE_LIST,
        ).expect("Cannot create pipeline")
    }
}

impl Material for TexMaterial {
    fn pipeline(renderer: &Renderer) -> Pipeline {
        let vertex_shader = vk::ShaderModuleCreateInfo::builder()
            .code(vk_shader_macros::include_glsl!(
                "./shaders/vertex_simple2.glsl", 
                kind: vert,
            ));
        
        let fragment_shader = vk::ShaderModuleCreateInfo::builder()
            .code(vk_shader_macros::include_glsl!(
                "./shaders/fragment_simple2.glsl",
                kind: frag,
            ));
            
        let instance_attributes = vec![
            ShaderInputAttribute {
                binding: 1,
                location: 3,
                offset: 0,
                format: ShaderInputFormat::R8G8B8A8_UINT,
            },
        ];
        
        Pipeline::init(
            &renderer,
            &vertex_shader,
            &fragment_shader,
            instance_attributes,
            4,
            vk::PrimitiveTopology::TRIANGLE_LIST,
        ).expect("Cannot create pipeline")
    }
}
