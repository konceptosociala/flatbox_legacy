pub mod backend;
pub mod pbr;
pub mod ui;
pub mod renderer;
pub mod debug;
pub mod screenshot;

pub use screenshot::*;
pub use renderer::*;
pub use debug::*;

#[cfg(feature = "egui")]
pub use ui::*;

pub use pbr::camera::{Camera, CameraBuilder, CameraBundle, CameraType};
pub use pbr::model::{Mesh, MeshType, Model, ModelBundle, ModelBundleBuilder};
pub use pbr::texture::*;
pub use pbr::light::*;
pub use pbr::material::*;
pub use pbr::color::*;
pub use pbr::skybox::*;

pub use backend::shader::*;
pub use backend::pipeline::*;
pub use backend::window::*;

pub use winit::event::VirtualKeyCode as KeyCode;
pub use winit::window::Icon;

pub use vk_shader_macros::include_glsl;
