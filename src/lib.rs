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
#[cfg(all(feature = "x11", feature = "windows"))]
compile_error!("features \"x11\" and \"windows\" cannot be enabled at the same time");

use ash::vk;
use gpu_allocator::vulkan::*;
use winit::{
	window::WindowBuilder,
	event::{Event, WindowEvent},
};
use hecs::{
	World,
};

pub mod render;
pub mod engine;
pub mod ecs;
pub mod physics;
pub mod scripting;

use crate::render::{
	renderer::Renderer,
	debug::Debug,
};
use crate::engine::{
	camera::Camera,
	screenshot::Screenshot,
	model::{
		Model,
		TexturedInstanceData,
		TexturedVertexData,
	},
	texture::{
		TextureStorage, 
		Filter
	},
};

// Main struct
pub struct Despero {
	pub world: World,
	pub renderer: Renderer,
	pub models: Vec<Model<TexturedVertexData, TexturedInstanceData>>,
	pub texture_storage: TextureStorage,
	pub camera: Camera,
}

impl Despero {	
	pub fn init(window_builder: WindowBuilder) -> Result<Despero, Box<dyn std::error::Error>> {
		let renderer = Renderer::init(window_builder)?;
		let camera = Camera::builder().build();
		Ok(Despero {
			world: World::new(),
			renderer,
			models: vec![],
			texture_storage: TextureStorage::new(),
			camera,
		})
	}
	// TODO: replace textures' vec with hecs ECS
	pub fn texture_from_file<P: AsRef<std::path::Path>>(
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
	}
	
	pub fn run(mut self) {
		// Update Models' buffers
		for m in &mut self.models {
			m.update_vertexbuffer(
				&self.renderer.device, 
				&mut self.renderer.allocator
			).expect("Cannot update vertexbuffer");
			
			m.update_instancebuffer(
				&self.renderer.device, 
				&mut self.renderer.allocator
			).expect("Cannot update instancebuffer");
			
			m.update_indexbuffer(
				&self.renderer.device,
				&mut self.renderer.allocator
			).expect("Cannot update indexbuffer");
		}
		// Winit EventLoop
		let mut eventloop = None;
		std::mem::swap(&mut self.renderer.eventloop, &mut eventloop);
		let eventloop = eventloop.unwrap();
		eventloop.run(move |event, _, controlflow| match event {
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
						Screenshot::take_jpg(&mut self, name, path).expect("Failed to create a screenshot");
						Debug::info(format!("Screenshot \"{}\" saved in \"{}\"", name, path).as_str());
					}
					winit::event::VirtualKeyCode::F11 => {
						self.texture_storage.textures.swap(0, 1);
					}
					// Rotating
					winit::event::VirtualKeyCode::Right => {
						self.camera.turn_right(0.05);
					}
					winit::event::VirtualKeyCode::Left => {
						self.camera.turn_left(0.05);
					}
					winit::event::VirtualKeyCode::Up => {
						self.camera.turn_up(0.05);
					}
					winit::event::VirtualKeyCode::Down => {
						self.camera.turn_down(0.05);
					}
					// Movement
					winit::event::VirtualKeyCode::W => {
						self.camera.move_forward(0.05);
					}
					winit::event::VirtualKeyCode::S => {
						self.camera.move_backward(0.05);
					}
					winit::event::VirtualKeyCode::A => {
						self.camera.move_left(0.05);
					}
					winit::event::VirtualKeyCode::D => {
						self.camera.move_right(0.05);
					}
					_ => {}
				},
				_ => {}
			},
			
			Event::MainEventsCleared => {
				self.renderer.window.request_redraw();
			}
			
