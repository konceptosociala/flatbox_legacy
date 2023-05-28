use std::mem::ManuallyDrop;
use std::sync::{Arc, Mutex};
use std::any::TypeId;
use std::collections::HashMap;
use ash::vk;
use ash::Device;
use gpu_allocator::vulkan::*;
use gpu_allocator::MemoryLocation;
use nalgebra as na;
use winit::{
    window::{
        Window as WinitWindow
    },
};
#[cfg(feature = "egui")]
use egui_winit_ash_integration::*;

use crate::assets::*;
use crate::ecs::*;
use crate::render::{
    backend::{
        instance::Instance,
        window::Window,
        queues::QueueFamilies,
        swapchain::Swapchain,
        pipeline::Pipeline,
        commandbuffers::CommandBufferPools,
        buffer::Buffer,        
        descriptor_pool::DescriptorPool,
    },
    pbr::{
        model::*,
        material::*,
    },
};
#[cfg(feature = "egui")]
use crate::render::ui::{GuiContext, GuiHandler};

use crate::physics::{
    physics_handler::PhysicsHandler,
    debug_render::*,
};
use crate::math::transform::Transform;
use crate::ecs::event::EventHandler;
use crate::error::DesperoResult;
use crate::WindowBuilder;

/// Maximum number of textures, which can be pushed to descriptor sets
pub const MAX_NUMBER_OF_TEXTURES: u32 = 1024;

#[derive(Debug, Clone, Default, PartialEq, Hash)]
pub enum RenderType {
    #[default]
    Forward,
    Deferred,
}

pub type PipelineCollection = HashMap<TypeId, Pipeline>;

/// Main rendering collection, including Vulkan components
pub struct Renderer {
    pub(crate) instance: Instance,
    pub(crate) window: Window,
    pub(crate) queue_families: QueueFamilies,
    pub(crate) device: Device,
    pub(crate) swapchain: Swapchain,
    pub(crate) renderpass: vk::RenderPass,
    pub(crate) material_pipelines: PipelineCollection,
    pub(crate) debug_renderer: DebugRenderer,
    pub(crate) commandbuffer_pools: CommandBufferPools,
    pub(crate) allocator: Arc<Mutex<Allocator>>,
    pub(crate) camera_buffer: Buffer,
    pub(crate) light_buffer: Buffer,
    pub(crate) descriptor_pool: DescriptorPool,
    #[cfg(feature = "egui")]
    pub(crate) egui: GuiHandler,
}

impl Renderer {    
    pub(crate) fn init(window_builder: WindowBuilder) -> DesperoResult<Renderer> {
        let instance = Instance::init()?;
        let window = Window::init(&instance, window_builder.clone().into())?;
        let (device, queue_families) = QueueFamilies::init(&instance, &window)?;
            
        let mut allocator = Allocator::new(&AllocatorCreateDesc {
            instance: instance.instance.clone(),
            device: device.clone(),
            physical_device: instance.physical_device.clone(),
            debug_settings: Default::default(),
            buffer_device_address: true,
        }).expect("Cannot create allocator");
        
        let render_type = window_builder.renderer.unwrap_or(RenderType::Forward);
        if render_type == RenderType::Deferred {
            log::error!("Deferred rendering is not supported yet");
        }
        
        let mut swapchain = Swapchain::init(&instance, &device, &window.surface, &queue_families, &mut allocator)?;
        
        let renderpass = Pipeline::init_renderpass(&device, instance.physical_device.clone(), &window.surface)?;
        swapchain.create_framebuffers(&device, renderpass)?;
                
        let commandbuffer_pools = CommandBufferPools::init(&device, &queue_families, &swapchain)?;
        
        let mut camera_buffer = Buffer::new(
            &device,
            &mut allocator,
            128,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            MemoryLocation::CpuToGpu,
            "Uniform buffer"
        )?;
        
        let camera_transform: [[[f32; 4]; 4]; 2] = [
            na::Matrix4::identity().into(),
            na::Matrix4::identity().into(),
        ];
        camera_buffer.fill(&device, &mut allocator, &camera_transform)?;
        
        let mut light_buffer = Buffer::new(
            &device,
            &mut allocator,
            8,
            vk::BufferUsageFlags::STORAGE_BUFFER,
            MemoryLocation::CpuToGpu,
            "Light buffer",
        )?;
        light_buffer.fill(&device, &mut allocator, &[0.,0.])?;
        
        let descriptor_pool = unsafe { DescriptorPool::init(&device, &swapchain)? };
        unsafe { descriptor_pool.bind_buffers(&device, &camera_buffer, &light_buffer) };
        
        let debug_renderer = DebugRenderer::new(
            &device,
            &swapchain,
            &descriptor_pool,
            &renderpass,
            &mut allocator,
        )?;
        
        let allocator = Arc::new(Mutex::new(allocator));
        
        #[cfg(feature = "egui")]
        let egui = ManuallyDrop::new(Integration::new(
            &*window.event_loop.lock().unwrap(),
            swapchain.extent.width,
            swapchain.extent.height,
            1.0,
            egui::FontDefinitions::default(),
            egui::Style::default(),
            device.clone(),
            Arc::clone(&allocator),
            queue_families.graphics_index.unwrap(),
            queue_families.graphics_queue,
            swapchain.swapchain_loader.clone(),
            swapchain.swapchain,
            *window.surface.get_formats(instance.physical_device)?.first().unwrap(),
        ));
         
        Ok(Renderer {
            instance,
            window,
            queue_families,
            device,
            swapchain,
            renderpass,
            material_pipelines: PipelineCollection::new(),
            debug_renderer,
            commandbuffer_pools,
            allocator,
            camera_buffer,
            light_buffer,
            descriptor_pool,
            #[cfg(feature = "egui")]
            egui,
        })
    }
    
