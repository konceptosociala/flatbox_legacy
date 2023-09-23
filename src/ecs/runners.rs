#[cfg(feature = "render")]
use winit::{
    event::*,
    event::Event as WinitEvent,
    platform::run_return::EventLoopExtRunReturn,
};

use crate::Flatbox;
use super::event::{AppExit, EventHandler};

pub fn empty_runner(_: &mut Flatbox){}

#[cfg(feature = "render")]
pub fn default_runner(flatbox: &mut Flatbox) {
    use crate::render::ui::GuiContext;

    let mut setup_systems = flatbox.schedules.get_mut("update").unwrap().build();
    let mut systems = flatbox.schedules.get_mut("update").unwrap().build();
    
    #[cfg(feature = "egui")]
    flatbox.events.push_handler(EventHandler::<GuiContext>::new());
    flatbox.events.push_handler(EventHandler::<AppExit>::new());
    
    setup_systems.execute((
        &mut flatbox.world,
        &mut flatbox.lua_manager,
        &mut flatbox.renderer,
        &mut flatbox.events,
        &mut flatbox.time_handler,
        &mut flatbox.physics_handler,
        &mut flatbox.asset_manager,
    )).expect("Cannot execute setup schedule");

    let event_loop = (&flatbox.renderer.window.event_loop).clone();
    (*event_loop.lock().unwrap()).run_return(move |event, _, controlflow| match event {    
        WinitEvent::WindowEvent { event, window_id: _ } => {
            #[cfg(feature = "egui")]
            let _response = flatbox.renderer.egui.handle_event(&event);
            
            match event {
                WindowEvent::CloseRequested => {
                    *controlflow = winit::event_loop::ControlFlow::Exit;
                }
                _ => (),
            }
        }
        
        WinitEvent::NewEvents(StartCause::Init) => {
            unsafe { flatbox.renderer.recreate_swapchain().expect("Cannot recreate swapchain"); }
            log::debug!("Recreated swapchain");
        }
                
        WinitEvent::MainEventsCleared => {
            flatbox.renderer.window.request_redraw();
            if let Some(handler) = flatbox.events.get_handler::<AppExit>() {
                if let Some(_) = handler.read() {
                    *controlflow = winit::event_loop::ControlFlow::Exit;
                }
            }
        }
        
        WinitEvent::RedrawRequested(_) => {
            systems.execute((
                &mut flatbox.world,
                &mut flatbox.lua_manager,
                &mut flatbox.renderer,
                &mut flatbox.events,
                &mut flatbox.time_handler,
                &mut flatbox.physics_handler,
                &mut flatbox.asset_manager,
            )).expect("Cannot execute loop schedule");
            
            flatbox.world.clear_trackers(); // TODO: Clear all events (krom GUI)
        }
        
        _ => {}
    });
}

#[cfg(not(feature = "render"))]
pub fn default_runner(flatbox: &mut Flatbox) {
    let mut setup_systems = flatbox.schedules.get_mut("setup").unwrap().build();
    let mut systems = flatbox.schedules.get_mut("update").unwrap().build();

    flatbox.events.push_handler(EventHandler::<AppExit>::new());

    setup_systems.execute((
        &mut flatbox.world,
        &mut flatbox.lua_manager,
        &mut flatbox.events,
        &mut flatbox.time_handler,
        &mut flatbox.physics_handler,
        &mut flatbox.asset_manager,
    )).expect("Cannot execute setup schedule");

    loop {
        systems.execute((
            &mut flatbox.world,
            &mut flatbox.lua_manager,
            &mut flatbox.events,
            &mut flatbox.time_handler,
            &mut flatbox.physics_handler,
            &mut flatbox.asset_manager,
        )).expect("Cannot execute loop schedule");

        if let Some(handler) = flatbox.events.get_handler::<AppExit>() {
            if let Some(_) = handler.read() {
                return ();
            }
        }
        
        flatbox.world.clear_trackers();
    }
}