			Event::RedrawRequested(_) => {
				// Get image of swapchain
				let (image_index, _) = unsafe {
					self.renderer
						.swapchain
						.swapchain_loader
						.acquire_next_image(
							self.renderer.swapchain.swapchain,
							std::u64::MAX,
							self.renderer.swapchain.image_available[self.renderer.swapchain.current_image],
							vk::Fence::null(),
						)
						.expect("Error image acquisition")
				};
				// Control fences
				unsafe {
					self.renderer
						.device
						.wait_for_fences(
							&[self.renderer.swapchain.may_begin_drawing[self.renderer.swapchain.current_image]],
							true,
							std::u64::MAX,
						)
						.expect("fence-waiting");
					self.renderer
						.device
						.reset_fences(&[
							self.renderer.swapchain.may_begin_drawing[self.renderer.swapchain.current_image]
						])
						.expect("resetting fences");
				}
				
				self.camera.update_buffer(
					&self.renderer.device, 
					&mut self.renderer.allocator, 
					&mut self.renderer.uniformbuffer
				).expect("Cannot update uniformbuffer");
				
				// Get image descriptor info
				let imageinfos = self.texture_storage.get_descriptor_image_info();
				let descriptorwrite_image = vk::WriteDescriptorSet::builder()
					.dst_set(self.renderer.descriptor_sets_texture[self.renderer.swapchain.current_image])
					.dst_binding(0)
					.dst_array_element(0)
					.descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
					.image_info(&imageinfos)
					.build();

				// Update descriptors
				unsafe {
					self.renderer
						.device
						.update_descriptor_sets(&[descriptorwrite_image], &[]);
				}
				
				self
					.update_commandbuffer(image_index as usize)
					.expect("Cannot update CommandBuffer");
				
				// Submit commandbuffers
				let semaphores_available = [self.renderer.swapchain.image_available[self.renderer.swapchain.current_image]];
				let waiting_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
				let semaphores_finished = [self.renderer.swapchain.rendering_finished[self.renderer.swapchain.current_image]];
				let commandbuffers = [self.renderer.commandbuffers[image_index as usize]];
				let submit_info = [vk::SubmitInfo::builder()
					.wait_semaphores(&semaphores_available)
					.wait_dst_stage_mask(&waiting_stages)
					.command_buffers(&commandbuffers)
					.signal_semaphores(&semaphores_finished)
					.build()];
				unsafe {
					self.renderer
						.device
						.queue_submit(
							self.renderer.queues.graphics_queue,
							&submit_info,
							self.renderer.swapchain.may_begin_drawing[self.renderer.swapchain.current_image],
						)
						.expect("queue submission");
				};
				let swapchains = [self.renderer.swapchain.swapchain];
				let indices = [image_index];
				let present_info = vk::PresentInfoKHR::builder()
					.wait_semaphores(&semaphores_finished)
					.swapchains(&swapchains)
					.image_indices(&indices);
				unsafe {
					if self.renderer
						.swapchain
						.swapchain_loader
						.queue_present(self.renderer.queues.graphics_queue, &present_info)
						.expect("queue presentation")
					{
						self.renderer.recreate_swapchain().expect("swapchain recreation");
						
						self.camera.set_aspect(
							self.renderer.swapchain.extent.width as f32
								/ self.renderer.swapchain.extent.height as f32,
						);
						
						self.camera
							.update_buffer(
								&self.renderer.device, 
								&mut self.renderer.allocator, 
								&mut self.renderer.uniformbuffer
							).expect("camera buffer update");
					}
				};
				// Set swapchain image
				self.renderer.swapchain.current_image =
					(self.renderer.swapchain.current_image + 1) % self.renderer.swapchain.amount_of_images as usize;
			}
			_ => {}
		});
	}
	
	pub fn update_commandbuffer(&mut self, index: usize) -> Result<(), vk::Result> {
		let commandbuffer = self.renderer.commandbuffers[index];
		let commandbuffer_begininfo = vk::CommandBufferBeginInfo::builder();
		
		unsafe {
			self.renderer.device
				.begin_command_buffer(commandbuffer, &commandbuffer_begininfo)?;
		}
		
		let clearvalues = [
			vk::ClearValue {
				color: vk::ClearColorValue {
					float32: [0.0, 0.0, 0.0, 1.0],
				},
			},
			vk::ClearValue {
				depth_stencil: vk::ClearDepthStencilValue {
					depth: 1.0,
					stencil: 0,
				},
			},
		];
		
		let renderpass_begininfo = vk::RenderPassBeginInfo::builder()
			.render_pass(self.renderer.renderpass)
			.framebuffer(self.renderer.swapchain.framebuffers[index])
			.render_area(vk::Rect2D {
				offset: vk::Offset2D { x: 0, y: 0 },
				extent: self.renderer.swapchain.extent,
			})
			.clear_values(&clearvalues);
		unsafe {
			// Bind RenderPass
			self.renderer.device.cmd_begin_render_pass(
				commandbuffer,
				&renderpass_begininfo,
				vk::SubpassContents::INLINE,
			);
			// Bind Pipeline
			self.renderer.device.cmd_bind_pipeline(
				commandbuffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.renderer.pipeline.pipeline,
			);
			// Bind DescriptorSets
			self.renderer.device.cmd_bind_descriptor_sets(
				commandbuffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.renderer.pipeline.layout,
				0,
				&[
					self.renderer.descriptor_sets_camera[index],
					self.renderer.descriptor_sets_texture[index],
					//self.renderer.descriptor_sets_light[index],
				],
				&[],
			);
			for m in &self.models {
				m.draw(
					&self.renderer.device,
					commandbuffer,
				);
			}
			self.renderer.device.cmd_end_render_pass(commandbuffer);
			self.renderer.device.end_command_buffer(commandbuffer)?;
		}
		Ok(())
	}
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
			for m in &mut self.models {
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
