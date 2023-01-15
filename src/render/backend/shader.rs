use ash::vk;

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
