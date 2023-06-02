/*! 

# Default systems

Here are systems that the engine uses to process the main 
components of the game: rendering, audio, physics, etc.

You can enable their use with [`Despero::default_systems`] 

```rust
use despero::prelude::*;

Despero::init(WindowBuilder::default())
    .default_systems()
    .run();
```

or add necessary ones manually with [`Despero::add_system`]

```rust
use despero::prelude::*;

Despero::init(WindowBuilder::default())
    .add_system(time_system)
    .add_system(update_physics)
    .run();
```

*/

#[allow(unused_imports)]
use crate::Despero;

#[cfg(feature = "render")]
use {
    ash::vk,
    std::mem::{size_of, size_of_val},
    gpu_allocator::MemoryLocation,
};

#[cfg(feature = "render")]
use crate::{
    assets::*,
    render::{
        renderer::Renderer,
        pbr::{
            camera::Camera,
            model::*,
            light::*,
        },
        backend::{
            buffer::Buffer,
            swapchain::Swapchain,
        },
    },
};

use crate::audio::*;
use crate::time::*;
use crate::ecs::*;
use crate::physics::*;
use crate::error::DesperoResult;
use crate::math::transform::Transform;

#[cfg(feature = "egui")]
use crate::render::ui::GuiContext;

pub fn main_setup(){}

pub fn time_system(
    mut time: Write<Time>,
){
    time.update();
}

pub fn processing_audio(
    cast_world: SubWorld<(&Transform, &mut AudioCast)>,
    listener_world: SubWorld<(&Transform, &mut AudioListener)>
) -> DesperoResult<()> {
    for (_, (t, mut c)) in &mut cast_world.query::<(&Transform, &mut AudioCast)>(){
        c.set_transform(&t)?;
    }

    for (_, (t, mut l)) in &mut listener_world.query::<(&Transform, &mut AudioListener)>(){
        l.set_transform(&t)?;
    }

    Ok(())
}

pub fn update_physics(
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

#[cfg(feature = "render")]
pub fn generate_textures(
    mut asset_manager: Write<AssetManager>,
    mut renderer: Write<Renderer>,
) -> DesperoResult<()> {
    for texture in &mut asset_manager.textures {
        if texture.sampler == None {
            texture.generate(&mut renderer)?;
        }
    }
    
    Ok(())
}

#[cfg(feature = "render")]
pub fn rendering_system(
    mut physics_handler: Write<PhysicsHandler>,
    #[cfg(feature = "egui")]
    events: Read<Events>,
    mut renderer: Write<Renderer>,
    asset_manager: Read<AssetManager>,
    mut model_world: SubWorld<(&mut Mesh, &mut AssetHandle, &mut Transform)>,
    camera_world: SubWorld<(&mut Camera, &Transform)>,
) -> DesperoResult<()> {
    let image_index = get_image_index(&renderer.swapchain)?;
    
    check_fences(&renderer.device, &renderer.swapchain)?;
    
    for (_, (camera, transform)) in &mut camera_world.query::<(&Camera, &Transform)>(){
        if camera.is_active() {      
            camera.update_buffer(&mut renderer, &transform)?;
        }
    }    

    #[cfg(feature = "egui")]
    let mut egui_ctx = events.get_handler::<EventHandler<GuiContext>>().unwrap();

    renderer.update_commandbuffer(
        &mut model_world,
        #[cfg(feature = "egui")]
        &mut egui_ctx,
        &mut physics_handler,
        &asset_manager,
        image_index as usize,
    )?;
    
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
            )?
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
            .queue_present(renderer.queue_families.graphics_queue, &present_info)?
        {
            renderer.recreate_swapchain()?;
            
            for (_, (mut camera, transform)) in &mut camera_world.query::<(&mut Camera, &Transform)>(){
                if camera.is_active() {
                    camera.set_aspect(
                        renderer.swapchain.extent.width as f32
                            / renderer.swapchain.extent.height as f32,
                    );

                    camera.update_buffer(&mut renderer, &transform)?;
                }
            }
        }
    };

    renderer.swapchain.current_image =
        (renderer.swapchain.current_image + 1) % renderer.swapchain.amount_of_images as usize;
        
    Ok(())
}

#[cfg(feature = "render")]
pub fn update_models_system(
    mut renderer: Write<Renderer>,
    asset_manager: Read<AssetManager>,
    world: SubWorld<(&mut Mesh, &mut AssetHandle, &Transform)>,
) -> DesperoResult<()> {
    for (_, (mut mesh, handle, _)) in &mut world.query::<(
        &mut Mesh, &AssetHandle, &Transform,
    )>(){
        let material = asset_manager.get_material(*handle).unwrap();
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
    
        let mat_ptr = &**(material) as *const _ as *const u8;
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

#[cfg(feature = "render")]
pub fn update_lights(
    plight_world: SubWorld<(&PointLight, Changed<PointLight>)>,
    dlight_world: SubWorld<(&DirectionalLight, Changed<DirectionalLight>)>,
    mut renderer: Write<Renderer>,
) -> DesperoResult<()> {
    let directional_lights = dlight_world.query::<(&DirectionalLight, Changed<DirectionalLight>)>()
        .into_iter()
        .filter_map(|(_, (light, is_changed))| if is_changed { Some(light.clone()) } else { None })
        .collect::<Vec<DirectionalLight>>();
    
    let point_lights = plight_world.query::<(&PointLight, Changed<PointLight>)>()
        .into_iter()
        .filter_map(|(_, (light, is_changed))| if is_changed { Some(light.clone()) } else { None })
        .collect::<Vec<PointLight>>();
        
    if directional_lights.is_empty() && point_lights.is_empty() {
        return Ok(());
    }
    
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

#[cfg(feature = "render")]
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

#[cfg(feature = "render")]
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
