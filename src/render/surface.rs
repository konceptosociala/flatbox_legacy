use raw_window_handle::HasRawWindowHandle;
use raw_window_handle::HasRawDisplayHandle;
use ash::vk;

pub struct Surface {
	pub surface: vk::SurfaceKHR,
	pub surface_loader: ash::extensions::khr::Surface,
}

impl Surface {
	pub fn init(
		window: &winit::window::Window,
		entry: &ash::Entry,
		instance: &ash::Instance,
	) -> Result<Surface, vk::Result> {
		// Creating surface from `raw` handles with `ash-window` crate
		let surface = unsafe { ash_window::create_surface(
			&entry, 
			&instance, 
			window.raw_display_handle(), 
			window.raw_window_handle(), 
			None
		)? };
		
		let surface_loader = ash::extensions::khr::Surface::new(&entry, &instance);
		Ok(Surface {
			surface,
			surface_loader,
		})
	}
	
	pub fn get_capabilities(
		&self,
		physical_device: vk::PhysicalDevice,
	) -> Result<vk::SurfaceCapabilitiesKHR, vk::Result> {
		unsafe {
			self.surface_loader.get_physical_device_surface_capabilities(physical_device, self.surface)
		}
	}
	
	pub fn get_present_modes(
		&self,
		physical_device: vk::PhysicalDevice,
	) -> Result<Vec<vk::PresentModeKHR>, vk::Result> {
		unsafe {
			self.surface_loader
				.get_physical_device_surface_present_modes(physical_device, self.surface)
		}
	}
	
	pub fn get_formats(
		&self,
		physical_device: vk::PhysicalDevice,
	) -> Result<Vec<vk::SurfaceFormatKHR>, vk::Result> {
		unsafe {
			self.surface_loader
				.get_physical_device_surface_formats(physical_device, self.surface)
		}
	}
	
	pub fn get_physical_device_surface_support(
		&self,
		physical_device: vk::PhysicalDevice,
		queuefamilyindex: usize,
	) -> Result<bool, vk::Result> {
		unsafe {
			self.surface_loader.get_physical_device_surface_support(
				physical_device,
				queuefamilyindex as u32,
				self.surface,
			)
		}
	}

}

impl Drop for Surface {
	fn drop(&mut self) {
		unsafe {
			self.surface_loader.destroy_surface(self.surface, None);
		}
	}
}
