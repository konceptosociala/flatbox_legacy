use ash::vk;

use crate::render::{
	renderer::MAX_NUMBER_OF_TEXTURES,
	backend::{
		surface::Surface,
		swapchain::Swapchain,
	},
};

pub type ShaderInputAttribute = vk::VertexInputAttributeDescription;
pub type ShaderInputBinding = vk::VertexInputBindingDescription;
pub type ShaderInputFormat = vk::Format;

pub struct ShaderStages {
	vertex_stage: vk::PipelineShaderStageCreateInfo,
	fragment_stage: vk::PipelineShaderStageCreateInfo,
	vertex_module: vk::ShaderModule,
	fragment_module: vk::ShaderModule,
}

impl ShaderStages {
	pub unsafe fn create(
		vertex_shader: &vk::ShaderModuleCreateInfo,
		fragment_shader: &vk::ShaderModuleCreateInfo,
		logical_device: &ash::Device,
	) -> Result<ShaderStages, vk::Result> {
		let vertex_module = logical_device.create_shader_module(vertex_shader, None)?;
		let fragment_module = logical_device.create_shader_module(fragment_shader, None)?;
		
		let main_function = CString::new("main").unwrap();
		
		let vertex_stage = vk::PipelineShaderStageCreateInfo::builder()
			.stage(vk::ShaderStageFlags::VERTEX)
			.module(vertexshader_module)
			.name(&main_function)
			.build();
			
		let fragment_stage = vk::PipelineShaderStageCreateInfo::builder()
			.stage(vk::ShaderStageFlags::FRAGMENT)
			.module(fragmentshader_module)
			.name(&main_function)
			.build();
			
		Ok(ShaderStages {
			vertex_stage,
			fragment_stage,
			vertex_module,
			fragment_module,
		})
	}
	
	pub fn get_stages(&self) -> Vec<vk::PipelineShaderStageCreateInfo> {
		vec![self.vertex_stage.clone(), self.fragment_stage.clone()]
	}
	
	pub unsafe fn destroy(&self, logical_device: &ash::Device) {
		logical_device.destroy_shader_module(self.fragment_module, None);
		logical_device.destroy_shader_module(self.vertex_module, None);
	}
}

pub struct Pipeline {
	pub pipeline: vk::Pipeline,
	pub layout: vk::PipelineLayout,
}

impl Pipeline {	
	pub unsafe fn init(
		renderer: &Renderer,
		vertex_shader: &vk::ShaderModuleCreateInfo,
		fragment_shader: &vk::ShaderModuleCreateInfo,
		instance_attributes: Vec<ShaderInputAttribute>,
		instance_bytes: usize,
	) -> Result<Pipeline, vk::Result> {
		let vertex_attributes = vec![
			ShaderInputAttribute {
				binding: 0,
				location: 0,
				offset: 0,
				format: ShaderInputFormat::R32G32B32_SFLOAT,
			},
			ShaderInputAttribute {
				binding: 0,
				location: 1,
				offset: 12,
				format: ShaderInputFormat::R32G32B32_SFLOAT,
			},
			ShaderInputAttribute {
				binding: 0,
				location: 2,
				offset: 24,
				format: ShaderInputFormat::R32G32_SFLOAT,
			},
		];
		
		let vertex_bindings = vec![
			ShaderInputBinding {
				binding: 0,
				stride: 32,
				input_rate: vk::VertexInputRate::VERTEX,
			},
			ShaderInputBinding {
				binding: 1,
				stride: instance_bytes,
				input_rate: vk::VertexInputRate::INSTANCE,
			},
		];
		
		vertex_attributes.extend(instance_attributes);
		
		let shader_stages = unsafe { ShaderStages::create(vertex_shader, fragment_shader, &renderer.device)? };
		let vertex_input_info = Self::create_vertex_input_info(&vertex_attributes, &vertex_bindings);
		let input_assembly_info = Self::create_input_assembly();
		let viewport_info = Self::create_viewport_info(&swapchain);
		let rasterizer_info = Self::create_rasterizer();
		let multisampler_info = Self::create_multisampler();		
		let colourblend_info = Self::create_color_blend();
		let depth_stencil_info = Self::create_depth_stencil();
		let push_constants = Self::create_push_constants(vk::ShaderStageFlags::VERTEX, 0, 128);	
		let layout = Self::create_pipeline_layout(&logical_device, &descriptor_set_layouts, &push_constants)?;
		
		let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
			.stages(&shader_stages.get_stages())
			.vertex_input_state(&vertex_input_info)
			.input_assembly_state(&input_assembly_info)
			.viewport_state(&viewport_info)
			.rasterization_state(&rasterizer_info)
			.multisample_state(&multisampler_info)
			.depth_stencil_state(&depth_stencil_info)
			.color_blend_state(&colourblend_info)
			.layout(pipelinelayout)
			.render_pass(*renderpass)
			.subpass(0);
			
		let pipeline = unsafe {
			logical_device
				.create_graphics_pipelines(
					vk::PipelineCache::null(),
					&[pipeline_info.build()],
					None,
				).expect("Cannot create pipeline")				
		}[0];
		
		unsafe { shader_stages.destroy(&logical_device) };
		
		Ok(Pipeline {
			pipeline,
			layout,
		})
	}
	
