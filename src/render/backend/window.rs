use std::sync::{Arc, Mutex};
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
};

/// Main window structure, containing rendering surface, window instance and event loop
pub struct Window {
	pub(crate) event_loop: Arc<Mutex<EventLoop<()>>>,
	pub(crate) window: Arc<WinitWindow>,
	pub(crate) surface: ManuallyDrop<Surface>,
}

impl Window {
	pub fn init(instance: &Instance, window_builder: WindowBuilder) -> Result<Window, vk::Result> {
		let event_loop = Arc::new(Mutex::new(EventLoop::new()));
		let window = Arc::new(window_builder.build(&*event_loop.lock().unwrap()).expect("Cannot create window"));
		let surface = ManuallyDrop::new(Surface::init(&window, &instance)?);
		Ok(Window {
			event_loop,
			window,
			surface,
		})
	}
	
	/*pub fn extract_event_loop(&mut self) -> EventLoop<()> {
		let mut dummy = Arc::new(EventLoop::<()>::new());
		std::mem::swap(&mut self.event_loop, &mut dummy);
		return dummy;
	}*/
	
	pub fn request_redraw(&mut self) {
		self.window.request_redraw();
	}
	
	pub unsafe fn cleanup(&mut self) {
		ManuallyDrop::drop(&mut self.surface);
	}
}
