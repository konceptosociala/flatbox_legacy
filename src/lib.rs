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
use hecs::*;
use hecs_schedule::*;
use winit::{
	event::{
		Event,
		WindowEvent,
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
			Model,
			TexturedInstanceData,
			TexturedVertexData,
		},
	},
};

pub struct Despero {
	world: World,
	systems: ScheduleBuilder,
	setup_systems: ScheduleBuilder,
	pub event_writer: EventWriter<u32>,
	
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
	
	/// Run main event loop
	pub fn run(mut self) {
		// Init setup-systems Schedule
		let mut setup_systems = self.
				setup_systems
					//.add_system(init_models_system)
					.build();
		// Init systems Schedule
		let mut systems = self.systems
			.add_system(init_models_system)
			.add_system(rendering_system)
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
				event: WindowEvent::KeyboardInput {input: _, ..},
				..
			} => {
				self.event_writer.send(15u32).expect("Event send error");
			}
			
			_ => {}
		});
	}
}

impl Drop for Despero {
	fn drop(&mut self) {
		unsafe {
			self.renderer.device.device_wait_idle().expect("Error halting device");	
			// Destroy TextureStorage
			self.renderer.texture_storage.cleanup(&self.renderer.device, &mut self.renderer.allocator);
			// Destroy DescriptorPool
			self.renderer.device.destroy_descriptor_pool(self.renderer.descriptor_pool, None);
			// Destroy UniformBuffer
			self.renderer.device.destroy_buffer(self.renderer.uniformbuffer.buffer, None);
			self.renderer.device.free_memory(self.renderer.uniformbuffer.allocation.as_ref().unwrap().memory(), None);
			// Destroy LightBuffer
			self.renderer.device.destroy_buffer(self.renderer.lightbuffer.buffer, None);
			// Models clean
			for (_, m) in self.world.query_mut::<&mut Model<TexturedVertexData, TexturedInstanceData>>() {
				if let Some(vb) = &mut m.vertexbuffer {
					// Reassign VertexBuffer allocation to remove
					let alloc = extract(&mut vb.allocation);
					self.renderer.allocator.free(alloc).unwrap();
					self.renderer.device.destroy_buffer(vb.buffer, None);
				}
				
				if let Some(xb) = &mut m.indexbuffer {
					// Reassign IndexBuffer allocation to remove
					let alloc = extract(&mut xb.allocation);
					self.renderer.allocator.free(alloc).unwrap();
					self.renderer.device.destroy_buffer(xb.buffer, None);
				}
				
				if let Some(ib) = &mut m.instancebuffer {
					// Reassign IndexBuffer allocation to remove
					let alloc = extract(&mut ib.allocation);
					self.renderer.allocator.free(alloc).unwrap();
					self.renderer.device.destroy_buffer(ib.buffer, None);
				}
			}
			self.renderer.commandbuffer_pools.cleanup(&self.renderer.device);
			self.renderer.pipeline.cleanup(&self.renderer.device);
			self.renderer.device.destroy_render_pass(self.renderer.renderpass, None);
			self.renderer.swapchain.cleanup(&self.renderer.device, &mut self.renderer.allocator);
			self.renderer.device.destroy_device(None);
			std::mem::ManuallyDrop::drop(&mut self.renderer.surfaces);
			std::mem::ManuallyDrop::drop(&mut self.renderer.debug);
			self.renderer.instance.destroy_instance(None);
		};
	}
}

// Extract `Option` variable from struct
pub fn extract<T>(option: &mut Option<T>) -> T {
	// Create `None` option
	let mut empty: Option<T> = None;
	// Swap variable and `None`
	std::mem::swap(&mut empty, option);
	// Return unwrapped option
	empty.unwrap()
}
