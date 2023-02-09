use rapier3d::prelude::*;

use crate::render::renderer::Renderer;

impl DebugRenderBackend for Renderer {
    fn draw_line(
        &mut self,
        object: DebugRenderObject<'_>,
        a: Point<f32>,
        b: Point<f32>,
        color: [f32; 4]
    ){
        todo!();
    }
}