    pub fn get_window(&self) -> Arc<Mutex<WinitWindow>> {
        self.window.window.clone()
    }
    
    pub fn bind_material<M: Material + Sync + Send>(&mut self){
        if self.material_pipelines.contains_key(&TypeId::of::<M>()) {
            log::error!("Material type '{}' is already bound!", std::any::type_name::<M>());
        } else {
            self.material_pipelines.insert(TypeId::of::<M>(), M::pipeline(&self));
        }
    }
    
    pub fn update_commandbuffer<W: borrow::ComponentBorrow>(
        &mut self,
        world: &mut SubWorld<W>,
        #[cfg(feature = "egui")]
        event_handler: &mut EventHandler<GuiContext>,
        physics_handler: &mut PhysicsHandler,
        asset_manager: &AssetManager,
        index: usize,
    ) -> DesperoResult<()> {        
        let commandbuffer = *self.commandbuffer_pools.get_commandbuffer(index).unwrap();
        
        update_texture_sets(&asset_manager, &self.descriptor_pool, &self.swapchain, &self.device);
        
        begin_commandbuffer(&commandbuffer, &mut self.commandbuffer_pools, &self.device)?;
        begin_renderpass(&self.renderpass, &self.swapchain, &self.device, &commandbuffer, index);
        
        bind_descriptor_sets(&self.device, &commandbuffer, &self.descriptor_pool, index);
        
        for mat_type in self.material_pipelines.keys() {
            bind_graphics_pipeline(&self.material_pipelines, &self.device, &commandbuffer, mat_type);
            
            for (_, (mesh, handle, transform)) in &mut world.query::<(
                &Mesh, &AssetHandle, &Transform,
            )>(){
                if let (Some(vertexbuffer), Some(instancebuffer), Some(indexbuffer)) = 
                    (&mesh.vertexbuffer, &mesh.instancebuffer, &mesh.indexbuffer)
                {
                    let material = asset_manager.get_material(*handle).unwrap();
                    if (**material).type_id() == *mat_type {
                        bind_vertex_buffers(&self.device, &commandbuffer, &indexbuffer, &vertexbuffer, &instancebuffer);
                        
                        apply_transform(&self.device, &self.descriptor_pool, &commandbuffer, &transform);
                        draw_mesh(&self.device, &commandbuffer, mesh.indexdata.len());
                    }
                }
            }
        }
        
        match std::env::var("PHYSICS_DEBUG") {
            Ok(v) => if v.as_str() == "true" { 
                physics_handler.debug_render(self)
            },
            _ => {},
        }
        
        end_renderpass(&self.device, &commandbuffer);
        
        #[cfg(feature = "egui")]
        render_egui(&mut self.egui, &mut self.window, event_handler, &commandbuffer, index);
        
        end_commandbuffer(&self.device, &commandbuffer);
        self.commandbuffer_pools.current_commandbuffer = None;
            
        Ok(())
    }
    
