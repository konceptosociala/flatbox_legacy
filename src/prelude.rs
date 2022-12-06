pub type Debug 		= crate::render::debug::Debug;
pub type Despero 	= crate::Despero;
pub type Camera 	= crate::engine::camera::Camera;
pub type Filter		= crate::engine::texture::Filter;
pub type Renderer	= crate::render::renderer::Renderer;

pub type TexturedVertexData 	= crate::engine::model::TexturedVertexData;
pub type TexturedInstanceData 	= crate::engine::model::TexturedInstanceData;
pub type Model<V, I>			= crate::engine::model::Model<V, I>;

// Bundles
pub type CameraBundle = crate::engine::camera::CameraBundle;

// Math
pub type Transform 	= crate::engine::transform::Transform;
pub type Matrix4	= nalgebra::Matrix4<f32>;
pub type Vector3	= nalgebra::Vector3<f32>;
