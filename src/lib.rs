// 
// .d88888b                    oo                           oo     
// 88.    "'                                                      
// `Y88888b. .d8888b. 88d888b. dP .d8888b.       88d8b.d8b. dP    
//       `8b 88'  `88 88'  `88 88 88'  `88       88'`88'`88 88    
// d8'   .8P 88.  .88 88    88 88 88.  .88 dP    88  88  88 88    
//  Y88888P  `88888P' dP    dP 88 `88888P8 88    dP  dP  dP dP
//                             88          .P                                                                                   
//                           d8dP                                                                                               
//                                                   oo          dP
//                                                               88
// .d8888b. 88d8b.d8b. .d8888b. .d8888b.    dP   .dP dP 88d888b. 88  
// 88'  `88 88'`88'`88 88'  `88 Y8ooooo.    88   d8' 88 88'  `88 dP 
// 88.  .88 88  88  88 88.  .88       88    88 .88'  88 88    88 
// `88888P8 dP  dP  dP `88888P8 `88888P'    8888P'   dP dP    dP oo
//

/*!

**Sonja** is rusty data-driven 3D game engine, 
which implements paradigm of ECS and provides developers with
appropriate toolkit to develop PBR games with advanced technologies

# Simple example

```rust
use sonja::prelude::*;

fn main(){
    Sonja::init(WindowBuilder {
        title: Some("My Game"),
        ..Default::default()
    })
       
        .default_systems()

        .add_setup_system(create_model)
        .add_setup_system(create_camera)
        .add_system(rotate_model)
        .run();
}

fn create_model(
    mut cmd: Write<CommandBuffer>,
    mut asset_manager: Write<AssetManager>,
) -> SonjaResult<()> {
    let texture = asset_manager.create_texture("assets/texture.jpg", Filter::Linear);
       
    cmd.spawn(ModelBundle {
        model: Model::new("assets/model.obj")?,
        material: asset_manager.create_material(
            DefaultMat::builder()
                .color(Vector3::new(0.5, 0.5, 0.7))
                .albedo(texture)
                .metallic(0.0)
                .roughness(1.0)
                .build(),
        ),
        transform: Transform::default(),
    });

    debug!("I run only once!");

    Ok(())
}

fn rotate_model(
    world: SubWorld<&mut Transform>,
){
    for (_, mut t) in &mut world.query::<&mut Transform>() {
        t.rotation *= UnitQuaternion::from_axis_angle(&Unit::new_normalize(Vector3::new(0.0, 1.0, 0.0)), 0.05);
    }

    debug!("I run in loop!");
}
 
fn create_camera(
    mut cmd: Write<CommandBuffer>,
){
    cmd.spawn(CameraBundle{
        camera: 
            Camera::builder()
                .is_active(true)
                .camera_type(CameraType::LookAt)
                .build(),
        transform: Transform::default(),
    });
}
```

*/

#![cfg_attr(docsrs, feature(doc_cfg))]
// #![warn(missing_docs)]
// TODO: write full documentation for all components

#[cfg(all(feature = "egui", not(feature = "render")))]
compile_error!("Feature \"render\" must be enabled in order to use \"egui\"!");

#[cfg(all(feature = "gltf", not(feature = "render")))]
compile_error!("Feature \"render\" must be enabled in order to use \"gltf\"!");

use std::any::TypeId;

use crate::scripting::*;
use crate::assets::*;
use crate::ecs::*;
use crate::physics::*;
use crate::time::*;
#[cfg(feature = "render")]
use crate::render::{
    Icon,
    renderer::{Renderer, RenderType},
};

/// Submodules and structures to work with graphics
#[cfg(feature = "render")]
pub mod render;
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
/// [Rapier3D](https://crates.io/crates/rapier3d) implementations
pub mod physics;
/// [Mlua](https://crates.io/crates/mlua) scripting implementations
pub mod scripting;
/// Audio processing components
pub mod audio;
/// Bundle of all essential components of the engine
pub mod prelude;