    pub unsafe fn recreate_swapchain(&mut self) -> DesperoResult<()> {
        self.device.device_wait_idle()?;

        self.swapchain.cleanup(&self.device, &mut *self.allocator.lock().unwrap());
        self.swapchain = Swapchain::init(
            &self.instance,
            &self.device,
            &self.window.surface,
            &self.queue_families,
            &mut *self.allocator.lock().unwrap(),
        )?;
        
        self.swapchain.create_framebuffers(&self.device, self.renderpass)?;
        for p in self.material_pipelines.values_mut() {
            p.cleanup(&self.device);
            p.recreate_pipeline(
                &self.device,
                &self.swapchain,
                &self.descriptor_pool,
                self.renderpass,
            )?;
        }
        
        #[cfg(feature = "egui")]
        self.egui.update_swapchain(
            self.swapchain.extent.width,
            self.swapchain.extent.height,
            self.swapchain.swapchain,
            *self.window.surface.get_formats(self.instance.physical_device)?.first().unwrap(),
        );
    
        Ok(())
    }
    
    pub(crate) fn fill_lightbuffer<T: Sized>(
        &mut self,
        data: &[T],
    ) -> Result<(), vk::Result>{
        self.light_buffer.fill(&self.device, &mut *self.allocator.lock().unwrap(), data)?;
        Ok(())
    }
    
    /// Function to destroy renderer. Used in [`Despero`]'s ['Drop'] function
    pub(crate) fn cleanup(&mut self, world: &mut World){
        unsafe {
            self.device.device_wait_idle().expect("Error halting device");  
            self.debug_renderer.cleanup(&self.device, &self.allocator); 
            #[cfg(feature = "egui")] 
            self.egui.destroy();
            self.descriptor_pool.cleanup(&self.device);
            self.device.destroy_buffer(self.camera_buffer.buffer, None);
            self.device.free_memory(self.camera_buffer.allocation.as_ref().unwrap().memory(), None);
            self.device.destroy_buffer(self.light_buffer.buffer, None);

            for (_, mut m) in &mut world.query::<&mut Mesh>(){    
                clear_model_buffer(&mut m.vertexbuffer, &self.device, &mut self.allocator);
                clear_model_buffer(&mut m.indexbuffer, &self.device, &mut self.allocator);
                clear_model_buffer(&mut m.instancebuffer, &self.device, &mut self.allocator);
            }
            
            self.commandbuffer_pools.cleanup(&self.device);
            for pipeline in self.material_pipelines.values() {
                pipeline.cleanup(&self.device);
            }
            self.device.destroy_render_pass(self.renderpass, None);
            self.swapchain.cleanup(&self.device, &mut *self.allocator.lock().unwrap());
            self.device.destroy_device(None);
            self.window.cleanup();
            self.instance.cleanup();
        };
    }
}

fn set_clear_values(
    color: na::Vector3<f32>
) -> [vk::ClearValue; 2] {
    [
        vk::ClearValue {
            color: vk::ClearColorValue {
                float32: na::Vector4::from([color.x, color.y, color.z, 1.0]).into(),
            },
        },
        vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {
                depth: 1.0,
                stencil: 0,
            },
        },
    ]
}

fn begin_commandbuffer(
    commandbuffer: &vk::CommandBuffer, 
    commandbuffer_pools: &mut CommandBufferPools, 
    device: &ash::Device
) -> DesperoResult<()> {
    let commandbuffer_begininfo = vk::CommandBufferBeginInfo::builder();
    commandbuffer_pools.current_commandbuffer = Some(*commandbuffer);
    unsafe { device.begin_command_buffer(*commandbuffer, &commandbuffer_begininfo)? };
    
    Ok(())
}

fn update_texture_sets(
    asset_manager: &AssetManager, 
    descriptor_pool: &DescriptorPool, 
    swapchain: &Swapchain,
    device: &ash::Device,
){
    let imageinfos = asset_manager.descriptor_image_info();
    let descriptorwrite_image = vk::WriteDescriptorSet::builder()
        .dst_set(descriptor_pool.texture_sets[swapchain.current_image])
        .dst_binding(0)
        .dst_array_element(0)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .image_info(&imageinfos)
        .build();

    unsafe { device.update_descriptor_sets(&[descriptorwrite_image], &[]); }
}

fn begin_renderpass(
    renderpass: &vk::RenderPass, 
    swapchain: &Swapchain,
    device: &ash::Device,
    commandbuffer: &vk::CommandBuffer,
    index: usize,
){
    let clear_values = set_clear_values(na::Vector3::new(0.0, 0.0, 0.0));
    
    let renderpass_begininfo = vk::RenderPassBeginInfo::builder()
        .render_pass(*renderpass)
        .framebuffer(swapchain.framebuffers[index])
        .render_area(vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: swapchain.extent,
        })
        .clear_values(&clear_values);
    
    unsafe { 
        device.cmd_begin_render_pass(
            *commandbuffer,
            &renderpass_begininfo,
            vk::SubpassContents::INLINE,
        );
    }
}

