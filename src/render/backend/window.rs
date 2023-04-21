use std::sync::{Arc, Mutex};
use std::mem::ManuallyDrop;
use ash::vk;
#[cfg(feature = "winit")]
use winit::{
    event_loop::EventLoop,
    dpi::LogicalSize,
    window::{
        Icon,
        Fullscreen,   
        Window as WinitWindow,
        WindowBuilder as WinitWindowBuilder,
    },
};
use crate::render::{
    renderer::RenderType,
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
    pub(crate) window: Arc<Mutex<WinitWindow>>,
    #[cfg(feature = "gtk")]
    pub(crate) gl_area: gtk::GLArea,
    
    pub(crate) surface: ManuallyDrop<Surface>,
}

impl Window {
    pub fn init(
        instance: &Instance, 
        
        #[cfg(feature = "gtk")]
        window_builder: gtk::GLArea,
        #[cfg(feature = "winit")]
        window_builder: WinitWindowBuilder,
    ) -> Result<Window, vk::Result> {
        #[cfg(feature = "winit")]
        {
            let event_loop = Arc::new(Mutex::new(EventLoop::new()));
            let window = Arc::new(Mutex::new(window_builder.build(&*event_loop.lock().unwrap()).expect("Cannot create window")));
            let surface = ManuallyDrop::new(Surface::init(&window.lock().unwrap(), &instance)?);
            
            return Ok(Window {
                event_loop,
                window,
                surface,
            });
        }
        
        #[cfg(feature = "gtk")]
        {
            let surface = ManuallyDrop::new(Surface::init(&window_builder, &instance)?);
            
            return Ok(Window {
                gl_area: window_builder.clone(),
                surface,
            });
        }
    }
    
    #[cfg(feature = "winit")]
    pub fn request_redraw(&mut self) {
        self.window.lock().unwrap().request_redraw();
    }
    
    pub unsafe fn cleanup(&mut self) {
        ManuallyDrop::drop(&mut self.surface);
    }
}

#[cfg(feature = "winit")]
#[derive(Default, Debug, Clone)]
pub struct WindowBuilder {
    pub title: Option<&'static str>,
    pub icon: Option<Icon>,
    
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub fullscreen: Option<bool>,
    pub resizable: Option<bool>,
    
    pub renderer: Option<RenderType>,
}

#[cfg(feature = "winit")]
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
