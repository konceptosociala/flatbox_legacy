use std::mem::ManuallyDrop;
use std::sync::{Arc, Mutex};
use std::any::TypeId;
use std::collections::HashMap;
use ash::vk;
use ash::Device;
use gpu_allocator::vulkan::*;
use gpu_allocator::MemoryLocation;
use nalgebra as na;
#[cfg(feature = "winit")]
use winit::{
    window::WindowBuilder,
};
#[cfg(feature = "egui")]
use egui_winit_ash_integration::*;

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
        texture::*,
        material::*,
    },
};
#[cfg(feature = "egui")]
use crate::render::gui::GuiHandler;

use crate::physics::{
    physics_handler::PhysicsHandler,
    debug_render::*,
};
use crate::math::transform::Transform;
use crate::ecs::event::EventWriter;
use crate::error::DesperoResult;

#[cfg(all(feature = "gtk", feature = "winit"))]
compile_error!("cannot use \"gtk\" and \"winit\" at the same time!");

#[cfg(all(feature = "egui", not(feature = "winit")))]
compile_error!("cannot use \"egui\" without \"winit\"!");

/// Maximum number of textures, which can be pushed to descriptor sets
pub const MAX_NUMBER_OF_TEXTURES: u32 = 2;

/// Main rendering collection, including Vulkan components
pub struct Renderer {
    pub(crate) instance: Instance,
    pub(crate) window: Window,
    pub(crate) queue_families: QueueFamilies,
    pub(crate) device: Device,
    pub(crate) swapchain: Swapchain,
    pub(crate) renderpass: vk::RenderPass,
    pub(crate) material_pipelines: HashMap<TypeId, Pipeline>,
    pub(crate) debug_renderer: DebugRenderer,
    pub(crate) commandbuffer_pools: CommandBufferPools,
    pub(crate) allocator: Arc<Mutex<Allocator>>,
    pub(crate) camera_buffer: Buffer,
    pub(crate) light_buffer: Buffer,
    pub(crate) descriptor_pool: DescriptorPool,
    pub(crate) texture_storage: TextureStorage,
    pub(crate) materials: MaterialStorage,
    #[cfg(feature = "egui")]
    pub(crate) egui: GuiHandler,
}

