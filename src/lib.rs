// 
// .d88888b                    oo                           oo                                                      oo          dP 
// 88.    "'                                                                                                                    88 
// `Y88888b. .d8888b. 88d888b. dP .d8888b.       88d8b.d8b. dP    .d8888b. 88d8b.d8b. .d8888b. .d8888b.    dP   .dP dP 88d888b. 88 
//       `8b 88'  `88 88'  `88 88 88'  `88       88'`88'`88 88    88'  `88 88'`88'`88 88'  `88 Y8ooooo.    88   d8' 88 88'  `88 dP 
// d8'   .8P 88.  .88 88    88 88 88.  .88 dP    88  88  88 88    88.  .88 88  88  88 88.  .88       88    88 .88'  88 88    88 
//  Y88888P  `88888P' dP    dP 88 `88888P8 88    dP  dP  dP dP    `88888P8 dP  dP  dP `88888P8 `88888P'    8888P'   dP dP    dP oo
//                             88          .P                                                                                   
//                           d8dP                                                                                               
//

//! Despero (_esp._ **despair**) is rusty data-driven 3D game engine, 
//! which implements paradigm of ECS and provides developers with
//! appropriate toolkit to develop PBR games with advanced technologies
//!
//! # Simple example
//!
//! ```rust
//! use despero::prelude::*;
//! 
//! fn main(){
//!     Despero::init(WindowBuilder {
//!         title: Some("My Game"),
//!         ..Default::default()
//!     })
//!        
//!         .default_systems()
//!
//!         .add_setup_system(create_model)
//!         .add_setup_system(create_camera)
//!         .add_system(rotate_model)
//!         .run();
//! }
//! 
//! fn create_model(
//!     mut cmd: Write<CommandBuffer>,
//!     mut renderer: Write<Renderer>,
//! ){
//!     let texture = renderer.create_texture("assets/texture.jpg", Filter::LINEAR) as u32;
//!        
//!     cmd.spawn(ModelBundle {
//!         mesh: Mesh::load_obj("assets/model.obj").swap_remove(0),
//!         material: renderer.create_material(
//!             DefaultMat::builder()
//!                 .texture_id(texture)
//!                 .metallic(0.0)
//!                 .roughness(1.0)
//!                 .build(),
//!         ),
//!         transform: Transform::from_translation(Vector3::new(0.0, 0.0, 0.0)),
//!     });
//!
//!     info!("I run only once!");
//! }
//! 
//! fn rotate_model(
//!     world: SubWorld<&mut Transform>,
//! ){
//!     for (_, mut t) in &mut world.query::<&mut Transform>() {
//!         t.rotation *= UnitQuaternion::from_axis_angle(&Unit::new_normalize(Vector3::new(0.0, 1.0, 0.0)), 0.05);
//!     }
//!
//!     info!("I run in loop!");
//! }
//!  
//! fn create_camera(
//!     mut cmd: Write<CommandBuffer>,
//! ){
//!     cmd.spawn(CameraBundle{
//!         camera: 
//!             Camera::builder()
//!                 .is_active(true)
//!                 .build(),
//!         transform: Transform::default(),
//!     });
//! }
//! ```
//! 

#[cfg(feature = "render")]
use winit::{
    event::*,
    event::Event as WinitEvent,
    platform::run_return::EventLoopExtRunReturn,
    window::Icon,
};

#[cfg(feature = "render")]
use crate::render::{
    renderer::{Renderer, RenderType},
    pbr::material::*,    
};

#[cfg(feature = "egui")]
use crate::render::ui::GuiContext;

use crate::assets::*;
use crate::ecs::*;
use crate::physics::*;
use crate::time::*;

/// Module of the main engine's error handler [`Result`]
pub mod error;
/// Assets and scenes handling
pub mod assets;
/// Structures implementing mathematics
pub mod math;
/// ECS components and re-exports
pub mod ecs;
/// Component connected with time
pub mod time;
/// Submodules and structures to work with graphics
#[cfg(feature = "render")]
pub mod render;
/// [Rapier3D](https://crates.io/crates/rapier3d) implementations
pub mod physics;
/// [Mlua](https://crates.io/crates/mlua) scripting implementations
pub mod scripting;
/// Bundle of all essential components of the engine
pub mod prelude;

pub use crate::error::Result;

#[cfg(all(feature = "egui", not(feature = "render")))]
compile_error!("Feature \"render\" must be enabled in order to use \"egui\"!");