	pub(crate) fn cleanup(&self, logical_device: &ash::Device) {
		unsafe {
			for dsl in &self.descriptor_set_layouts {
				logical_device.destroy_descriptor_set_layout(*dsl, None);
			}
			logical_device.destroy_pipeline(self.pipeline, None);
			logical_device.destroy_pipeline_layout(self.layout, None);
		}
	}
	
	pub(crate) fn init_renderpass(
		logical_device: &ash::Device,
		physical_device: vk::PhysicalDevice,
		surfaces: &Surface
	) -> Result<vk::RenderPass, vk::Result> {
		let attachments = [
			vk::AttachmentDescription::builder()
				.format(
					surfaces
						.get_formats(physical_device)?
						.first()
						.unwrap()
						.format,
				)
				.load_op(vk::AttachmentLoadOp::CLEAR)
				.store_op(vk::AttachmentStoreOp::STORE)
				.stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
				.stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
				.initial_layout(vk::ImageLayout::UNDEFINED)
				.final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
				.samples(vk::SampleCountFlags::TYPE_1)
				.build(),
			vk::AttachmentDescription::builder()
				.format(vk::Format::D32_SFLOAT)
				.load_op(vk::AttachmentLoadOp::CLEAR)
				.store_op(vk::AttachmentStoreOp::DONT_CARE)
				.stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
				.stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
				.initial_layout(vk::ImageLayout::UNDEFINED)
				.final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
				.samples(vk::SampleCountFlags::TYPE_1)
				.build(),
		];
		
		// Color attachment reference
		let color_attachment_references = [vk::AttachmentReference {
			attachment: 0,
			layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
		}];
		
		// Depth attachment
		let depth_attachment_references = vk::AttachmentReference {
			attachment: 1,
			layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
		};

		let subpasses = [vk::SubpassDescription::builder()
			.color_attachments(&color_attachment_references)
			.depth_stencil_attachment(&depth_attachment_references)
			.pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
			.build()
		];
		
		let subpass_dependencies = [vk::SubpassDependency::builder()
			.src_subpass(vk::SUBPASS_EXTERNAL)
			.src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
			.dst_subpass(0)
			.dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
			.dst_access_mask(
				vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
			)
			.build()
		];
		
		let renderpass_info = vk::RenderPassCreateInfo::builder()
			.attachments(&attachments)
			.subpasses(&subpasses)
			.dependencies(&subpass_dependencies);
		let renderpass = unsafe { logical_device.create_render_pass(&renderpass_info, None)? };
		Ok(renderpass)
	}
	
