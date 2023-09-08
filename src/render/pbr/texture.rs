use std::path::{Path, PathBuf};
use std::fmt;
use image::RgbaImage;
use serde::{
    Serialize, 
    Deserialize,
    Serializer, 
    Deserializer, 
    de::*,
    de::Error as DeError,
    ser::SerializeStruct,
};
use gpu_allocator::vulkan::*;
use gpu_allocator::MemoryLocation;
use ash::vk;

use crate::render::{
    backend::buffer::Buffer,
    renderer::Renderer,
    pbr::color::Color,
};

use crate::error::SonjaResult;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "RawImage")]
pub struct SerializeRawImage {
    pub width: u32,
    pub height: u32,
    pub buffer: Vec<u8>,
}

impl From<SerializeRawImage> for RgbaImage {
    fn from(image: SerializeRawImage) -> Self {
        RgbaImage::from_raw(image.width, image.height, image.buffer.into()).unwrap()
    }
}

impl From<RgbaImage> for SerializeRawImage {
    fn from(image: RgbaImage) -> Self {
        SerializeRawImage {
            width: image.width(),
            height: image.height(),
            buffer: image.into_raw(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TextureLoadType {
    Color(Color<u8>, u32, u32),
    Loaded(PathBuf),
    Generic,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TextureType {
    Plain,
    Cubemap
}

#[readonly::make]
pub struct Texture {
    /// Texture load type. It can be selected manually and is
    /// readonly during future use
    #[readonly]
    pub texture_load_type: TextureLoadType,
    /// Texture type. It can be selected manually and is
    /// readonly during future use
    #[readonly]
    pub texture_type: TextureType,
    /// Image processing filter. In most cases you need `Linear` 
    /// for smooth textures and `Nearest` for pixelized
    pub filter: Filter,
    /// Raw image data
    pub(crate) image: Option<RgbaImage>,
    /// Vulkan image data
    pub(crate) vk_image: Option<vk::Image>,
    /// Image allocation
    pub(crate) image_allocation: Option<Allocation>,
    /// Vulkan image view
    pub(crate) imageview: Option<vk::ImageView>,    
    /// Vulkan sampler
    pub(crate) sampler: Option<vk::Sampler>,
}

impl Clone for Texture {
    fn clone(&self) -> Self {
        Texture { 
            texture_load_type: self.texture_load_type.clone(), 
            texture_type: self.texture_type.clone(), 
            filter: self.filter.clone(), 
            image: self.image.clone(), 
            vk_image: None, 
            image_allocation: None, 
            imageview: None, 
            sampler: None,
        }
    }
}

impl Default for Texture {
    fn default() -> Self {
        Texture::new_solid(Color::grayscale(255), TextureType::Plain, 512, 512)
    }
}

impl std::fmt::Debug for Texture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Texture")
            .field("texture_load_type", &self.texture_load_type)
            .field("texture_type", &self.texture_type)
            .field("filter", &self.filter)
            .finish()
    }
}

impl Serialize for Texture {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer,
    {
        let mut texture = serializer.serialize_struct("Texture", 4)?;
        texture.serialize_field("texture_load_type", &self.texture_load_type)?;
        texture.serialize_field("texture_type", &self.texture_type)?;
        texture.serialize_field("filter", &self.filter)?;

        match self.texture_load_type {
            TextureLoadType::Generic => {
                texture.serialize_field("raw_image", &Some(SerializeRawImage::from(self.image.clone().unwrap())))?;
            },
            _ => {
                texture.serialize_field("raw_image", &Option::<SerializeRawImage>::None)?;
            }
        }

        texture.end()
    }
}

impl<'de> Deserialize<'de> for Texture {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum TextureField { 
            TextureLoadType,
            TextureType,
            Filter,
            RawImage,
        }

        struct TextureVisitor;

        impl<'de> Visitor<'de> for TextureVisitor {
            type Value = Texture;
            
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Texture")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Texture, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let texture_load_type: TextureLoadType = seq.next_element()?.ok_or_else(|| DeError::invalid_length(0, &self))?;
                let texture_type: TextureType = seq.next_element()?.ok_or_else(|| DeError::invalid_length(1, &self))?;
                let filter: Filter = seq.next_element()?.ok_or_else(|| DeError::invalid_length(2, &self))?;

                let raw_image = match texture_load_type {
                    TextureLoadType::Loaded(ref path) => {
                        Texture::create_from_path(path)
                    },
                    TextureLoadType::Color(color, width, height) => {
                        Texture::create_from_color(width, height, color)
                    },
                    TextureLoadType::Generic => {
                        let raw_image: Option<SerializeRawImage> = seq.next_element()?.ok_or_else(|| DeError::invalid_length(3, &self))?;
                        if let Some(image) = raw_image {
                            RgbaImage::from(image)
                        } else {
                            log::error!("Error loading texture: generic texture is empty");
                            Texture::no_image_internal()
                        }
                    },
                };

                Ok(Texture {
                    texture_load_type,
                    texture_type,
                    filter,
                    image: Some(raw_image),
                    vk_image: None,
                    image_allocation: None,
                    imageview: None,
                    sampler: None,
                })
            }

            fn visit_map<V>(self, mut map: V) -> Result<Texture, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut texture_load_type: Option<TextureLoadType> = None;
                let mut texture_type: Option<TextureType> = None;
                let mut filter: Option<Filter> = None;
                let mut raw_image: Option<Option<SerializeRawImage>> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        TextureField::TextureLoadType => {
                            if texture_load_type.is_some() {
                                return Err(DeError::duplicate_field("texture_load_type"));
                            }
                            texture_load_type = Some(map.next_value()?);
                        },
                        TextureField::TextureType => {
                            if texture_type.is_some() {
                                return Err(DeError::duplicate_field("texture_type"));
                            }
                            texture_type = Some(map.next_value()?);
                        },
                        TextureField::Filter => {
                            if filter.is_some() {
                                return Err(DeError::duplicate_field("filter"));
                            }
                            filter = Some(map.next_value()?);
                        },
                        TextureField::RawImage => {
                            if raw_image.is_some() {
                                return Err(DeError::duplicate_field("raw_image"));
                            }
                            raw_image = Some(map.next_value()?);
                        },
                    }
                }

                let texture_load_type = texture_load_type.ok_or_else(|| DeError::missing_field("texture_load_type"))?;
                let texture_type = texture_type.ok_or_else(|| DeError::missing_field("texture_type"))?;
                let filter = filter.ok_or_else(|| DeError::missing_field("filter"))?;

                let raw_image = match texture_load_type {
                    TextureLoadType::Loaded(ref path) => {
                        Texture::create_from_path(path)
                    },
                    TextureLoadType::Color(color, width, height) => {
                        Texture::create_from_color(width, height, color)
                    },
                    TextureLoadType::Generic => {
                        let raw_image: Option<SerializeRawImage> = raw_image.ok_or_else(|| DeError::missing_field("raw_image"))?;
                        if let Some(image) = raw_image {
                            RgbaImage::from(image)
                        } else {
                            log::error!("Error loading texture: generic texture is empty");
                            Texture::no_image_internal()
                        }
                    },
                };

                Ok(Texture {
                    texture_load_type,
                    texture_type,
                    filter,
                    image: Some(raw_image),
                    vk_image: None,
                    image_allocation: None,
                    imageview: None,
                    sampler: None,
                })
            }
        }

        const FIELDS: &'static [&'static str] = &[
            "texture_load_type",
            "texture_type",
            "filter",
            "raw_image"
        ];
        deserializer.deserialize_struct("Texture", FIELDS, TextureVisitor)
    }
}

