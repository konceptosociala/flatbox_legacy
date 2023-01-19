use std::sync::Arc;
use std::mem::ManuallyDrop;
use ash::vk;
use winit::{
	event_loop::EventLoop,
	window::{
		Window as WinitWindow,
		WindowBuilder,
	},
};
use crate::render::{
	backend::{
		instance::Instance,
		surface::Surface,
	},
	renderer::extract_option,
};

/// Main window structure, containing rendering surface, window instance and event loop
pub struct Window {
	event_loop: Option<EventLoop<()>>,
	window: Arc<WinitWindow>,
	pub(crate) surface: ManuallyDrop<Surface>,
}

impl Window {
	pub fn init(instance: &Instance, window_builder: WindowBuilder) -> Result<Window, vk::Result> {
		let event_loop = EventLoop::new();
		let window = window_builder.build(&event_loop).expect("Cannot create window");
		let surface = ManuallyDrop::new(Surface::init(&window, &instance)?);
		Ok(Window {
			event_loop: Some(event_loop),
			window: Arc::new(window),
			surface,
		})
	}
	
	pub fn get_event_loop(&mut self) -> EventLoop<()> {
		extract_option(&mut self.event_loop)
	}
	
	pub fn request_redraw(&mut self) {
		self.window.request_redraw();
	}
	
	pub unsafe fn cleanup(&mut self) {
		ManuallyDrop::drop(&mut self.surface);
	}
}