fn bind_descriptor_sets(
    device: &ash::Device,
    commandbuffer: &vk::CommandBuffer,
    descriptor_pool: &DescriptorPool,
    index: usize,
){
    unsafe {
        device.cmd_bind_descriptor_sets(
            *commandbuffer,
            vk::PipelineBindPoint::GRAPHICS,
            descriptor_pool.pipeline_layout,
            0,
            &[
                descriptor_pool.camera_sets[index],
                descriptor_pool.texture_sets[index],
                descriptor_pool.light_sets[index],
            ],
            &[],
        );
    }
}

fn bind_graphics_pipeline(
    pipelines: &PipelineCollection, 
    device: &ash::Device,
    commandbuffer: &vk::CommandBuffer,
    mat_type: &TypeId,
){
    let pipeline = pipelines.get(mat_type).unwrap();
                
    unsafe {
        device.cmd_bind_pipeline(
            *commandbuffer,
            vk::PipelineBindPoint::GRAPHICS,
            pipeline.pipeline,
        );
    }
}

fn bind_vertex_buffers(
    device: &ash::Device,
    commandbuffer: &vk::CommandBuffer,
    indexbuffer: &Buffer,
    vertexbuffer: &Buffer,
    instancebuffer: &Buffer,
){
    unsafe {
        device.cmd_bind_index_buffer(
            *commandbuffer,
            indexbuffer.buffer,
            0,
            vk::IndexType::UINT32,
        );
        
        device.cmd_bind_vertex_buffers(
            *commandbuffer,
            0,
            &[vertexbuffer.buffer],
            &[0],
        );
        
        device.cmd_bind_vertex_buffers(
            *commandbuffer,
            1,
            &[instancebuffer.buffer],
            &[0],
        );
    }
}

fn apply_transform(
    device: &ash::Device,
    descriptor_pool: &DescriptorPool,
    commandbuffer: &vk::CommandBuffer,
    transform: &Transform,
){
    let transform_matrices = transform.to_matrices();
    let transform_ptr = &transform_matrices as *const _ as *const u8;
    let transform_slice = unsafe { std::slice::from_raw_parts(transform_ptr, 128) };
    
    unsafe {
        device.cmd_push_constants(
            *commandbuffer,
            descriptor_pool.pipeline_layout,
            vk::ShaderStageFlags::VERTEX,
            0,
            transform_slice,
        );
    }
}

fn draw_mesh(
    device: &ash::Device,
    commandbuffer: &vk::CommandBuffer,
    indices_count: usize,
){
    unsafe {
        device.cmd_draw_indexed(
            *commandbuffer,
            indices_count as u32,
            1,
            0,
            0,
            0,
        );
    }
}

fn end_renderpass(device: &ash::Device, commandbuffer: &vk::CommandBuffer){
    unsafe { device.cmd_end_render_pass(*commandbuffer); }
}

fn end_commandbuffer(device: &ash::Device, commandbuffer: &vk::CommandBuffer){
    unsafe { device.end_command_buffer(*commandbuffer).expect("Failed end commandbuffer"); }
}

#[cfg(feature = "egui")]
fn render_egui(
    egui: &mut GuiHandler,
    window: &mut Window,
    event_handler: &mut EventHandler<GuiContext>,
    commandbuffer: &vk::CommandBuffer,
    index: usize,
){
    egui.context().set_visuals(egui::style::Visuals::dark());
    egui.begin_frame(&window.window.lock().unwrap());
    event_handler.send(egui.context());
    let output = egui.end_frame(&mut window.window.lock().unwrap());
    let clipped_meshes = egui.context().tessellate(output.shapes);
    egui.paint(
        *commandbuffer,
        index,
        clipped_meshes,
        output.textures_delta
    );
}

fn clear_model_buffer(
    buf: &mut Option<Buffer>,
    logical_device: &ash::Device,
    allocator: &Arc<Mutex<Allocator>>,
){
    if let Some(b) = buf {
        let mut alloc: Option<Allocation> = None;
        std::mem::swap(&mut alloc, &mut b.allocation);
        (*allocator.lock().unwrap()).free(alloc.unwrap()).unwrap();
        unsafe { logical_device.destroy_buffer(b.buffer, None) };
    }
}

unsafe impl Send for Renderer {}
unsafe impl Sync for Renderer {}
