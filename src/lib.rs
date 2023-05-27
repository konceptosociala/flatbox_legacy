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

use winit::{
    event::*,
    event::Event as WinitEvent,
    platform::run_return::EventLoopExtRunReturn,
};

use crate::render::{
    backend::window::WindowBuilder,
    renderer::Renderer,
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
pub mod render;
/// [Rapier3D](https://crates.io/crates/rapier3d) implementations
pub mod physics;
/// [Mlua](https://crates.io/crates/mlua) scripting implementations
pub mod scripting;
/// Bundle of all essential components of the engine
pub mod prelude;

pub use crate::error::Result;

/// Main engine struct
pub struct Despero {
    world: World,
    systems: ScheduleBuilder,
    setup_systems: ScheduleBuilder,
    
    #[cfg(feature = "egui")]
    egui_ctx: EventHandler<GuiContext>,
    app_exit: EventHandler<AppExit>,
    
    physics_handler: PhysicsHandler,
    time_handler: Time,
    
    renderer: Renderer,
    asset_manager: AssetManager,
}

impl Despero {
    /// Initialize Despero application
    pub fn init(window_builder: WindowBuilder) -> Despero {
        init_logger();
        
        let mut renderer = Renderer::init(window_builder).expect("Cannot create renderer");
        renderer.bind_material::<DefaultMat>();
        
        Despero {
            world: World::new(),
            setup_systems: Schedule::builder(),
            systems: Schedule::builder(),
            #[cfg(feature = "egui")]
            egui_ctx: EventHandler::<GuiContext>::new(),
            app_exit: EventHandler::<AppExit>::new(),
            physics_handler: PhysicsHandler::new(),
            time_handler: Time::new(),
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
            .add_system(update_models_system)
            .add_system(rendering_system)
            .add_system(time_system)
            .add_system(update_lights)
            .add_system(update_physics)
            .add_system(generate_textures);
            
        return self;
    }
    
    /// Run main event loop
    pub fn run(mut self) {
        let mut setup_systems = self.setup_systems.build();
        let mut systems = self.systems.build();
        
        setup_systems.execute((
            &mut self.world,
            &mut self.renderer,
            #[cfg(feature = "egui")]
            &mut self.egui_ctx,
            &mut self.app_exit,
            &mut self.time_handler,
            &mut self.physics_handler,
            &mut self.asset_manager,
        )).expect("Cannot execute setup schedule");
    
        let event_loop = (&self.renderer.window.event_loop).clone();
        (*event_loop.lock().unwrap()).run_return(move |event, _, controlflow| match event {    
            WinitEvent::WindowEvent { event, window_id: _ } => {
                #[cfg(feature = "egui")]
                let _response = self.renderer.egui.handle_event(&event);
                
                match event {
                    WindowEvent::CloseRequested => {
                        *controlflow = winit::event_loop::ControlFlow::Exit;
                    }
                    _ => (),
                }
            }
            
            WinitEvent::NewEvents(StartCause::Init) => {
                unsafe { self.renderer.recreate_swapchain().expect("Cannot recreate swapchain"); }
                log::debug!("Recreated swapchain");
            }
                    
            WinitEvent::MainEventsCleared => {
                self.renderer.window.request_redraw();
                if let Some(_) = self.app_exit.read() {
                    *controlflow = winit::event_loop::ControlFlow::Exit;
                }
            }
            
            WinitEvent::RedrawRequested(_) => {
                systems.execute((
                    &mut self.world,
                    &mut self.renderer,
                    #[cfg(feature = "egui")]
                    &mut self.egui_ctx,
                    &mut self.app_exit,
                    &mut self.time_handler,
                    &mut self.physics_handler,
                    &mut self.asset_manager,
                )).expect("Cannot execute loop schedule");
                
                self.world.clear_trackers();
            }
            
            _ => {}
        });
    }
}

impl Drop for Despero {
    fn drop(&mut self) {
        self.asset_manager.cleanup(&mut self.renderer);
        self.renderer.cleanup(&mut self.world);
    }
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