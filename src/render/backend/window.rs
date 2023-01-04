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

pub(crate) struct Window {
	event_loop: Option<EventLoop<()>>,
	window: WinitWindow,
	pub(crate) surface: ManuallyDrop<Surface>,
}

impl Window {
	pub(crate) fn init(instance: &Instance, window_builder: WindowBuilder) -> Result<Window, vk::Result> {
		let event_loop = EventLoop::new();
		let window = window_builder.build(&event_loop).expect("Cannot create window");
		let surface = ManuallyDrop::new(Surface::init(&window, &instance)?);
		Ok(Window {
			event_loop: Some(event_loop),
			window,
			surface,
		})
	}
	
	pub(crate) fn get_event_loop(&mut self) -> EventLoop<()> {
		extract_option(&mut self.event_loop)
	}
	
	pub(crate) fn request_redraw(&mut self) {
		self.window.request_redraw();
	}
	
	pub(crate) unsafe fn cleanup(&mut self) {
		ManuallyDrop::drop(&mut self.surface);
	}
}
