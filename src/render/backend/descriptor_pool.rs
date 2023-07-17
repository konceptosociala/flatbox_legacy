use ash::vk;

use crate::render::backend::{
    swapchain::Swapchain,
    buffer::Buffer,
};

pub struct DescriptorPool {
    pub descriptor_pool: vk::DescriptorPool,
    pub camera_sets: Vec<vk::DescriptorSet>, 
    pub texture_sets: Vec<vk::DescriptorSet>,
    pub light_sets: Vec<vk::DescriptorSet>,
    pub skybox_sets: Vec<vk::DescriptorSet>,
    pub set_layouts: Vec<vk::DescriptorSetLayout>,
    pub pipeline_layout: vk::PipelineLayout,
}

impl DescriptorPool {
    pub unsafe fn init(
        logical_device: &ash::Device,
        swapchain: &Swapchain,
        max_sampler_count: u32,
    ) -> Result<DescriptorPool, vk::Result> {        
        let descriptor_pool = Self::create_descriptor_pool(&logical_device, &swapchain, max_sampler_count)?;
        
        let camera_set_layout = Self::create_descriptor_set_layout(
            &logical_device,
            vk::DescriptorType::UNIFORM_BUFFER,
            vk::ShaderStageFlags::VERTEX,
            0,
            1,
        )?;
            
        let texture_set_layout = Self::create_descriptor_set_layout(
            &logical_device,
            vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            vk::ShaderStageFlags::FRAGMENT,
            0,
            max_sampler_count,
        )?;
        
        let light_set_layout = Self::create_descriptor_set_layout(
            &logical_device,
            vk::DescriptorType::STORAGE_BUFFER,
            vk::ShaderStageFlags::FRAGMENT,
            0,
            1,
        )?;

        let skybox_set_layout = Self::create_descriptor_set_layout(
            &logical_device, 
            vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            vk::ShaderStageFlags::FRAGMENT, 
            0, 
            1,
        )?;
        
        let camera_sets = Self::allocate_descriptor_sets(
            &logical_device,
            &swapchain, 
            descriptor_pool,
            camera_set_layout,
        )?;
        
        let texture_sets = Self::allocate_descriptor_sets(
            &logical_device,
            &swapchain, 
            descriptor_pool,
            texture_set_layout,
        )?;
        
        let light_sets = Self::allocate_descriptor_sets(
            &logical_device,
            &swapchain, 
            descriptor_pool,
            light_set_layout,
        )?;

        let skybox_sets = Self::allocate_descriptor_sets(
            &logical_device, 
            &swapchain, 
            descriptor_pool, 
            skybox_set_layout,
        )?;
        
        let set_layouts = vec![
            camera_set_layout, 
            texture_set_layout, 
            light_set_layout, 
            skybox_set_layout
        ];
        
        let push_constants = [
            vk::PushConstantRange::builder()
                .stage_flags(vk::ShaderStageFlags::VERTEX)
                .offset(0)
                .size(128)
                .build()
        ];
                
        let pipelinelayout_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&set_layouts)
            .push_constant_ranges(&push_constants);
            
        let pipeline_layout = logical_device.create_pipeline_layout(&pipelinelayout_info, None)?;
        
        Ok(DescriptorPool {
            descriptor_pool,    
            camera_sets, 
            texture_sets,
            light_sets,
            skybox_sets,
            set_layouts,
            pipeline_layout,
        })
    }
    
    pub unsafe fn bind_buffers(
        &self,
        logical_device: &ash::Device,
        camera_buffer: &Buffer,
        light_buffer: &Buffer,
    ){
        for descset in &self.camera_sets {
            let buffer_infos = [vk::DescriptorBufferInfo {
                buffer: camera_buffer.buffer,
                offset: 0,
                range: 128,
            }];
            let desc_sets_write = [vk::WriteDescriptorSet::builder()
                .dst_set(*descset)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&buffer_infos)
                .build()
            ];
            logical_device.update_descriptor_sets(&desc_sets_write, &[]);
        }
        
        for descset in &self.light_sets {
            let buffer_infos = [vk::DescriptorBufferInfo {
                buffer: light_buffer.buffer,
                offset: 0,
                range: 8,
            }];
            let desc_sets_write = [vk::WriteDescriptorSet::builder()
                .dst_set(*descset)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .buffer_info(&buffer_infos)
                .build()
            ];
            logical_device.update_descriptor_sets(&desc_sets_write, &[]);
        }
    }
    
    pub unsafe fn cleanup(&self, logical_device: &ash::Device){
        logical_device.destroy_descriptor_pool(self.descriptor_pool, None);
        for dsl in &self.set_layouts {
            logical_device.destroy_descriptor_set_layout(*dsl, None);
        }
        logical_device.destroy_pipeline_layout(self.pipeline_layout, None);
    }
    
    unsafe fn create_descriptor_set_layout(
        logical_device: &ash::Device,
        dtype: vk::DescriptorType,
        stage_flags: vk::ShaderStageFlags,
        binding: u32,
        dcount: u32,
    ) -> Result<vk::DescriptorSetLayout, vk::Result> {
        let description = [
            vk::DescriptorSetLayoutBinding::builder()
                .binding(binding)
                .descriptor_type(dtype)
                .descriptor_count(dcount)
                .stage_flags(stage_flags)
                .build()
        ];
        
        let create_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&description);

        logical_device.create_descriptor_set_layout(&create_info, None)
    }
    
    unsafe fn create_descriptor_pool(
        logical_device: &ash::Device,
        swapchain: &Swapchain,
        max_sampler_count: u32,
    ) -> Result<vk::DescriptorPool, vk::Result> {
        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: swapchain.amount_of_images,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: max_sampler_count * swapchain.amount_of_images,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: swapchain.amount_of_images,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: swapchain.amount_of_images,
            },
        ];
        
        let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(4 * swapchain.amount_of_images)
            .pool_sizes(&pool_sizes); 
            
        logical_device.create_descriptor_pool(&descriptor_pool_info, None)
    }
    
    unsafe fn allocate_descriptor_sets(
        logical_device: &ash::Device,
        swapchain: &Swapchain,
        descriptor_pool: vk::DescriptorPool,
        descriptor_set_layout: vk::DescriptorSetLayout,
    ) -> Result<Vec<vk::DescriptorSet>, vk::Result> {
        let desc_layouts = vec![descriptor_set_layout; swapchain.amount_of_images as usize];
        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&desc_layouts);
        logical_device.allocate_descriptor_sets(&descriptor_set_allocate_info)
    }
}
