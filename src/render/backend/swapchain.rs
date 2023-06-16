use ash::vk;
use ash::extensions::khr::Swapchain as SwapchainLoader;
use gpu_allocator::vulkan::*;
use nalgebra as na;

use crate::render::backend::{
    instance::Instance,
    surface::Surface,
    queues::QueueFamilies,
    depth_image::DepthImage,
};

type SwapchainBundle = (SwapchainLoader, vk::SwapchainKHR, [u32; 1], vk::Extent2D);
type SwapchainImages = (Vec<vk::Image>, u32, Vec<vk::ImageView>);
type SwapchainSync = (Vec<vk::Semaphore>, Vec<vk::Semaphore>, Vec<vk::Fence>);

pub struct Swapchain {
    pub swapchain_loader: SwapchainLoader,
    pub swapchain: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub imageviews: Vec<vk::ImageView>,
    pub depth_image: DepthImage,
    pub framebuffers: Vec<vk::Framebuffer>,
    pub extent: vk::Extent2D,
    pub may_begin_drawing: Vec<vk::Fence>,
    pub image_available: Vec<vk::Semaphore>,
    pub rendering_finished: Vec<vk::Semaphore>,
    pub amount_of_images: u32,
    pub current_image: usize,
    pub clear_color: na::Vector3<f32>,
}

impl Swapchain {
    /// Initialize [`Swapchain`]
    pub fn init(
        instance: &Instance,
        logical_device: &ash::Device,
        surface: &Surface,
        queue_families: &QueueFamilies,
        allocator: &mut Allocator,
        clear_color: na::Vector3<f32>,
    ) -> Result<Swapchain, vk::Result> {
        let (swapchain_loader, swapchain, queue_family_indices, extent) = unsafe {
            Self::create_swapchain(instance, logical_device, surface, queue_families)?
        };
            
        let (swapchain_images, amount_of_images, swapchain_imageviews) = unsafe {
            Self::create_swapchain_images(&swapchain_loader, &swapchain, &logical_device)?
        };
            
        let (image_available, rendering_finished, may_begin_drawing) = unsafe {
            Self::create_semaphores_and_fences(amount_of_images, &logical_device)?
        };
            
        let depth_image = DepthImage::new(&logical_device, allocator, &extent, &queue_family_indices)?;
        
        Ok(Swapchain {
            swapchain_loader,
            swapchain,
            images: swapchain_images,
            imageviews: swapchain_imageviews,
            depth_image,
            framebuffers: vec![],
            extent,
            image_available,
            rendering_finished,
            may_begin_drawing,
            amount_of_images,
            current_image: 0,
            clear_color,
        })
    }
    
