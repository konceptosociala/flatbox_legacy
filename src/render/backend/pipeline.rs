use ash::vk;

use crate::render::{
	renderer::*,
	backend::{
		surface::Surface,
		//~ swapchain::Swapchain,
		shader::*,
	},
};

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
		let mut vertex_attributes = vec![
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
				stride: instance_bytes as u32,
				input_rate: vk::VertexInputRate::INSTANCE,
			},
		];
		
		vertex_attributes.extend(instance_attributes);
		
		let vertexshader_module = renderer.device.create_shader_module(&vertex_shader, None)?;
		let fragmentshader_module = renderer.device.create_shader_module(&fragment_shader, None)?;
		
		// Set main function's name to `main`
		let main_function = std::ffi::CString::new("main").unwrap();
		
		let vertex_shader_stage = vk::PipelineShaderStageCreateInfo::builder()
			.stage(vk::ShaderStageFlags::VERTEX)
			.module(vertexshader_module)
			.name(&main_function);
		let fragment_shader_stage = vk::PipelineShaderStageCreateInfo::builder()
			.stage(vk::ShaderStageFlags::FRAGMENT)
			.module(fragmentshader_module)
			.name(&main_function);
			
		let shader_stages = vec![vertex_shader_stage.build(), fragment_shader_stage.build()];
		
		// Bind vertex inputs
		let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
			.vertex_attribute_descriptions(&vertex_attributes)
			.vertex_binding_descriptions(&vertex_bindings);
			
		let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
			.topology(vk::PrimitiveTopology::TRIANGLE_LIST);
		
		// Viewports
		let viewports = [vk::Viewport {
			x: 0.,
			y: 0.,
			width: renderer.swapchain.extent.width as f32,
			height: renderer.swapchain.extent.height as f32,
			min_depth: 0.,
			max_depth: 1.,
		}];
		
		let scissors = [vk::Rect2D {
			offset: vk::Offset2D { x: 0, y: 0 },
			extent: renderer.swapchain.extent,
		}];

		let viewport_info = vk::PipelineViewportStateCreateInfo::builder()
			.viewports(&viewports)
			.scissors(&scissors);
			
		// Rasterizer
		let rasterizer_info = vk::PipelineRasterizationStateCreateInfo::builder()
			.line_width(1.0)
			.front_face(vk::FrontFace::COUNTER_CLOCKWISE)
			.cull_mode(vk::CullModeFlags::BACK)
			.polygon_mode(vk::PolygonMode::FILL);
			
		// Multisampler	
		let multisampler_info = vk::PipelineMultisampleStateCreateInfo::builder()
			.rasterization_samples(vk::SampleCountFlags::TYPE_1);
			
		// Color blend
		let colorblend_attachments = [vk::PipelineColorBlendAttachmentState::builder()
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
			.build()];
		let colourblend_info = vk::PipelineColorBlendStateCreateInfo::builder()
			.attachments(&colorblend_attachments);
		let depth_stencil_info = vk::PipelineDepthStencilStateCreateInfo::builder()
			.depth_test_enable(true)
			.depth_write_enable(true)
			.depth_compare_op(vk::CompareOp::LESS_OR_EQUAL);
		
		let push_constants = [
			vk::PushConstantRange::builder()
				.stage_flags(vk::ShaderStageFlags::VERTEX)
				.offset(0)
				.size(128)
				.build()
		];
				
		let pipelinelayout_info = vk::PipelineLayoutCreateInfo::builder()
			.set_layouts(&renderer.descriptor_pool.descriptor_set_layouts)
			.push_constant_ranges(&push_constants);
			
		let layout = renderer.device.create_pipeline_layout(&pipelinelayout_info, None)?;
		
		let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
			.stages(&shader_stages)
			.vertex_input_state(&vertex_input_info)
			.input_assembly_state(&input_assembly_info)
			.viewport_state(&viewport_info)
			.rasterization_state(&rasterizer_info)
			.multisample_state(&multisampler_info)
			.depth_stencil_state(&depth_stencil_info)
			.color_blend_state(&colourblend_info)
			.layout(layout)
			.render_pass(renderer.renderpass)
			.subpass(0);
			
		let pipeline = renderer.device
			.create_graphics_pipelines(
				vk::PipelineCache::null(),
				&[pipeline_info.build()],
				None,
			).expect("Cannot create pipeline")[0];
		
		renderer.device.destroy_shader_module(fragmentshader_module, None);
		renderer.device.destroy_shader_module(vertexshader_module, None);
		
		Ok(Pipeline {
			pipeline,
			layout,
		})
	}
	
	pub(crate) fn cleanup(&self, logical_device: &ash::Device) {
		unsafe {
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
}
