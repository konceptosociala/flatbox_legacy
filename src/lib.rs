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
use gpu_allocator::vulkan::*;
use winit::window::WindowBuilder;
use hecs::*;
use hecs_schedule::*;

pub mod render;
pub mod engine;
pub mod ecs;
pub mod physics;
pub mod scripting;
pub mod prelude;

use crate::ecs::systems::*;
use crate::render::renderer::Renderer;
use crate::engine::{
	model::{
		Model,
		TexturedInstanceData,
		TexturedVertexData,
	},
};

pub struct Despero {
	pub world: World,
	pub schedule: ScheduleBuilder,
	pub renderer: Renderer,
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
		self.
			schedule
				.add_system(eventloop_system)
				.add_system(init_models_system)
				.build()
				.execute((&mut self.world, &mut self.renderer))
				.expect("Cannot execute schedule");
	}
	
// TODO: replace textures' vec with hecs ECS
	/*pub fn texture_from_file<P: AsRef<std::path::Path>>(
		&mut self,
		path: P,
		filter: Filter,
	) -> Result<usize, Box<dyn std::error::Error>> {
		self.texture_storage.new_texture_from_file(
			path,
			filter,
			&self.renderer.device,
			&mut self.renderer.allocator,
			&self.renderer.commandbuffer_pools.commandpool_graphics,
			&self.renderer.queues.graphics_queue,
		)
	}*/
}

impl Drop for Despero {
	fn drop(&mut self) {
		unsafe {
			self.renderer.device.device_wait_idle().expect("Error halting device");	
			// Destroy TextureStorage
			self.texture_storage.cleanup(&self.renderer.device, &mut self.renderer.allocator);
			self.renderer.device.destroy_descriptor_pool(self.renderer.descriptor_pool, None);
			// Destroy UniformBuffer
			self.renderer.device.destroy_buffer(self.renderer.uniformbuffer.buffer, None);
			self.renderer.device.free_memory(self.renderer.uniformbuffer.allocation.as_ref().unwrap().memory(), None);
			// Destroy LightBuffer
			self.renderer.device.destroy_buffer(self.renderer.lightbuffer.buffer, None);
			// Models clean
			for (_, m) in self.world.query_mut::<&Model<TexturedVertexData, TexturedInstanceData>>() {
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
