use crate::render::{
    pbr::texture::Texture,
    backend::{
        pipeline::Pipeline,
        buffer::Buffer,
    }
};

pub struct SkyBox {
    pub pipeline: Pipeline,
    pub texture: Texture,
    pub buffer: Buffer,
}

impl SkyBox {
    pub fn new(path: &'static str) -> Self {
        todo!("Skybox::new()");
    }

    pub fn set_texture(&mut self, path: &'static str){
        todo!("Skybox::set_texture()");
    }
}