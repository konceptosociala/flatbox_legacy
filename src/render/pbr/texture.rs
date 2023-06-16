use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use gpu_allocator::vulkan::*;
use gpu_allocator::MemoryLocation;
use ash::vk;

use crate::render::{
    backend::buffer::Buffer,
    renderer::Renderer,
};

use crate::error::*;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum Filter {
    #[default]
    Linear,
    Nearest,
    Cubic,
} 

impl From<Filter> for vk::Filter {
    fn from(filter: Filter) -> Self {
        match filter {
            Filter::Linear => vk::Filter::LINEAR,
            Filter::Nearest => vk::Filter::NEAREST,
            Filter::Cubic => vk::Filter::CUBIC_EXT,
        }
    }
}

#[allow(dead_code)]
#[derive(Default, Serialize, Deserialize)]
pub struct Texture {
    pub path: PathBuf,
    pub filter: Filter,
    
    // Raw image data
    #[serde(skip_serializing, skip_deserializing)]
    pub(crate) image: Option<image::RgbaImage>,
    
    // Vulkan image data
    #[serde(skip_serializing, skip_deserializing)]
    pub(crate) vk_image: Option<vk::Image>,
    
    // Data allocation
    #[serde(skip_serializing, skip_deserializing)]
    pub(crate) image_allocation: Option<Allocation>,
    
    // Vulkan image view
    #[serde(skip_serializing, skip_deserializing)]
    pub(crate) imageview: Option<vk::ImageView>,
    
    // Vulkan sampler
    #[serde(skip_serializing, skip_deserializing)]
    pub(crate) sampler: Option<vk::Sampler>,
}

impl std::fmt::Debug for Texture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Texture")
            .field("path", &self.path)
            .field("filter", &self.filter)
            .finish()
    }
}

impl Texture {
    /// Create empty texture with given `filter` and `path`. Requires [`generate`](#method.generate) method applied to it to load the texture to memory
    pub fn new_blank(
        path: &'static str, 
        filter: Filter,
    ) -> Self {
        Texture {
            path: path.into(),
            filter,
            image: None,
            vk_image: None,
            image_allocation: None,
            imageview: None,
            sampler: None,
        }
    }
    
    /// Create texture from file and load it to memory
    pub fn new_from_file(
        path: &'static str, 
        filter: Filter,
        renderer: &mut Renderer
    ) -> DesperoResult<Self> {
        let mut texture = Texture::new_blank(path, filter);
        texture.generate(renderer)?;
        
        Ok(texture)
    }

    /// Generate texture rendering data for blank texture
    pub fn generate(&mut self, renderer: &mut Renderer) -> DesperoResult<()> {
        let image = image::open(self.path.clone())
            .map(|img| img.to_rgba8())
            .expect("unable to open image");
        
        self.generate_from(renderer, image)?;
        
        Ok(())
    }