/// Error handler from `error` module
pub use crate::error::Result;
pub use log::{error, warn, info, debug, trace, log};

/// Main struct representing a game engine instance with various fields and functionality
pub struct Sonja {
    /// Game world containing entities and components. Can be serialized and deserialized
    pub world: World,
    /// Lua script manager
    pub lua_manager: LuaManager,
    /// System schedule builders collection, which can be accessed by name
    pub schedules: Schedules,
    /// Function that defines the game loop and handles game execution. It takes an instance of Sonja as an argument
    pub runner: Box<dyn Fn(&mut Sonja)>,
    /// Collection of event handlers for managing user input and system events
    pub events: Events,
    /// Handler for managing the physics simulation within the game
    pub physics_handler: PhysicsHandler,
    /// A handler for managing game time and timing-related operations
    pub time_handler: Time,
    /// Asset manager for loading, managing, and accessing game assets such as textures, sounds, and materials
    pub asset_manager: AssetManager,
    /// Applied extension types
    pub extensions: Vec<TypeId>,
    /// Builder for configuring the game window properties: size, title, window mode etc
    pub window_builder: WindowBuilder,
    /// Rendering context for managing render pipeline and Vulkan components
    #[cfg(feature = "render")]
    pub renderer: Renderer,
}

impl Default for Sonja {
    fn default() -> Self {
        Sonja::init(WindowBuilder::default())
    }
}

impl Sonja {
    /// Initialize Sonja application
    pub fn init(window_builder: WindowBuilder) -> Sonja {
        if window_builder.init_logger {
            init_logger();
        }
        
        Sonja {
            world: World::new(),
            lua_manager: LuaManager::new(),
            schedules: Schedules::from([
                ("setup", Schedule::builder()),
                ("update", Schedule::builder()),
            ]),
            runner: Box::new(default_runner),
            events: Events::new(),
            physics_handler: PhysicsHandler::new(),
            time_handler: Time::new(),
            asset_manager: AssetManager::new(window_builder.cast_count, window_builder.listener_count),
            extensions: vec![],
            window_builder: window_builder.clone(),
            #[cfg(feature = "render")]
            renderer: Renderer::init(window_builder).expect("Cannot create renderer"),
        }
    }
    
    /// Add setup system to schedule
    pub fn add_setup_system<Args, Ret, S>(&mut self, system: S) -> &mut Self 
    where
        S: 'static + System<Args, Ret> + Send,
    {
        self.schedules.get_mut("setup").unwrap().add_system(system);
        self
    }

    /// Add cyclical system to schedule
    pub fn add_system<Args, Ret, S>(&mut self, system: S) -> &mut Self 
    where
        S: 'static + System<Args, Ret> + Send,
    {
        self.schedules.get_mut("update").unwrap().add_system(system);
        self
    }

    pub fn flush_setup_systems(&mut self) -> &mut Self {
        self.schedules.get_mut("setup").unwrap().flush();
        self
    }

    pub fn flush_systems(&mut self) -> &mut Self {
        self.schedules.get_mut("update").unwrap().flush();
        self
    }
    
    /// Use default engine systems, including processing of physics, time, lights and rendering. 
    /// To process rendering `render` feature must be enabled. You can manually add necessary
    /// ones using [`systems`] module
    pub fn default_systems(&mut self) -> &mut Self {
        self.schedules.get_mut("setup").unwrap()
            .add_system(main_setup);
        
        self.schedules.get_mut("update").unwrap()
            .add_system(time_system)
            .add_system(update_physics)
            .add_system(processing_audio);
            
        #[cfg(feature = "render")]
        self.schedules.get_mut("update").unwrap()
            .add_system(update_models_system)
            .add_system(rendering_system)
            .add_system(update_lights)
            .add_system(generate_textures);
            
        self
    }

    pub fn add_events<E: Event>(&mut self) -> &mut Self {
        self.events.push_handler(EventHandler::<E>::new());
        self
    }

    /// Set custom game runner. Default is [`default_runner`]
    pub fn set_runner(&mut self, runner: Box<dyn Fn(&mut Sonja)>) -> &mut Self {
        self.runner = runner;
        self
    }

