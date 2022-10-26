pub mod essentials;

use ash::vk;
use winit::event::{Event, WindowEvent};
use essentials::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let eventloop = winit::event_loop::EventLoop::new();
	let window = winit::window::Window::new(&eventloop)?;
	let mut despero = Despero::init(window)?;
	
	eventloop.run(move |event, _, controlflow| match event {
		Event::WindowEvent {
			event: WindowEvent::CloseRequested,
			..
		} => {
			*controlflow = winit::event_loop::ControlFlow::Exit;
		}
		Event::MainEventsCleared => {
			despero.window.request_redraw();
		}
		Event::RedrawRequested(_) => {
			// Get image of swapchain
			let (image_index, _) = unsafe {
				despero
					.swapchain
					.swapchain_loader
					.acquire_next_image(
						despero.swapchain.swapchain,
						std::u64::MAX,
						despero.swapchain.image_available[despero.swapchain.current_image],
						vk::Fence::null(),
					)
					.expect("Error image acquisition")
			};
			// Control fences
			unsafe {
				despero
					.device
					.wait_for_fences(
						&[despero.swapchain.may_begin_drawing[despero.swapchain.current_image]],
						true,
						std::u64::MAX,
					)
					.expect("fence-waiting");
				despero
					.device
					.reset_fences(&[
						despero.swapchain.may_begin_drawing[despero.swapchain.current_image]
					])
					.expect("resetting fences");
			}
			// Submit commandbuffers
			let semaphores_available =
				[despero.swapchain.image_available[despero.swapchain.current_image]];
			let waiting_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
			let semaphores_finished =
				[despero.swapchain.rendering_finished[despero.swapchain.current_image]];
			let commandbuffers = [despero.commandbuffers[image_index as usize]];
			let submit_info = [vk::SubmitInfo::builder()
				.wait_semaphores(&semaphores_available)
				.wait_dst_stage_mask(&waiting_stages)
				.command_buffers(&commandbuffers)
				.signal_semaphores(&semaphores_finished)
				.build()];
			unsafe {
				despero
					.device
					.queue_submit(
						despero.queues.graphics_queue,
						&submit_info,
						despero.swapchain.may_begin_drawing[despero.swapchain.current_image],
					)
					.expect("queue submission");
			};
			let swapchains = [despero.swapchain.swapchain];
			let indices = [image_index];
			let present_info = vk::PresentInfoKHR::builder()
				.wait_semaphores(&semaphores_finished)
				.swapchains(&swapchains)
				.image_indices(&indices);
			unsafe {
				despero
					.swapchain
					.swapchain_loader
					.queue_present(despero.queues.graphics_queue, &present_info)
					.expect("queue presentation");
			};
			// Set swapchain image
			despero.swapchain.current_image =
				(despero.swapchain.current_image + 1) % despero.swapchain.amount_of_images as usize;
		}
		_ => {}
	});
}
