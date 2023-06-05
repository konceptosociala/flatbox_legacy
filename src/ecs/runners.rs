#[cfg(feature = "render")]
use winit::{
    event::*,
    event::Event as WinitEvent,
    platform::run_return::EventLoopExtRunReturn,
};

use crate::Despero;
use super::event::{AppExit, EventHandler};

pub fn empty_runner(_: &mut Despero){}

#[cfg(feature = "render")]
pub fn default_runner(despero: &mut Despero) {
    use crate::render::ui::GuiContext;

    let mut setup_systems = despero.setup_systems.build();
    let mut systems = despero.systems.build();
    
    #[cfg(feature = "egui")]
    despero.events.push_handler(EventHandler::<GuiContext>::new());
    despero.events.push_handler(EventHandler::<AppExit>::new());
    
    setup_systems.execute((
        &mut despero.world,
        &mut despero.renderer,
        &mut despero.events,
        &mut despero.time_handler,
        &mut despero.physics_handler,
        &mut despero.asset_manager,
    )).expect("Cannot execute setup schedule");

    let event_loop = (&despero.renderer.window.event_loop).clone();
    (*event_loop.lock().unwrap()).run_return(move |event, _, controlflow| match event {    
        WinitEvent::WindowEvent { event, window_id: _ } => {
            #[cfg(feature = "egui")]
            let _response = despero.renderer.egui.handle_event(&event);
            
            match event {
                WindowEvent::CloseRequested => {
                    *controlflow = winit::event_loop::ControlFlow::Exit;
                }
                _ => (),
            }
        }
        
        WinitEvent::NewEvents(StartCause::Init) => {
            unsafe { despero.renderer.recreate_swapchain().expect("Cannot recreate swapchain"); }
            log::debug!("Recreated swapchain");
        }
                
        WinitEvent::MainEventsCleared => {
            despero.renderer.window.request_redraw();
            if let Some(handler) = despero.events.get_handler::<AppExit>() {
                if let Some(_) = handler.read() {
                    *controlflow = winit::event_loop::ControlFlow::Exit;
                }
            }
        }
        
        WinitEvent::RedrawRequested(_) => {
            systems.execute((
                &mut despero.world,
                &mut despero.renderer,
                &mut despero.events,
                &mut despero.time_handler,
                &mut despero.physics_handler,
                &mut despero.asset_manager,
            )).expect("Cannot execute loop schedule");
            
            despero.world.clear_trackers();
        }
        
        _ => {}
    });
}

#[cfg(not(feature = "render"))]
pub fn default_runner(despero: &mut Despero) {
    let mut setup_systems = despero.setup_systems.build();
    let mut systems = despero.systems.build();

    despero.events.push_handler(EventHandler::<AppExit>::new());

    setup_systems.execute((
        &mut despero.world,
        &mut despero.events,
        &mut despero.time_handler,
        &mut despero.physics_handler,
        &mut despero.asset_manager,
    )).expect("Cannot execute setup schedule");

    loop {
        systems.execute((
            &mut despero.world,
            &mut despero.events,
            &mut despero.time_handler,
            &mut despero.physics_handler,
            &mut despero.asset_manager,
        )).expect("Cannot execute loop schedule");

        if let Some(handler) = despero.events.get_handler::<EventHandler<AppExit>>() {
            if let Some(_) = handler.read() {
                return ();
            }
        }
        
        despero.world.clear_trackers();
    }
}