    pub(crate) fn generate_from(
        &mut self, 
        renderer: &mut Renderer,
        image: image::RgbaImage,
    ) -> DesperoResult<()>{
        let raw_filter: vk::Filter = self.filter.clone().into();
            
        let (width, height) = image.dimensions();

        unsafe { renderer.device.device_wait_idle()?; }
        
        let img_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .format(vk::Format::R8G8B8A8_SRGB)
            .samples(vk::SampleCountFlags::TYPE_1)
            .usage(
                vk::ImageUsageFlags::TRANSFER_DST |
                vk::ImageUsageFlags::SAMPLED
            );
        let vk_image = unsafe { renderer.device.create_image(&img_create_info, None)? };
        // Allocation info
        let allocation_info = &AllocationCreateDesc {
            name: "Texture allocation",
            requirements: unsafe { renderer.device.get_image_memory_requirements(vk_image) },
            location: MemoryLocation::GpuOnly,
            linear: true,
        };
        // Create memory allocation
        let allocation = renderer.allocator.lock().unwrap().allocate(allocation_info).unwrap();
        // Bind memory allocation to the vk_image
        unsafe { renderer.device.bind_image_memory(
            vk_image, 
            allocation.memory(), 
            allocation.offset()).unwrap()
        };
        
        // Create ImageView
        let view_create_info = vk::ImageViewCreateInfo::builder()
            .image(vk_image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vk::Format::R8G8B8A8_SRGB)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                level_count: 1,
                layer_count: 1,
                ..Default::default()
            });
        let imageview = unsafe { renderer.device.create_image_view(&view_create_info, None)? };
        
        // Create Sampler
        let sampler_info = vk::SamplerCreateInfo::builder()
            .mag_filter(raw_filter)
            .min_filter(raw_filter);
        let sampler = unsafe { renderer.device.create_sampler(&sampler_info, None)? };
        
        // Prepare buffer for the texture
        let data = image.clone().into_raw();
        let mut buffer = Buffer::new(
            &renderer.device,
            &mut *renderer.allocator.lock().unwrap(),
            data.len() as u64,
            vk::BufferUsageFlags::TRANSFER_SRC,
            MemoryLocation::CpuToGpu,
            "Texture allocation"
        )?;
        buffer.fill(&renderer.device, &mut *renderer.allocator.lock().unwrap(), &data)?;
        
        // Create CommandBuffer
        let commandbuf_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(renderer.commandbuffer_pools.commandpool_graphics)
            .command_buffer_count(1);
        let copycmdbuffer = unsafe {
            renderer.device.allocate_command_buffers(&commandbuf_allocate_info)
        }
        .unwrap()[0];

        let cmdbegininfo = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            
        // Begin CommandBuffer
        unsafe {
            renderer.device.begin_command_buffer(copycmdbuffer, &cmdbegininfo)
        }?;
        
        // Change image layout for transfering
        let barrier = vk::ImageMemoryBarrier::builder()
            .image(vk_image)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })
            .build();
            
        unsafe {
            renderer.device.cmd_pipeline_barrier(
                copycmdbuffer,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            )
        };
        
        // Copy data from the buffer to the image
        let image_subresource = vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: 1,
        };
        
        let region = vk::BufferImageCopy {
            buffer_offset: 0,
            buffer_row_length: 0,
            buffer_image_height: 0,
            image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
            image_extent: vk::Extent3D {
                width,
                height,
                depth: 1,
            },
            image_subresource,
            ..Default::default()
        };
        
        unsafe {
            renderer.device.cmd_copy_buffer_to_image(
                copycmdbuffer,
                buffer.buffer,
                vk_image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[region],
            );
        }
        
        // Change image layout for fragment shader
        
        let barrier = vk::ImageMemoryBarrier::builder()
            .image(vk_image)
            .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ)
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })
            .build();
            
        unsafe {
            renderer.device.cmd_pipeline_barrier(
                copycmdbuffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            )
        };
        
        // End CommandBuffer
        unsafe { renderer.device.end_command_buffer(copycmdbuffer) }?;
        let submit_infos = [vk::SubmitInfo::builder()
            .command_buffers(&[copycmdbuffer])
            .build()];
        let fence = unsafe {
            renderer.device.create_fence(&vk::FenceCreateInfo::default(), None)
        }?;
        
        unsafe {
            renderer.device.queue_submit(renderer.queue_families.graphics_queue, &submit_infos, fence)
        }?;
        
        // Destroy buffer
        unsafe { renderer.device.wait_for_fences(&[fence], true, std::u64::MAX) }?;
        unsafe { renderer.device.destroy_fence(fence, None) };
                
        let mut alloc: Option<Allocation> = None;
        std::mem::swap(&mut alloc, &mut buffer.allocation);
        let alloc = alloc.unwrap();
        renderer.allocator.lock().unwrap().free(alloc).unwrap();
        unsafe { renderer.device.destroy_buffer(buffer.buffer, None) };
        
        unsafe {
            renderer.device.free_command_buffers(
                renderer.commandbuffer_pools.commandpool_graphics,
                &[copycmdbuffer]
            )
        };
        
        self.image = Some(image);
        self.vk_image = Some(vk_image);
        self.image_allocation = Some(allocation);
        self.imageview = Some(imageview);
        self.sampler = Some(sampler);

        Ok(())
    }
    
    pub fn cleanup(&mut self, renderer: &mut Renderer) {

        if let Some(_) = self.image_allocation {
            let mut new_alloc: Option<Allocation> = None;
            std::mem::swap(&mut new_alloc, &mut self.image_allocation);
            let new_alloc = new_alloc.unwrap();
            (*renderer.allocator.lock().unwrap()).free(new_alloc).unwrap();
        }
        
        if let Some(sampler) = self.sampler {
            unsafe { renderer.device.destroy_sampler(sampler, None); }
            self.sampler = None;
        }
        
        if let Some(imageview) = self.imageview {
            unsafe { renderer.device.destroy_image_view(imageview, None); }
            self.imageview = None;
        }    

        if let Some(vk_image) = self.vk_image {
            unsafe { renderer.device.destroy_image(vk_image, None); }
            self.vk_image = None;
        }    
    }
}
