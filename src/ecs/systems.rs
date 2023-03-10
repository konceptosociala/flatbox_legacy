use ash::vk;
use std::mem::{size_of, size_of_val};
use gpu_allocator::MemoryLocation;

use crate::ecs::*;
use crate::physics::*;
use crate::error::DesperoResult;
use crate::math::transform::Transform;
use crate::render::{
    renderer::Renderer,
    pbr::{
        camera::Camera,
        model::*,
        light::*,
        material::*,
    },
    backend::{
        buffer::Buffer,
        swapchain::Swapchain,
    },
    gui::GuiContext,
};

pub(crate) fn rendering_system(
    mut physics_handler: Write<PhysicsHandler>,
    mut egui_ctx: Write<EventHandler<GuiContext>>,
    mut renderer: Write<Renderer>,
    mut model_world: SubWorld<(&mut Mesh, &mut MaterialHandle, &mut Transform)>,
    camera_world: SubWorld<(&mut Camera, &Transform)>,
) -> DesperoResult<()> {
    let image_index = get_image_index(&renderer.swapchain)?;
    
    check_fences(&renderer.device, &renderer.swapchain)?;
    
    for (_, camera) in &mut camera_world.query::<&mut Camera>(){
        if camera.is_active {        
            camera.update_buffer(&mut renderer)?;
        }
    }    

    unsafe { 
        renderer.update_commandbuffer(
            &mut model_world,
            &mut egui_ctx,
            &mut physics_handler,
            image_index as usize,
        )? 
    };
    
    // Submit commandbuffers
    let semaphores_available = [renderer.swapchain.image_available[renderer.swapchain.current_image]];
    let waiting_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
    let semaphores_finished = [renderer.swapchain.rendering_finished[renderer.swapchain.current_image]];
    let commandbuffers = [*renderer.commandbuffer_pools.get_commandbuffer(image_index as usize).unwrap()];
    let submit_info = [
        vk::SubmitInfo::builder()
            .wait_semaphores(&semaphores_available)
            .wait_dst_stage_mask(&waiting_stages)
            .command_buffers(&commandbuffers)
            .signal_semaphores(&semaphores_finished)
            .build()
    ];
    
    unsafe {
        renderer
            .device
            .queue_submit(
                renderer.queue_families.graphics_queue,
                &submit_info,
                renderer.swapchain.may_begin_drawing[renderer.swapchain.current_image],
            )
            .expect("queue submission");
    };
    
    let swapchains = [renderer.swapchain.swapchain];
    let indices = [image_index];
    let present_info = vk::PresentInfoKHR::builder()
        .wait_semaphores(&semaphores_finished)
        .swapchains(&swapchains)
        .image_indices(&indices);
    unsafe {
        if renderer
            .swapchain
            .swapchain_loader
            .queue_present(renderer.queue_families.graphics_queue, &present_info)
            .expect("Queue presentation error")
        {
            renderer.recreate_swapchain().expect("Cannot recreate swapchain");
            
            for (_, mut camera) in &mut camera_world.query::<&mut Camera>(){
                if camera.is_active {
                    camera.set_aspect(
                        renderer.swapchain.extent.width as f32
                            / renderer.swapchain.extent.height as f32,
                    );

                    camera.update_buffer(&mut renderer).expect("Cannot update camera buffer");
                }
            }
        }
    };
    // Set swapchain image
    renderer.swapchain.current_image =
        (renderer.swapchain.current_image + 1) % renderer.swapchain.amount_of_images as usize;
        
    Ok(())
}

pub(crate) fn update_models_system(
    mut renderer: Write<Renderer>,
    world: SubWorld<(&mut Mesh, &mut MaterialHandle, &Transform)>,
) -> DesperoResult<()> {
    for (_, (mut mesh, handle, _)) in &mut world.query::<(
        &mut Mesh, &MaterialHandle, &Transform,
    )>(){
        let material = renderer.materials.get(handle.get()).unwrap().clone();
        let logical_device = renderer.device.clone();
        let allocator = &mut renderer.allocator;
        let vertexdata = mesh.vertexdata.clone();
        let indexdata = mesh.indexdata.clone();

        if let Some(buffer) = &mut mesh.vertexbuffer {
            buffer.fill(
                &logical_device,
                &mut *allocator.lock().unwrap(),
                &vertexdata
            )?;
        } else {
            let bytes = (mesh.vertexdata.len() * size_of::<Vertex>()) as u64;        
            let mut buffer = Buffer::new(
                &logical_device,
                &mut *allocator.lock().unwrap(),
                bytes,
                vk::BufferUsageFlags::VERTEX_BUFFER,
                MemoryLocation::CpuToGpu,
                "Model vertex buffer"
            )?;
            
            buffer.fill(
                &logical_device,
                &mut *allocator.lock().unwrap(),
                &indexdata
            )?;
            mesh.vertexbuffer = Some(buffer);
        }
    
        let mat_ptr = &*material as *const _ as *const u8;
        let mat_slice = unsafe {std::slice::from_raw_parts(mat_ptr, size_of_val(&*material))};
        if let Some(buffer) = &mut mesh.instancebuffer {
            buffer.fill(
                &logical_device,
                &mut *allocator.lock().unwrap(),
                mat_slice,
            )?;
        } else {
            let bytes = size_of_val(&material) as u64; 
            let mut buffer = Buffer::new(
                &logical_device,
                &mut *allocator.lock().unwrap(),
                bytes,
                vk::BufferUsageFlags::VERTEX_BUFFER,
                MemoryLocation::CpuToGpu,
                "Model instance buffer"
            )?;
            
            buffer.fill(
                &logical_device,
                &mut *allocator.lock().unwrap(),
                mat_slice
            )?;
            mesh.instancebuffer = Some(buffer);
        }

        if let Some(buffer) = &mut mesh.indexbuffer {
            buffer.fill(
                &logical_device,
                &mut *allocator.lock().unwrap(),
                &indexdata,
            )?;
        } else {
            let bytes = (mesh.indexdata.len() * size_of::<u32>()) as u64;        
            let mut buffer = Buffer::new(
                &logical_device,
                &mut *allocator.lock().unwrap(),
                bytes,
                vk::BufferUsageFlags::INDEX_BUFFER,
                MemoryLocation::CpuToGpu,
                "Model buffer of vertex indices"
            )?;
            
            buffer.fill(
                &logical_device,
                &mut *allocator.lock().unwrap(),
                &indexdata,
            )?;
            mesh.indexbuffer = Some(buffer);
        }
    }
    
    Ok(())
}

