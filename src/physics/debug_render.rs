use std::sync::{Arc, Mutex};
use ash::vk;
use rapier3d::prelude::*;
use gpu_allocator::*;
use gpu_allocator::vulkan::*;

use crate::error::DesperoResult;
use crate::render::{
    backend::{
        pipeline::*,
        buffer::*,
        shader::*,
        swapchain::*,
        descriptor_pool::*,
    },
    renderer::Renderer,
};

pub const DEBUG_TOPOLOGY: vk::PrimitiveTopology = vk::PrimitiveTopology::LINE_LIST;

pub struct DebugRenderer {
    pub pipeline: Pipeline,
    pub vertexbuffer: Buffer,
    pub instancebuffer: Buffer,
}

impl DebugRenderer {
    pub fn new(
        logical_device: &ash::Device,
        swapchain: &Swapchain,
        descriptor_pool: &DescriptorPool,
        renderpass: &vk::RenderPass,
        allocator: &mut gpu_allocator::vulkan::Allocator,
    ) -> DesperoResult<Self> {
        let vertex_shader = vk::ShaderModuleCreateInfo::builder()
            .code(vk_shader_macros::include_glsl!(
                "./shaders/vertex_debug.glsl", 
                kind: vert,
            ));
        
        let fragment_shader = vk::ShaderModuleCreateInfo::builder()
            .code(vk_shader_macros::include_glsl!(
                "./shaders/fragment_debug.glsl",
                kind: frag,
            ));
            
        let vertex_attributes = vec![
            ShaderInputAttribute{
                binding: 0,
                location: 0,
                offset: 0,
                format: ShaderInputFormat::R32G32B32_SFLOAT,
            },
            ShaderInputAttribute{
                binding: 1,
                location: 1,
                offset: 0,
                format: ShaderInputFormat::R32G32B32A32_SFLOAT,
            }
        ]; 
        
        let pipeline = unsafe {Pipeline::init_internal(
            &logical_device,
            &swapchain,
            &descriptor_pool,
            *renderpass,
            &*vertex_shader,
            &*fragment_shader,
            vertex_attributes.clone(),
            12,
            16,
            DEBUG_TOPOLOGY,
        )?};
        
        let pipeline = Pipeline {
            pipeline,
            vertex_shader: *vertex_shader,
            fragment_shader: *fragment_shader,
            vertex_attributes,
            vertex_bytes: 12,
            instance_bytes: 16,
            topology: DEBUG_TOPOLOGY,
        };
        
        let vertexbuffer = Buffer::new(
            &logical_device,
            allocator,
            24,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            MemoryLocation::CpuToGpu,
            "Debug renderer vertex buffer"
        )?;
        
        let instancebuffer = Buffer::new(
            &logical_device,
            allocator,
            16,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            MemoryLocation::CpuToGpu,
            "Debug renderer instance buffer"
        )?;
        
        Ok(DebugRenderer {
            pipeline,
            vertexbuffer,
            instancebuffer,
        })
    }
    
    pub fn cleanup(
        &mut self,
        logical_device: &ash::Device,
        allocator: &Arc<Mutex<Allocator>>,
    ){
        let mut alloc: Option<Allocation> = None;
        std::mem::swap(&mut alloc, &mut self.vertexbuffer.allocation);
        (*allocator.lock().unwrap()).free(alloc.unwrap()).unwrap();
        
        let mut alloc: Option<Allocation> = None;
        std::mem::swap(&mut alloc, &mut self.instancebuffer.allocation);
        (*allocator.lock().unwrap()).free(alloc.unwrap()).unwrap();
        
        unsafe { logical_device.destroy_buffer(self.vertexbuffer.buffer, None) };
        unsafe { logical_device.destroy_buffer(self.instancebuffer.buffer, None) };
        
        self.pipeline.cleanup(&logical_device);
    }
}

impl DebugRenderBackend for Renderer {
    fn draw_line(
        &mut self,
        _object: DebugRenderObject<'_>,
        a: Point<f32>,
        b: Point<f32>,
        color: [f32; 4]
    ){        
        let vertexdata: [[f32; 3]; 2] = [a.into(), b.into()];
        
        self.debug_renderer.vertexbuffer.fill(
            &self.device,
            &mut *self.allocator.lock().unwrap(),
            &vertexdata,
        ).expect("Cannot fill debug renderer vertex buffer");
        
        self.debug_renderer.instancebuffer.fill(
            &self.device,
            &mut *self.allocator.lock().unwrap(),
            &color,
        ).expect("Cannot fill debug renderer instance buffer");
        
        unsafe {
            self.device.cmd_bind_pipeline(
                self.commandbuffer_pools.current_commandbuffer.unwrap(),
                vk::PipelineBindPoint::GRAPHICS,
                self.debug_renderer.pipeline.pipeline,
            );
            
            self.device.cmd_bind_vertex_buffers(
                self.commandbuffer_pools.current_commandbuffer.unwrap(),
                0,
                &[self.debug_renderer.vertexbuffer.buffer],
                &[0],
            );
            
            self.device.cmd_bind_vertex_buffers(
                self.commandbuffer_pools.current_commandbuffer.unwrap(),
                1,
                &[self.debug_renderer.instancebuffer.buffer],
                &[0],
            );
            
            self.device.cmd_draw(
                self.commandbuffer_pools.current_commandbuffer.unwrap(),
                2,
                1,
                0,
                0,
            );
        }
    }
}
