use crate::graphics::vulkanish::*;

struct Model {
    vertexdata: Vec<[f32; 5]>,
    visible_instances: Vec<[f32; 4]>,
    invisible_instances: Vec<[f32; 4]>,
    vertexbuffer: Buffer,
    instancebuffer: Buffer,
}
