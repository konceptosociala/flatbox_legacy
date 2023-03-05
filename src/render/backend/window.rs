use std::sync::{Arc, Mutex};
use std::mem::ManuallyDrop;
use ash::vk;
#[cfg(feature = "winit")]
use winit::{
    event_loop::EventLoop,
    window::{
        Window as WinitWindow,
        WindowBuilder,
    },
};
use crate::render::{
    backend::{
        instance::Instance,
        surface::Surface,
    },
};

/// Main window structure, containing rendering surface, window instance and event loop
pub struct Window {
    #[cfg(feature = "winit")]
    pub(crate) event_loop: Arc<Mutex<EventLoop<()>>>,
    #[cfg(feature = "winit")]
    pub(crate) window: Arc<WinitWindow>,
    
    pub(crate) surface: ManuallyDrop<Surface>,
}

impl Window {
    pub fn init(
        instance: &Instance, 
        
        #[cfg(feature = "gtk")]
        window_builder: gtk::GLArea,
        
        #[cfg(feature = "winit")]
        window_builder: WindowBuilder,
    ) -> Result<Window, vk::Result> {
        #[cfg(feature = "winit")]
        {
            let event_loop = Arc::new(Mutex::new(EventLoop::new()));
            let window = Arc::new(window_builder.build(&*event_loop.lock().unwrap()).expect("Cannot create window"));
            let surface = ManuallyDrop::new(Surface::init(&window, &instance)?);
            
            return Ok(Window {
                event_loop,
                window,
                surface,
            });
        }
        
        #[cfg(feature = "gtk")]
        todo!();
    }
    
    #[cfg(feature = "winit")]
    pub fn request_redraw(&mut self) {
        self.window.request_redraw();
    }
    
    pub unsafe fn cleanup(&mut self) {
        ManuallyDrop::drop(&mut self.surface);
    }
}
