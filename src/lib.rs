//! ```text
//!                                                                             
//!          8I                                                                 
//!          8I                                                                 
//!          8I                                                                 
//!          8I                                                                 
//!    ,gggg,8I   ,ggg,     ,g,     gg,gggg,     ,ggg,    ,gggggg,    ,ggggg,   
//!   dP"  "Y8I  i8" "8i   ,8'8,    I8P"  "Yb   i8" "8i   dP""""8I   dP"  "Y8ggg
//!  i8'    ,8I  I8, ,8I  ,8'  Yb   I8'    ,8i  I8, ,8I  ,8'    8I  i8'    ,8I  
//! ,d8,   ,d8b, `YbadP' ,8'_   8) ,I8 _  ,d8'  `YbadP' ,dP     Y8,,d8,   ,d8'  
//! P"Y8888P"`Y8888P"Y888P' "YY8P8PPI8 YY88888P888P"Y8888P      `Y8P"Y8888P"    
//!                                 I8                                          
//!                                 I8                                          
//!                                 I8                                          
//!                                 I8                                          
//!                                 I8                                          
//!                                 I8                                          
//! ```
//! Despero (_esp._ **despair**) is rusty data-driven 3D game engine, 
//! which implements paradigm of ECS and provides developers with
//! appropriate toolkit to develop PBR games with advanced technologies
//!
//! # Simple example
//!
//! ```rust
//! use despero::prelude::*;
//! 
//! fn main(){
//! 	let mut despero = Despero::init(WindowBuilder::new().with_title("The Game"));
//! 
//! 	despero
//! 		.add_setup_system(s1)
//! 		.add_system(s2)
//! 		.run()
//! }
//! 
//! fn s1(){
//! 	Debug::info("I run only once!");
//! }
//! 
//! fn s2(){
//! 	Debug::info("I run in loop!");
//! } 
//! ```
//! 

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
//
use std::sync::Arc;
use despero_ecs::*;
use winit::{
	event::*,
	event::Event as WinitEvent,
	platform::run_return::EventLoopExtRunReturn,
	window::WindowBuilder,
};

use ecs::{
	event::*,
};

use crate::render::{
	renderer::Renderer,
	gui::{
		ctx::*,
	},
	pbr::material::*,
	systems::*,
};

/// Module of the main engine error handler [`Desperror`]
pub mod error;
/// Structures implementing mathematics
pub mod math;
/// Submodules and structures to work with graphics
pub mod render;
/// ECS implementations
pub use despero_ecs as ecs;
/// [Rapier3D](https://crates.io/crates/rapier3d) implementations
pub mod physics;
/// [Mlua](https://crates.io/crates/mlua) scripting implementations
pub mod scripting;
/// Bundle of all essential components of the engine
pub mod prelude;

/// Re-import the engine error handling to use as [`despero::Result`]
pub use crate::error::Result;

/// Main engine struct
pub struct Despero {
	world: World,
	systems: ScheduleBuilder,
	setup_systems: ScheduleBuilder,
	event_writer: EventWriter,
	
	renderer: Renderer,
}

impl Despero {	
	/// Initialize Despero application
	pub fn init(window_builder: WindowBuilder) -> Despero {
		let mut renderer = Renderer::init(window_builder).expect("Cannot create renderer");
		renderer.bind_material::<DefaultMat>();
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
		let mut setup_systems = self.setup_systems.build();
		let mut systems = self.systems
			.add_system(update_models_system)
			.add_system(rendering_system)
			.add_system(update_lights)
			.build();
		
		setup_systems.execute((
			&mut self.world,
			&mut self.renderer,
			&mut self.event_writer
		)).expect("Cannot execute setup schedule");
		
		let event_loop = Arc::clone(&self.renderer.window.event_loop);
		(*event_loop.lock().unwrap()).run_return(move |event, _, controlflow| match event {	
			WinitEvent::WindowEvent { event, window_id: _ } => {
				let _response = self.renderer.egui.handle_event(&event);
				
				match event {
                    WindowEvent::CloseRequested => {
                        *controlflow = winit::event_loop::ControlFlow::Exit;
                    }
                    WindowEvent::KeyboardInput {input, ..} => {
						self.event_writer.send(Arc::new(input)).expect("Event send error");
					}
                    _ => (),
                }
			}
					
			WinitEvent::MainEventsCleared => {
				self.renderer.window.request_redraw();
			}
			
			WinitEvent::RedrawRequested(_) => {
				systems.execute((
					&mut self.world,
					&mut self.renderer,
					&mut self.event_writer
				)).expect("Cannot execute loop schedule");
			}
			
			_ => {}
		});
	}
}

impl Drop for Despero {
	fn drop(&mut self) {
		self.renderer.cleanup(&mut self.world);
	}
}
