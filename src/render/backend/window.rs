use std::sync::{Arc, Mutex};
use std::mem::ManuallyDrop;
use winit::{
    event_loop::EventLoop,
    dpi::LogicalSize,
    window::{
        Fullscreen,   
        Window as WinitWindow,
        WindowBuilder as WinitWindowBuilder,
    },
};
use crate::render::{
    backend::{
        instance::Instance,
        surface::Surface,
    },
};
use crate::WindowBuilder;

use crate::error::*;

/// Main window structure, containing rendering surface, window instance and event loop
pub struct Window {
    pub(crate) event_loop: Arc<Mutex<EventLoop<()>>>,
    pub(crate) window: Arc<Mutex<WinitWindow>>,    
    pub(crate) surface: ManuallyDrop<Surface>,
}

impl Window {
    pub fn init(
        instance: &Instance,         
        window_builder: WinitWindowBuilder,
    ) -> DesperoResult<Window> {
        let event_loop = Arc::new(Mutex::new(EventLoop::new()));
        let window = Arc::new(Mutex::new(window_builder.build(&*event_loop.lock().unwrap()).expect("Cannot create window")));
        let surface = ManuallyDrop::new(Surface::init(&window.lock().unwrap(), &instance)?);
        
        return Ok(Window {
            event_loop,
            window,
            surface,
        });
    }
    
    pub fn request_redraw(&mut self) {
        self.window.lock().unwrap().request_redraw();
    }
    
    pub unsafe fn cleanup(&mut self) {
        ManuallyDrop::drop(&mut self.surface);
    }
}

impl From<WindowBuilder> for WinitWindowBuilder {
    fn from(v: WindowBuilder) -> Self {
        WinitWindowBuilder::new()
            .with_title(v.title.unwrap_or("My Game").to_owned())
            .with_window_icon(v.icon)
            
            .with_inner_size(
                LogicalSize {
                    width: v.width.unwrap_or(800.0),
                    height: v.height.unwrap_or(600.0),
                }
            )
            
            .with_resizable(v.resizable.unwrap_or(true))
            .with_fullscreen(
                match v.fullscreen.unwrap_or(false) {
                    true => Some(Fullscreen::Borderless(None)),
                    false => None,
                }
            )
    }
}
