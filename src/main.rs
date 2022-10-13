pub mod essentials;

use ash::vk;
use winit::event::{Event, WindowEvent};
use essentials::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let eventloop = winit::event_loop::EventLoop::new();
	let window = winit::window::Window::new(&eventloop)?;
	let app = Despero::init(window)?;
    
	dbg!(&app);
    
	Ok(())
}
