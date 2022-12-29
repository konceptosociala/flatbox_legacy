//
//   _____                             _____   _                                    _ 
//  / ____|                           |_   _| | |                                  | |
// | (___   ___  _ __  _   _  __ _      | |   | | _____   _____   _   _  ___  _   _| |
//  \___ \ / _ \| '_ \| | | |/ _` |     | |   | |/ _ \ \ / / _ \ | | | |/ _ \| | | | |
//  ____) | (_) | | | | |_| | (_| |_   _| |_  | | (_) \ V /  __/ | |_| | (_) | |_| |_|
// |_____/ \___/|_| |_|\__, |\__,_( ) |_____| |_|\___/ \_/ \___|  \__, |\___/ \__,_(_)
//                      __/ |     |/                               __/ |
//                     |___/                                      |___/	
//
use std::sync::Arc;
use hecs::*;
use hecs_schedule::*;
use winit::{
	event::{
		Event,
		WindowEvent,
		KeyboardInput,
	},
	platform::run_return::EventLoopExtRunReturn,
	window::WindowBuilder,
};

pub mod render;
pub mod ecs;
pub mod physics;
pub mod scripting;
pub mod prelude;

use crate::ecs::{
	systems::*,
	event::*,
};

use crate::render::{
	renderer::Renderer,
	pbr::{
		model::{
			DefaultMat,
			Vertex,
		},
	},
};

pub struct Despero {
	world: World,
	systems: ScheduleBuilder,
	setup_systems: ScheduleBuilder,
	pub event_writer: EventWriter,
	
	renderer: Renderer,
}

impl Despero {	
	/// Initialize Despero application
	pub fn init(window_builder: WindowBuilder) -> Despero {
		let renderer = Renderer::init(window_builder).expect("Cannot create renderer");
		Despero {
			world: World::new(),
			setup_systems: Schedule::builder(),
			systems: Schedule::builder(),
			event_writer: EventWriter::new(),
			renderer,
		}
	}
	
	/// Add cyclical system to schedule
	pub fn add_system<Args, Ret, S>(mut self, system: S) -> Self 
	where
        S: 'static + System<Args, Ret> + Send,
	{
		self.systems.add_system(system);
		self
	}
	
	/// Add setup system to schedule
	pub fn add_setup_system<Args, Ret, S>(mut self, system: S) -> Self 
	where
        S: 'static + System<Args, Ret> + Send,
	{
		self.setup_systems.add_system(system);
		self
	}
	
	pub fn add_event_reader(&mut self) -> EventReader {
		EventReader::new(&mut self.event_writer)
	}
	
	/// Run main event loop
	pub fn run(mut self) {
		// Init setup-systems Schedule
		let mut setup_systems = self.setup_systems
				//
					.build();
		// Init systems Schedule
		let mut systems = self.systems
			.add_system(update_models_system)
			.add_system(rendering_system)
			.add_system(update_lights)
			.build();
		// Execute setup-systems Schedule
		setup_systems
			.execute((&mut self.world, &mut self.renderer, &mut self.event_writer))
			.expect("Cannot execute setup schedule");
		// Extract `EventLoop` from `Renderer`
		let mut eventloop = extract(&mut self.renderer.eventloop);
		// Run EventLoop
		eventloop.run_return(move |event, _, controlflow| match event {	
			Event::WindowEvent {
				event: WindowEvent::CloseRequested,
				..
			} => {
				*controlflow = winit::event_loop::ControlFlow::Exit;
			}
					
			Event::MainEventsCleared => {
				self.renderer.window.request_redraw();
			}
			
			Event::RedrawRequested(_) => {
				// Execute loop schedule	
				systems
					.execute((&mut self.world, &mut self.renderer, &mut self.event_writer))
					.expect("Cannot execute loop schedule");
			}
			
			Event::WindowEvent {
				event: WindowEvent::KeyboardInput {input, ..},
				..
			} => {
				self.event_writer.send::<KeyboardInput>(
					Arc::new(input)
				).expect("Event send error");
			}
			
			_ => {}
		});
	}
}

impl Drop for Despero {
	fn drop(&mut self) {
		unsafe {
			self.renderer.cleanup(&mut self.world);
		};
	}
}

// Extract `Option` variable from struct
pub(crate) fn extract<T>(option: &mut Option<T>) -> T {
	// Create `None` option
	let mut empty: Option<T> = None;
	// Swap variable and `None`
	std::mem::swap(&mut empty, option);
	// Return unwrapped option
	empty.unwrap()
}
