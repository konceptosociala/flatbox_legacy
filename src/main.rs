pub mod essentials;

use ash::vk;
use winit::event::{Event, WindowEvent};
use essentials::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let eventloop = winit::event_loop::EventLoop::new();
    let window = winit::window::Window::new(&eventloop)?;
    let app = Despero::init(window)?;
    
    eventloop.run(move |event, _, controlflow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *controlflow = winit::event_loop::ControlFlow::Exit;
        }
        Event::MainEventsCleared => {
            // doing the work here (later)
            app.window.request_redraw();
        }
        Event::RedrawRequested(_) => {
            //render here (later)
        }
        _ => {}
    });
    Ok(())
}
