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
use ash::vk;
use gpu_allocator::vulkan::*;
use hecs::*;
use hecs_schedule::*;
use winit::{
	event::{
		Event,
		WindowEvent
	},
	platform::run_return::EventLoopExtRunReturn,
	window::WindowBuilder,
};

pub mod render;
pub mod engine;
pub mod ecs;
pub mod physics;
pub mod scripting;

pub mod prelude;
pub mod prelude_eo;

use crate::ecs::systems::*;
use crate::render::renderer::Renderer;
use crate::engine::{
	model::{
		Model,
		TexturedInstanceData,
		TexturedVertexData,
	},
	camera::Camera,
};

pub struct Despero {
	world: World,
	schedule: ScheduleBuilder,
	renderer: Renderer,
}

impl Despero {	
	pub fn init(window_builder: WindowBuilder) -> Despero {
		let renderer = Renderer::init(window_builder).expect("Cannot create renderer");
		Despero {
			world: World::new(),
			schedule: Schedule::builder(),
			renderer,
		}
	}
	
	pub fn add_system<Args, Ret, S>(mut self, system: S) -> Self 
	where
        S: 'static + System<Args, Ret> + Send,
	{
		self.schedule.add_system(system);
		self
	}
	
	pub fn run(mut self) {
		// Unwrap `EventLoop`
		let mut el = None;
		std::mem::swap(&mut el, &mut self.renderer.eventloop);
		let mut eventloop = el.unwrap();
		// Run EventLoop
		eventloop.run_return(move |event, _, controlflow| match event {			
			Event::MainEventsCleared => {
				self.renderer.window.request_redraw();
			}
			
			Event::RedrawRequested(_) => {
				self.
					schedule
						.add_system(init_models_system)
						.add_system(rendering_system)
						.build()
						.execute((&mut self.world, &mut self.renderer))
						.expect("Cannot execute schedule");
			}
			_ => {}
			
			// Closing
			Event::WindowEvent {
				event: WindowEvent::CloseRequested,
				..
			} => {
				*controlflow = winit::event_loop::ControlFlow::Exit;
			}
			
			Event::WindowEvent {
				event: WindowEvent::KeyboardInput {input, ..},
				..
			} => match input {
				winit::event::KeyboardInput {
					state: winit::event::ElementState::Pressed,
					virtual_keycode: Some(keycode),
					..
				} => match keycode {
					// System
					winit::event::VirtualKeyCode::F5 => {
						let path = "screenshots";
						let name = "name";
						self.renderer.screenshot(format!("{}/{}.jpg", path, name).as_str()).expect("Cannot create screenshot");
					}
					winit::event::VirtualKeyCode::F11 => {
						self.renderer.texture_storage.textures.swap(0, 1);
					}
					winit::event::VirtualKeyCode::Right => {
						for (_, camera) in self.world.query_mut::<&mut Camera>(){
							if camera.is_active {
								camera.turn_right(0.05);
							}
						}
					}
					_ => {}
				},
				_ => {}
			},
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
					let mut alloc: Option<Allocation> = None;
					std::mem::swap(&mut alloc, &mut vb.allocation);
					let alloc = alloc.unwrap();
					self.renderer.allocator.free(alloc).unwrap();
					self.renderer.device.destroy_buffer(vb.buffer, None);
				}
				
				if let Some(xb) = &mut m.indexbuffer {
					// Reassign IndexBuffer allocation to remove
					let mut alloc: Option<Allocation> = None;
					std::mem::swap(&mut alloc, &mut xb.allocation);
					let alloc = alloc.unwrap();
					self.renderer.allocator.free(alloc).unwrap();
					self.renderer.device.destroy_buffer(xb.buffer, None);
				}
				
				if let Some(ib) = &mut m.instancebuffer {
					// Reassign IndexBuffer allocation to remove
					let mut alloc: Option<Allocation> = None;
					std::mem::swap(&mut alloc, &mut ib.allocation);
					let alloc = alloc.unwrap();
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

pub trait Extract {
	fn extract<T>(option: &mut Option<T>) -> T {
		let mut empty: Option<T> = None;
		std::mem::swap(&mut empty, option);
		return empty.unwrap();
	}
}