    /// Create swapchain and swapchain loader
    unsafe fn create_swapchain(
        instance: &Instance,
        logical_device: &ash::Device,
        surface: &Surface,
        queue_families: &QueueFamilies,
    ) -> Result<SwapchainBundle, vk::Result> {
        let surface_capabilities = surface.get_capabilities(instance.physical_device)?;
        let extent = surface_capabilities.current_extent;
        let surface_format = *surface.get_formats(instance.physical_device)?.first().unwrap();
        let queue_family_indices = [queue_families.graphics_index.unwrap()];
        
        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface.surface)
            .min_image_count(
                3.max(surface_capabilities.max_image_count)
                    .min(surface_capabilities.min_image_count),
            )
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(surface_capabilities.current_extent)
            .image_array_layers(1)
            .image_usage(
                vk::ImageUsageFlags::COLOR_ATTACHMENT |
                vk::ImageUsageFlags::TRANSFER_SRC
            )
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&queue_family_indices)
            .pre_transform(surface_capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(vk::PresentModeKHR::FIFO);
            
        let swapchain_loader = SwapchainLoader::new(&instance.instance, &logical_device);
        let swapchain = swapchain_loader.create_swapchain(&swapchain_create_info, None)?;
        
        Ok((swapchain_loader, swapchain, queue_family_indices, extent))
    }
    
    /// Create swapchain images
    unsafe fn create_swapchain_images(
        swapchain_loader: &SwapchainLoader,
        swapchain: &vk::SwapchainKHR,
        logical_device: &ash::Device,
    ) -> Result<SwapchainImages, vk::Result> {
        let swapchain_images = swapchain_loader.get_swapchain_images(*swapchain)?;
        let amount_of_images = swapchain_images.len() as u32;
        let mut swapchain_imageviews = Vec::with_capacity(swapchain_images.len());
        
        Self::fill_imageviews(&mut swapchain_imageviews, &swapchain_images, &logical_device)?;
        
        Ok((swapchain_images, amount_of_images, swapchain_imageviews))
    }
    
    /// Fill swapchain imageviews with images
    unsafe fn fill_imageviews(
        swapchain_imageviews: &mut Vec<vk::ImageView>,
        swapchain_images: &Vec<vk::Image>,
        logical_device: &ash::Device,
    ) -> Result<(), vk::Result> {
        for image in swapchain_images {
            let subresource_range = vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1);
            let imageview_create_info = vk::ImageViewCreateInfo::builder()
                .image(*image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(vk::Format::B8G8R8A8_SRGB)
                .subresource_range(*subresource_range);
            let imageview = logical_device.create_image_view(&imageview_create_info, None)?;
            swapchain_imageviews.push(imageview);
        }
        
        Ok(())
    }
    
    /// Create semaphores and fences for swapchain to control synchronization
    unsafe fn create_semaphores_and_fences(
        amount_of_images: u32,
        logical_device: &ash::Device,
    ) -> Result<SwapchainSync, vk::Result>{
        let mut image_available = vec![];
        let mut rendering_finished = vec![];
        let mut may_begin_drawing = vec![];
        
        let semaphoreinfo = vk::SemaphoreCreateInfo::builder();
        let fenceinfo = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
        for _ in 0..amount_of_images {
            let semaphore_available = logical_device.create_semaphore(&semaphoreinfo, None)?;
            let semaphore_finished = logical_device.create_semaphore(&semaphoreinfo, None)?;
            image_available.push(semaphore_available);
            rendering_finished.push(semaphore_finished);
            let fence = logical_device.create_fence(&fenceinfo, None)?;
            may_begin_drawing.push(fence);
        }
        
        Ok((image_available, rendering_finished, may_begin_drawing))
    }
    
    /// Create swapchain framebuffers
    pub fn create_framebuffers(
        &mut self,
        logical_device: &ash::Device,
        renderpass: vk::RenderPass,
    ) -> Result<(), vk::Result> {
        for iv in &self.imageviews {
            let iview = [*iv, self.depth_image.depth_imageview];
            let framebuffer_info = vk::FramebufferCreateInfo::builder()
                .render_pass(renderpass)
                .attachments(&iview)
                .width(self.extent.width)
                .height(self.extent.height)
                .layers(1);
            let fb = unsafe { logical_device.create_framebuffer(&framebuffer_info, None) }?;
            self.framebuffers.push(fb);
        }
        Ok(())
    }
    
    /// Get swapchain's count of framebuffers
    pub fn framebuffers_count(&self) -> usize {
        self.framebuffers.len()
    }
    
    /// Destroy [`Swapchain`]
    pub unsafe fn cleanup(&mut self, logical_device: &ash::Device, allocator: &mut Allocator) {
        self.depth_image.cleanup(&logical_device, allocator);
        for fence in &self.may_begin_drawing {
            logical_device.destroy_fence(*fence, None);
        }

        for semaphore in &self.image_available {
            logical_device.destroy_semaphore(*semaphore, None);
        }
        
        for semaphore in &self.rendering_finished {
            logical_device.destroy_semaphore(*semaphore, None);
        }

        for iv in &self.imageviews {
            logical_device.destroy_image_view(*iv, None);
        }
        
        for fb in &self.framebuffers {
            logical_device.destroy_framebuffer(*fb, None);
        }
        
        self.swapchain_loader.destroy_swapchain(self.swapchain, None)
    }
}
