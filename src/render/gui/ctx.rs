use std::sync::Arc;
use egui_winit_ash_integration::*;
use egui::*;
//~ use egui_winit::*;
use ash::vk;

use crate::render::renderer::Renderer;

pub trait EguiExt {
	type InitError;
	
	fn init(renderer: &Renderer) -> Result<Self, Self::InitError>
	where 
		Self: Sized;
}

impl EguiExt for Context {
	type InitError = vk::Result;
	
	fn init(renderer: &Renderer) -> Result<Self, Self::InitError> {
		let integration = Integration::new(
			&*renderer.window.event_loop.lock().unwrap(),
			renderer.swapchain.extent.width,
			renderer.swapchain.extent.height,
			1.0,
			FontDefinitions::default(),
			Style::default(),
			renderer.device.clone(),
			Arc::clone(&renderer.allocator),
			renderer.queue_families.graphics_index.unwrap(),
			renderer.queue_families.graphics_queue,
			renderer.swapchain.swapchain_loader.clone(),
			renderer.swapchain.swapchain,
			*renderer.window.surface.get_formats(*renderer.instance.physical_device)?.first().unwrap(),
		);
				
		Ok(integration.context())
	}
}