    /// Apply [`Extension`] to the application. Only **one** instance of a concrete 
    /// extension is allowed, otherwise non-panic error is logged
    pub fn apply_extension<Ext: Extension + 'static>(&mut self, ext: Ext) -> &mut Self {
        if self.extensions.contains(&TypeId::of::<Ext>()) {
            log::error!("Extension \"{}\" is already bound!", std::any::type_name::<Ext>());
            return self;
        }

        ext.apply(self);
        self.extensions.push(TypeId::of::<Ext>());
        self
    }
    
    /// Run main event loop
    pub fn run(&mut self) {
        let runner = std::mem::replace(&mut self.runner, Box::new(empty_runner));
        runner(self);
    }
}

impl Drop for Sonja {
    fn drop(&mut self) {
        #[cfg(feature = "render")]
        self.asset_manager.cleanup(&mut self.renderer);
        #[cfg(feature = "render")]
        self.renderer.cleanup(&mut self.world);
    }
}

fn init_logger() {
    #[cfg(debug_assertions)]
    pretty_env_logger::formatted_builder()
        .filter_level(
            env_logger::filter::Builder::new()
                .parse(&std::env::var("SONJA_LOG").unwrap_or(String::from("SONJA_LOG=debug")))
                .build()
                .filter()
        )
        .init();
        
    #[cfg(not(debug_assertions))]
    pretty_env_logger::formatted_builder()
        .filter_level(
            env_logger::filter::Builder::new()
                .parse(&std::env::var("SONJA_LOG").unwrap_or(String::from("SONJA_LOG=info")))
                .build()
                .filter()
        )
        .init();
}

/// Builder struct for creating window configurations. It's taken as an argument during [`Sonja`] initializing
#[derive(Debug, Clone)]
pub struct WindowBuilder {
    // === WINDOW SETTINGS ===
    /// Title of the window
    pub title: &'static str, 
    /// Width of the window
    pub width: f32,
    /// Height of the window
    pub height: f32,
    /// Specifies whether the window should be fullscreen or windowed
    pub fullscreen: bool,
    /// Specifies whether the window is maximized on startup
    pub maximized: bool,
    /// Specifies whether the window should be resizable
    pub resizable: bool,
    /// Specifies whether the logger must be initialized
    pub init_logger: bool,

    // === AUDIO SETTINGS ===
    /// Maximum count of audio listeners
    pub listener_count: usize,
    /// Maximum count of audio casts
    pub cast_count: usize,

    // === RENDERING SETTINGS ===
    /// Window clear background color:
    #[cfg(feature = "render")]
    pub clear_color: nalgebra::Vector3<f32>,
    /// Icon of the winit window. Requires feature `render` enabled
    #[cfg(feature = "render")]
    pub icon: Option<Icon>,
    /// Type of renderer to use for rendering the window. Can be `Forward` or `Deferred` (WIP). Requires feature `render` enabled
    #[cfg(feature = "render")]
    pub renderer: RenderType,
    /// Maximum numbers of textures pushed to Descriptor Sets. Default value is 4096. Requires feature `render` enabled
    #[cfg(feature = "render")]
    pub textures_count: u32,
}

impl Default for WindowBuilder {
    fn default() -> Self {
        WindowBuilder { 
            title: "My Game", 
            width: 800.0, 
            height: 600.0, 
            fullscreen: false, 
            maximized: false, 
            resizable: true, 
            init_logger: true, 
            listener_count: 8, 
            cast_count: 128, 
            #[cfg(feature = "render")]
            clear_color: nalgebra::Vector3::new(0.0, 0.0, 0.0), 
            #[cfg(feature = "render")]
            icon: None, 
            #[cfg(feature = "render")]
            renderer: RenderType::Forward, 
            #[cfg(feature = "render")]
            textures_count: 4096, 
        }
    }
}

/// [`Sonja`] application extension trait for fast configuration without writing boileplate
pub trait Extension {
    fn apply(&self, app: &mut Sonja);
}