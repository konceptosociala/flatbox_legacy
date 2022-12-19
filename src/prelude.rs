pub type Despero	= crate::Despero;

pub type Renderer	= crate::render::renderer::Renderer;
pub type Debug 		= crate::render::debug::Debug;
pub type Transform 	= crate::render::transform::Transform;

pub type Camera 				= crate::render::pbr::camera::Camera;
pub type Filter					= crate::render::pbr::texture::Filter;
pub type DirectionalLight		= crate::render::pbr::light::DirectionalLight;
pub type PointLight				= crate::render::pbr::light::PointLight;
pub type LightManager				= crate::render::pbr::light::LightManager;
pub type TexturedVertexData 	= crate::render::pbr::model::TexturedVertexData;
pub type TexturedInstanceData 	= crate::render::pbr::model::TexturedInstanceData;
pub type Model<V, I>			= crate::render::pbr::model::Model<V, I>;

// Bundles
pub type CameraBundle = crate::render::pbr::camera::CameraBundle;

// Math
pub type Matrix4	= nalgebra::Matrix4<f32>;
pub type Vector3	= nalgebra::Vector3<f32>;
