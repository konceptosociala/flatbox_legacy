#[cfg(feature = "render")]
use winit::{
    event::*,
    event::Event as WinitEvent,
    platform::run_return::EventLoopExtRunReturn,
};

use crate::Sonja;
use super::event::{AppExit, EventHandler};

pub fn empty_runner(_: &mut Sonja){}

#[cfg(feature = "render")]
pub fn default_runner(sonja: &mut Sonja) {
    use crate::render::ui::GuiContext;

    let mut setup_systems = sonja.setup_systems.build();
    let mut systems = sonja.systems.build();
    
    #[cfg(feature = "egui")]
    sonja.events.push_handler(EventHandler::<GuiContext>::new());
    sonja.events.push_handler(EventHandler::<AppExit>::new());
    
    setup_systems.execute((
        &mut sonja.world,
        &mut sonja.renderer,
        &mut sonja.events,
        &mut sonja.time_handler,
        &mut sonja.physics_handler,
        &mut sonja.asset_manager,
    )).expect("Cannot execute setup schedule");

    let event_loop = (&sonja.renderer.window.event_loop).clone();
    (*event_loop.lock().unwrap()).run_return(move |event, _, controlflow| match event {    
        WinitEvent::WindowEvent { event, window_id: _ } => {
            #[cfg(feature = "egui")]
            let _response = sonja.renderer.egui.handle_event(&event);
            
            match event {
                WindowEvent::CloseRequested => {
                    *controlflow = winit::event_loop::ControlFlow::Exit;
                }
                _ => (),
            }
        }
        
        WinitEvent::NewEvents(StartCause::Init) => {
            unsafe { sonja.renderer.recreate_swapchain().expect("Cannot recreate swapchain"); }
            log::debug!("Recreated swapchain");
        }
                
        WinitEvent::MainEventsCleared => {
            sonja.renderer.window.request_redraw();
            if let Some(handler) = sonja.events.get_handler::<AppExit>() {
                if let Some(_) = handler.read() {
                    *controlflow = winit::event_loop::ControlFlow::Exit;
                }
            }
        }
        
        WinitEvent::RedrawRequested(_) => {
            systems.execute((
                &mut sonja.world,
                &mut sonja.renderer,
                &mut sonja.events,
                &mut sonja.time_handler,
                &mut sonja.physics_handler,
                &mut sonja.asset_manager,
            )).expect("Cannot execute loop schedule");
            
            sonja.world.clear_trackers();
        }
        
        _ => {}
    });
}

#[cfg(not(feature = "render"))]
pub fn default_runner(sonja: &mut Sonja) {
    let mut setup_systems = sonja.setup_systems.build();
    let mut systems = sonja.systems.build();

    sonja.events.push_handler(EventHandler::<AppExit>::new());

    setup_systems.execute((
        &mut sonja.world,
        &mut sonja.events,
        &mut sonja.time_handler,
        &mut sonja.physics_handler,
        &mut sonja.asset_manager,
    )).expect("Cannot execute setup schedule");

    loop {
        systems.execute((
            &mut sonja.world,
            &mut sonja.events,
            &mut sonja.time_handler,
            &mut sonja.physics_handler,
            &mut sonja.asset_manager,
        )).expect("Cannot execute loop schedule");

        if let Some(handler) = sonja.events.get_handler::<AppExit>() {
            if let Some(_) = handler.read() {
                return ();
            }
        }
        
        sonja.world.clear_trackers();
    }
}
