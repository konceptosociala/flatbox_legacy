pub mod backend;
pub mod pbr;
pub mod ui;
pub mod renderer;
pub mod debug;
pub mod screenshot;

pub use screenshot::ScreenshotExt;
pub use renderer::{PipelineCollection, RenderType, UniformBuffersCollection, Renderer};
pub use debug::Debug;

#[cfg(feature = "egui")]
pub use ui::{GuiContext, GuiEvent, GuiHandler, Key, gui};

pub use pbr::camera::{Camera, CameraBuilder, CameraBundle, CameraType};
pub use pbr::model::{Mesh, MeshType, Model, ModelBundle, ModelBundleBuilder};
pub use pbr::texture::{Texture, TextureLoadType, TextureType, Filter};
pub use pbr::light::{DirectionalLight, PointLight};
pub use pbr::material::{Material, DefaultMat, DefaultMatBuilder, CachedMaterials};
pub use pbr::color::Color;
pub use pbr::skybox::{SkyBox, SkyBoxMat};

pub use backend::shader::{ShaderInput, ShaderInputAttribute, ShaderInputFormat, ShaderTopology};
pub use backend::pipeline::Pipeline;
pub use backend::window::{WinitFullscreen, Window};

pub use winit::event::VirtualKeyCode as KeyCode;
pub use winit::window::Icon;

pub use vk_shader_macros::include_glsl;