pub(crate) fn update_lights(
    plight_world: SubWorld<&PointLight>,
    dlight_world: SubWorld<&DirectionalLight>,
    mut renderer: Write<Renderer>,
) -> DesperoResult<()> {
    let directional_lights = dlight_world.query::<&DirectionalLight>()
        .into_iter()
        .map(|(_, l)| l.clone())
        .collect::<Vec<DirectionalLight>>();
        
    let point_lights = plight_world.query::<&PointLight>()
        .into_iter()
        .map(|(_, l)| l.clone())
        .collect::<Vec<PointLight>>();
    
    let mut data = vec![
        directional_lights.len() as f32,
        point_lights.len() as f32,
        0.0,
        0.0,
    ];
    
    for dl in directional_lights {
        data.push(dl.direction.x);
        data.push(dl.direction.y);
        data.push(dl.direction.z);
        data.push(0.0);
        data.push(dl.illuminance[0]);
        data.push(dl.illuminance[1]);
        data.push(dl.illuminance[2]);
        data.push(0.0);
    }
    for pl in point_lights {
        data.push(pl.position.x);
        data.push(pl.position.y);
        data.push(pl.position.z);
        data.push(0.0);
        data.push(pl.luminous_flux[0]);
        data.push(pl.luminous_flux[1]);
        data.push(pl.luminous_flux[2]);
        data.push(0.0);
    }
    renderer.fill_lightbuffer(&data)?;

    for descset in &renderer.descriptor_pool.light_sets {
        let buffer_infos = [vk::DescriptorBufferInfo {
            buffer: renderer.light_buffer.buffer,
            offset: 0,
            range: 4 * data.len() as u64,
        }];
        let desc_sets_write = [vk::WriteDescriptorSet::builder()
            .dst_set(*descset)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .buffer_info(&buffer_infos)
            .build()];
        unsafe { renderer.device.update_descriptor_sets(&desc_sets_write, &[]) };
    }
    Ok(())
}

pub(crate) fn update_physics(
    mut physics_handler: Write<PhysicsHandler>,
    physics_world: SubWorld<(&mut Transform, &BodyHandle)>,
    added_world: SubWorld<(&Transform, &BodyHandle, Added<BodyHandle>)>,
) -> DesperoResult<()> {    
    for (_, (transform, handle, added)) in &mut added_world.query::<(
        &Transform, &BodyHandle, Added<BodyHandle>
    )>(){
        if added {                        
            let rigidbody = physics_handler.rigidbody_mut(*handle)?;
            rigidbody.set_translation(transform.translation, false);
            rigidbody.set_rotation(transform.rotation, false);          
        }
    }
    
    physics_handler.step();
    
    for (_, (mut transform, handle)) in &mut physics_world.query::<(
        &mut Transform, &BodyHandle,
    )>(){
        let rigidbody = physics_handler.rigidbody(*handle)?;
        transform.translation = *rigidbody.translation();
        transform.rotation = *rigidbody.rotation();        
    }
    
    Ok(())
}

fn get_image_index(swapchain: &Swapchain) -> DesperoResult<u32> {
    let (image_index, _) = unsafe {
        swapchain
            .swapchain_loader
            .acquire_next_image(
                swapchain.swapchain,
                std::u64::MAX,
                swapchain.image_available[swapchain.current_image],
                vk::Fence::null(),
            )?
    };
    Ok(image_index)
}

fn check_fences(
    logical_device: &ash::Device,
    swapchain: &Swapchain
) -> DesperoResult<()> {
    unsafe {
        logical_device
            .wait_for_fences(
                &[swapchain.may_begin_drawing[swapchain.current_image]],
                true,
                std::u64::MAX,
            )?;
            
        logical_device
            .reset_fences(&[
                swapchain.may_begin_drawing[swapchain.current_image]
            ])?;
    }
    
    Ok(())
}