impl Renderer {    
    pub(crate) fn init(
        #[cfg(feature = "gtk")]
        window_builder: gtk::GLArea,
        
        #[cfg(feature = "winit")]
        window_builder: WindowBuilder,
    ) -> DesperoResult<Renderer> {
        let instance = Instance::init()?;
        let window = Window::init(&instance, window_builder)?;
        let (device, queue_families) = QueueFamilies::init(&instance, &window)?;
            
        let mut allocator = Allocator::new(&AllocatorCreateDesc {
            instance: instance.instance.clone(),
            device: device.clone(),
            physical_device: instance.physical_device.clone(),
            debug_settings: Default::default(),
            buffer_device_address: true,
        }).expect("Cannot create allocator");
        
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
            material_pipelines: HashMap::new(),
            debug_renderer,
            commandbuffer_pools,
            allocator,
            camera_buffer,
            light_buffer,
            descriptor_pool,
            texture_storage: TextureStorage::new(),
            materials: vec![],
            #[cfg(feature = "egui")]
            egui,
        })
    }
    
    pub fn create_texture<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
        filter: Filter,
    ) -> usize {
        self.texture_storage.new_texture_from_file(
            path,
            filter,
            &self.device,
            &mut *self.allocator.lock().unwrap(),
            &self.commandbuffer_pools.commandpool_graphics,
            &self.queue_families.graphics_queue,
        ).expect("Cannot create texture")
    }
    
    pub fn create_material<M: Material + Send + Sync>(
        &mut self,
        material: M,
    ) -> MaterialHandle {
        let index = self.materials.len();
        self.materials.push(Arc::new(material));
        return MaterialHandle::new(index);
    }
    
    pub fn bind_material<M: Material + Sync + Send>(&mut self){
        if self.material_pipelines.contains_key(&TypeId::of::<M>()) {
            log::error!("Material type '{}' is already bound!", std::any::type_name::<M>());
        } else {
            self.material_pipelines.insert(TypeId::of::<M>(), M::pipeline(&self));
        }
    }
    
    pub(crate) unsafe fn update_commandbuffer<W: borrow::ComponentBorrow>(
        &mut self,
        world: &mut SubWorld<W>,
        event_writer: &mut EventWriter,
        physics_handler: &mut PhysicsHandler,
        index: usize,
    ) -> DesperoResult<()> {
        let imageinfos = self.texture_storage.get_descriptor_image_info();
        let descriptorwrite_image = vk::WriteDescriptorSet::builder()
            .dst_set(self.descriptor_pool.texture_sets[self.swapchain.current_image])
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&imageinfos)
            .build();

        self.device.update_descriptor_sets(&[descriptorwrite_image], &[]);
        
        let commandbuffer = *self.commandbuffer_pools.get_commandbuffer(index).unwrap();
        let commandbuffer_begininfo = vk::CommandBufferBeginInfo::builder();
        
        self.commandbuffer_pools.current_commandbuffer = Some(commandbuffer);
        
        let clear_values = Self::set_clear_values(na::Vector3::new(0.0, 0.0, 0.0));
        
        let renderpass_begininfo = vk::RenderPassBeginInfo::builder()
            .render_pass(self.renderpass)
            .framebuffer(self.swapchain.framebuffers[index])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent,
            })
            .clear_values(&clear_values);
        
        self.device.begin_command_buffer(commandbuffer, &commandbuffer_begininfo)?;
        
        self.device.cmd_begin_render_pass(
            commandbuffer,
            &renderpass_begininfo,
            vk::SubpassContents::INLINE,
        );
        
        self.device.cmd_bind_descriptor_sets(
            commandbuffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.descriptor_pool.pipeline_layout,
            0,
            &[
                self.descriptor_pool.camera_sets[index],
                self.descriptor_pool.texture_sets[index],
                self.descriptor_pool.light_sets[index],
            ],
            &[],
        );
        
        for material_type in self.material_pipelines.keys() {
            let pipeline = self.material_pipelines.get(&material_type).unwrap();
                        
            self.device.cmd_bind_pipeline(
                commandbuffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.pipeline,
            );
            
            for (_, (mesh, handle, transform)) in &mut world.query::<(
                &Mesh, &MaterialHandle, &Transform,
            )>(){
                if let (
                    Some(vertexbuffer), 
                    Some(instancebuffer), 
                    Some(indexbuffer),
                ) = (
                    &mesh.vertexbuffer, 
                    &mesh.instancebuffer, 
                    &mesh.indexbuffer
                ){
                    let material = &self.materials[handle.get()];
                    if (**material).type_id() == *material_type {
                        self.device.cmd_bind_index_buffer(
                            commandbuffer,
                            indexbuffer.buffer,
                            0,
                            vk::IndexType::UINT32,
                        );
                        
                        self.device.cmd_bind_vertex_buffers(
                            commandbuffer,
                            0,
                            &[vertexbuffer.buffer],
                            &[0],
                        );
                        
                        self.device.cmd_bind_vertex_buffers(
                            commandbuffer,
                            1,
                            &[instancebuffer.buffer],
                            &[0],
                        );
                        
                        let transform_matrices = transform.to_matrices();
                        let transform_ptr = &transform_matrices as *const _ as *const u8;
                        let transform_slice = std::slice::from_raw_parts(transform_ptr, 128);
                        self.device.cmd_push_constants(
                            commandbuffer,
                            self.descriptor_pool.pipeline_layout,
                            vk::ShaderStageFlags::VERTEX,
                            0,
                            transform_slice,
                        );
                        
                        self.device.cmd_draw_indexed(
                            commandbuffer,
                            mesh.indexdata.len() as u32,
                            1,
                            0,
                            0,
                            0,
                        );                                    
                    }
                }
            }
        }
        
        physics_handler.debug_render(self);
        
        self.device.cmd_end_render_pass(commandbuffer);
        
        #[cfg(feature = "egui")]
        {
            self.egui.context().set_visuals(egui::style::Visuals::dark());
            self.egui.begin_frame(&self.window.window);
            event_writer.send(Arc::new(self.egui.context()))?;
            let output = self.egui.end_frame(&mut self.window.window);
            let clipped_meshes = self.egui.context().tessellate(output.shapes);
            self.egui.paint(
                commandbuffer,
                index,
                clipped_meshes,
                output.textures_delta
            );
        }
        
        self.device.end_command_buffer(commandbuffer)?;
        
        self.commandbuffer_pools.current_commandbuffer = None;
            
        Ok(())
    }
    
    pub(crate) unsafe fn recreate_swapchain(&mut self) -> DesperoResult<()> {
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
            self.texture_storage.cleanup(&self.device, &mut *self.allocator.lock().unwrap());
            self.descriptor_pool.cleanup(&self.device);
            self.device.destroy_buffer(self.camera_buffer.buffer, None);
            self.device.free_memory(self.camera_buffer.allocation.as_ref().unwrap().memory(), None);
            self.device.destroy_buffer(self.light_buffer.buffer, None);

            for (_, mut m) in &mut world.query::<&mut Mesh>(){    
                Self::clear_model_buffer(&mut m.vertexbuffer, &self.device, &mut self.allocator);
                Self::clear_model_buffer(&mut m.indexbuffer, &self.device, &mut self.allocator);
                Self::clear_model_buffer(&mut m.instancebuffer, &self.device, &mut self.allocator);
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
}

unsafe impl Send for Renderer {}
unsafe impl Sync for Renderer {}