impl Texture {
    /// Create empty texture with given `filter` and `path`. Requires [`generate`](#method.generate) method applied to it to load the texture to memory
    pub fn new_from_path(
        path: &str, 
        filter: Filter,
        texture_type: TextureType,
    ) -> Self {
        Texture {
            texture_load_type: TextureLoadType::Loaded(path.into()),
            texture_type,
            filter,
            image: None,
            vk_image: None,
            image_allocation: None,
            imageview: None,
            sampler: None,
        }
    }

    pub fn new_from_raw(
        raw_data: &[u8],
        filter: Filter,
        texture_type: TextureType,
        width: u32,
        height: u32,
    ) -> Self {
        Texture { 
            texture_load_type: TextureLoadType::Generic,
            texture_type,
            filter, 
            image: Some(RgbaImage::from_raw(width, height, raw_data.into())
                .unwrap_or(Texture::no_image_internal())), 
            vk_image: None, 
            image_allocation: None, 
            imageview: None, 
            sampler: None
        }
    }

    pub fn new_solid(
        color: impl Into<Color<u8>>,
        texture_type: TextureType,
        width: u32,
        height: u32,
    ) -> Self {
        let color: Color<u8> = color.into();
        let image = Some(Texture::create_from_color(width, height, color));

        Texture { 
            texture_load_type: TextureLoadType::Color(color, width, height),
            texture_type,
            filter: Filter::Nearest, 
            image, 
            vk_image: None, 
            image_allocation: None, 
            imageview: None, 
            sampler: None
        }
    }
    
    /// Create texture from file and load it to memory
    pub fn new_generated(
        path: &'static str, 
        filter: Filter,
        texture_type: TextureType,
        renderer: &mut Renderer
    ) -> SonjaResult<Self> {
        let mut texture = Texture::new_from_path(path, filter, texture_type);
        texture.generate(renderer)?;
        
        Ok(texture)
    }

    pub fn default_skybox() -> Self {
        todo!("Texture::default_skybox()");
    }

    pub fn no_image() -> Self {
        Texture {
            texture_load_type: TextureLoadType::Generic,
            texture_type: TextureType::Plain,
            filter: Filter::Nearest,
            image: Some(Texture::no_image_internal()),
            vk_image: None,
            image_allocation: None,
            imageview: None,
            sampler: None,
        }
    }