	unsafe fn create_pipeline_layout(
		logical_device: &ash::Device,
		descriptor_set_layouts: &Vec<vk::DescriptorSetLayout>,
		push_constants: &[vk::PushConstantRange],
	) -> Result<vk::PipelineLayout, vk::Result> {
		let pipelinelayout_info = vk::PipelineLayoutCreateInfo::builder()
			.set_layouts(&descriptor_set_layouts)
			.push_constant_ranges(&push_constants);
			
		logical_device.create_pipeline_layout(&pipelinelayout_info, None)?
	}
	
	fn create_push_constants(
		flags: vk::ShaderStageFlags,
		offset: u32,
		constant_size: u32,
	) -> [vk::PushConstantRange; 1] {
		[
			vk::PushConstantRange::builder()
				.stage_flags(flags)
				.offset(offset)
				.size(constant_size)
				.build()
		]
	}
	
	fn create_input_assembly() -> vk::PipelineInputAssemblyStateCreateInfo {
		vk::PipelineInputAssemblyStateCreateInfo::builder()
			.topology(vk::PrimitiveTopology::TRIANGLE_LIST)
			.build()
	}
	
	fn create_vertex_input_info(
		vertex_attributes: Vec<ShaderInputAttribute>,
		vertex_bindings: Vec<ShaderInputBinding>,
	) -> vk::PipelineVertexInputStateCreateInfo {
		vk::PipelineVertexInputStateCreateInfo::builder()
			.vertex_attribute_descriptions(&vertex_attributes)
			.vertex_binding_descriptions(&vertex_bindings)
			.build()
	}
	
	fn create_multisampler() -> vk::PipelineMultisampleStateCreateInfo {
		vk::PipelineMultisampleStateCreateInfo::builder()
			.rasterization_samples(vk::SampleCountFlags::TYPE_1)
			.build()
	}
	
	fn create_rasterizer() -> vk::PipelineRasterizationStateCreateInfo {
		vk::PipelineRasterizationStateCreateInfo::builder()
			.line_width(1.0)
			.front_face(vk::FrontFace::COUNTER_CLOCKWISE)
			.cull_mode(vk::CullModeFlags::BACK)
			.polygon_mode(vk::PolygonMode::FILL)
			.build()
	}
	
	fn create_depth_stencil() -> vk::PipelineDepthStencilStateCreateInfo {
		vk::PipelineDepthStencilStateCreateInfo::builder()
			.depth_test_enable(true)
			.depth_write_enable(true)
			.depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
			.build()
	}
	
	fn create_viewport_info(swapchain: &Swapchain) -> vk::PipelineViewportState {
		let viewports = Self::create_viewports(&swapchain);
		let scissors = Self::create_scissors(&swapchain);
		
		vk::PipelineViewportStateCreateInfo::builder()
			.viewports(viewports)
			.scissors(scissors)
			.build()
	}
	
	fn create_scissors(swapchain: &Swapchain) -> [vk::Rect2D; 1] {
		[
			vk::Rect2D {
				offset: vk::Offset2D { x: 0, y: 0 },
				extent: swapchain.extent,
			}
		]
	}
	
	fn create_color_blend() -> vk::PipelineColorBlendStateCreateInfo {
		let colorblend_attachments = [
			vk::PipelineColorBlendAttachmentState::builder()
				.blend_enable(true)
				.src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
				.dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
				.color_blend_op(vk::BlendOp::ADD)
				.src_alpha_blend_factor(vk::BlendFactor::SRC_ALPHA)
				.dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
				.alpha_blend_op(vk::BlendOp::ADD)
				.color_write_mask(
					vk::ColorComponentFlags::R
						| vk::ColorComponentFlags::G
						| vk::ColorComponentFlags::B
						| vk::ColorComponentFlags::A,
				)
				.build()
		];
			
		vk::PipelineColorBlendStateCreateInfo::builder()
			.attachments(&colorblend_attachments)
			.build()
	}
	
	fn create_viewports(swapchain: &Swapchain) -> [vk::Viewport; 1] {
		[
			vk::Viewport {
				x: 0.,
				y: 0.,
				width: swapchain.extent.width as f32,
				height: swapchain.extent.height as f32,
				min_depth: 0.,
				max_depth: 1.,
			}
		]
	}
}