/// Main engine struct
pub struct Despero {
    pub world: World,
    pub systems: ScheduleBuilder,
    pub setup_systems: ScheduleBuilder,
    pub runner: Box<dyn Fn(Despero)>,
    pub window_builder: WindowBuilder,
    
    pub events: Events,
    pub physics_handler: PhysicsHandler,
    pub time_handler: Time,
    
    #[cfg(feature = "render")]
    pub renderer: Renderer,
    pub asset_manager: AssetManager,
}

impl Despero {
    /// Initialize Despero application
    pub fn init(window_builder: WindowBuilder) -> Despero {
        init_logger();
        
        #[cfg(feature = "render")]
        let mut renderer = Renderer::init(window_builder.clone()).expect("Cannot create renderer");
        #[cfg(feature = "render")]
        renderer.bind_material::<DefaultMat>();
        
        Despero {
            world: World::new(),
            setup_systems: Schedule::builder(),
            systems: Schedule::builder(),
            runner: Box::new(default_runner),
            window_builder,
            events: Events::new(),
            physics_handler: PhysicsHandler::new(),
            time_handler: Time::new(),
            #[cfg(feature = "render")]
            renderer,
            asset_manager: AssetManager::new(),
        }
    }
    
    /// Add cyclical system to schedule
    pub fn add_system<Args, Ret, S>(mut self, system: S) -> Self 
    where
        S: 'static + System<Args, Ret> + Send,
    {
        self.systems.add_system(system);
        self
    }
    
    /// Add setup system to schedule
    pub fn add_setup_system<Args, Ret, S>(mut self, system: S) -> Self 
    where
        S: 'static + System<Args, Ret> + Send,
    {
        self.setup_systems.add_system(system);
        self
    }
    
    pub fn default_systems(mut self) -> Self {
        self.setup_systems
            .add_system(main_setup);
        
        self.systems
            .add_system(time_system)
            .add_system(update_physics);
            
        #[cfg(feature = "render")]
        self.systems
            .add_system(update_models_system)
            .add_system(rendering_system)
            .add_system(update_lights)
            .add_system(generate_textures);
            
        self
    }
    
    /// Run main event loop
    pub fn run(mut self) {
        let runner = std::mem::replace(&mut self.runner, Box::new(empty_runner));
        runner(self);
    }
}

impl Drop for Despero {
    fn drop(&mut self) {
        #[cfg(feature = "render")]
        self.asset_manager.cleanup(&mut self.renderer);
        #[cfg(feature = "render")]
        self.renderer.cleanup(&mut self.world);
    }
}

fn empty_runner(_: Despero){}

#[cfg(not(feature = "render"))]
fn default_runner(mut despero: Despero) {
    let mut setup_systems = despero.setup_systems.build();
    let mut systems = despero.systems.build();

    setup_systems.execute((
        &mut despero.world,
        &mut despero.app_exit,
        &mut despero.time_handler,
        &mut despero.physics_handler,
    )).expect("Cannot execute setup schedule");

    loop {
        systems.execute((
            &mut despero.world,
            &mut despero.app_exit,
            &mut despero.time_handler,
            &mut despero.physics_handler,
        )).expect("Cannot execute loop schedule");

        if let Some(_) = despero.app_exit.read() {
            return ();
        }
        
        despero.world.clear_trackers();
    }
}

#[cfg(feature = "render")]
fn default_runner(mut despero: Despero) {
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
            if let Some(handler) = despero.events.get_handler::<EventHandler<AppExit>>() {
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

fn init_logger() {
    #[cfg(debug_assertions)]
    pretty_env_logger::formatted_builder()
        .filter_level(
            env_logger::filter::Builder::new()
                .parse(&std::env::var("DESPERO_LOG").unwrap_or(String::from("DESPERO_LOG=debug")))
                .build()
                .filter()
        )
        .init();
        
    #[cfg(not(debug_assertions))]
    pretty_env_logger::formatted_builder()
        .filter_level(
            env_logger::filter::Builder::new()
                .parse(&std::env::var("DESPERO_LOG").unwrap_or(String::from("DESPERO_LOG=info")))
                .build()
                .filter()
        )
        .init();
}

#[derive(Default, Debug, Clone)]
pub struct WindowBuilder {
    pub title: Option<&'static str>,
    #[cfg(feature = "render")]
    pub icon: Option<Icon>,
    
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub fullscreen: Option<bool>,
    pub resizable: Option<bool>,
    
    #[cfg(feature = "render")]
    pub renderer: Option<RenderType>,
}