    pub fn no_image_blurry() -> Self {
        Texture {
            texture_load_type: TextureLoadType::Generic,
            texture_type: TextureType::Plain,
            filter: Filter::Linear,
            image: Some(Texture::no_image_internal()),
            vk_image: None,
            image_allocation: None,
            imageview: None,
            sampler: None,
        }
    }

    /// Generate texture rendering data for blank texture
    pub fn generate(&mut self, renderer: &mut Renderer) -> SonjaResult<()> {
        let image = {
            if let Some(ref image) = self.image {
                image.clone()
            } else {
                match self.texture_load_type {
                    TextureLoadType::Color(color, width, height) => {
                        Texture::create_from_color(width, height, color)
                    },
                    TextureLoadType::Loaded(ref path) => {
                        Texture::create_from_path(path)
                    },
                    TextureLoadType::Generic => {
                        log::error!("Error loading texture: generic texture is empty");
                        Texture::no_image_internal()
                    },
                }
            }
        };
        
        self.generate_from(renderer, image)?;
        
        Ok(())
    }

    pub fn is_generated(&self) -> bool {
        self.image.is_some() &&
        self.vk_image.is_some() &&
        self.image_allocation.is_some() &&
        self.imageview.is_some() &&
        self.sampler.is_some()
    }

    pub(crate) fn generate_from(
        &mut self, 
        renderer: &mut Renderer,
        image: RgbaImage,
    ) -> SonjaResult<()>{
        let is_cubemap = self.texture_type == TextureType::Cubemap;
        let raw_filter: vk::Filter = self.filter.clone().into();
            
        let (width, height) = if !is_cubemap {
            image.dimensions()
        } else {
            (image.width(), image.width())
        };

        unsafe { renderer.device.device_wait_idle()?; }
        
        let mut img_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .mip_levels(1)
            .format(vk::Format::R8G8B8A8_SRGB)
            .samples(vk::SampleCountFlags::TYPE_1)
            .array_layers(1)
            .usage(
                vk::ImageUsageFlags::TRANSFER_DST |
                vk::ImageUsageFlags::SAMPLED
            );

        if is_cubemap {
            img_create_info.array_layers = 6;
            img_create_info.flags = vk::ImageCreateFlags::CUBE_COMPATIBLE;
        }
        
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
        let mut view_create_info = vk::ImageViewCreateInfo::builder()
            .image(vk_image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vk::Format::R8G8B8A8_SRGB)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                level_count: 1,
                layer_count: 1,
                ..Default::default()
            });

        if is_cubemap {
            view_create_info.view_type = vk::ImageViewType::CUBE;
            view_create_info.subresource_range.layer_count = 6;
        }

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
        let mut barrier = vk::ImageMemoryBarrier::builder()
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
        
        if is_cubemap {
            barrier.subresource_range.layer_count = 6;
        }
            
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

        let mut regions = vec![];
        let offset: u64 = (width as u64) * (width as u64) * 4;
        let faces: u32 = match is_cubemap {
            true => 6,
            false => 1,
        };

        for face in 0..faces {
            let offset = offset * face as u64;

            let image_subresource = vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: face,
                layer_count: 1,
            };

            let region = vk::BufferImageCopy {
                buffer_offset: offset,
                buffer_row_length: 0,
                buffer_image_height: 0,
                image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
                image_extent: vk::Extent3D {
                    width,
                    height,
                    depth: 1,
                },
                image_subresource,
            };

            regions.push(region);
        }
        
        unsafe {
            renderer.device.cmd_copy_buffer_to_image(
                copycmdbuffer,
                buffer.buffer,
                vk_image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &regions,
            );
        }
        
        // Change image layout for fragment shader

        let mut subresource_range = vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        };

        if is_cubemap {
            subresource_range.layer_count = 6;
        }
        
        let barrier = vk::ImageMemoryBarrier::builder()
            .image(vk_image)
            .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ)
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .subresource_range(subresource_range)
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

    fn create_from_color(
        width: u32, 
        height: u32,
        color: Color<u8>
    ) -> RgbaImage {
        RgbaImage::from_pixel(width, height, image::Rgba(color.into()))
    }

    fn create_from_path(
        path: impl AsRef<Path>,
    ) -> RgbaImage {
        image::open(path.as_ref().clone())
            .map(|img| img.to_rgba8())
            .unwrap_or_else(|_|{
                log::error!("Error loading texture: path '{}' not found", path.as_ref().display());
                Texture::no_image_internal()
            })
    }

    fn no_image_internal() -> RgbaImage {
        RgbaImage::from_raw(2, 2, vec![
            0, 0, 0, 255,
            255, 0, 255, 255,
            255, 0, 255, 255,
            0, 0, 0, 255,
        ]).unwrap()
    }
    
    pub fn cleanup(&mut self, renderer: &mut Renderer) {
        if let Some(_) = self.image_allocation {
            let new_alloc = std::mem::take(&mut self.image_allocation).unwrap();
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
