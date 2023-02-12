use ash::vk;
use rapier3d::prelude::*;

use crate::error::DesperoResult;
use crate::render::{
    backend::{
        pipeline::Pipeline,
        shader::*,
    },
    renderer::Renderer,
};

impl DebugRenderBackend for Renderer {
    fn draw_line(
        &mut self,
        _object: DebugRenderObject<'_>,
        a: Point<f32>,
        b: Point<f32>,
        color: [f32; 4]
    ){
        todo!();
    }
}
