use std::any::Any;
use nalgebra as na;
use hecs::*;

use crate::render::{
	backend::{
		buffer::Buffer,
		pipeline::{Pipeline, ShaderInputAttribute},
	},
	transform::Transform,
	debug::Debug,
};

/// Struct that handles vertex information
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
	pub position: [f32; 3],
	pub normal: [f32; 3],
	pub texcoord: [f32; 2],
}

impl Vertex {
	/// Get middle point between two vertices
	pub fn midpoint(a: &Vertex, b: &Vertex) -> Vertex {
		Vertex {
			position: [
				0.5 * (a.position[0] + b.position[0]),
				0.5 * (a.position[1] + b.position[1]),
				0.5 * (a.position[2] + b.position[2]),
			],
			normal: Self::normalize([
				0.5 * (a.normal[0] + b.normal[0]),
				0.5 * (a.normal[1] + b.normal[1]),
				0.5 * (a.normal[2] + b.normal[2]),
			]),
			texcoord: [
				0.5 * (a.texcoord[0] + b.texcoord[0]),
				0.5 * (a.texcoord[1] + b.texcoord[1]),
			],
		}
	}
	
	/// Normalize vector/vertex. Returns vector with the same direction and `1` lenght
	pub fn normalize(v: [f32; 3]) -> [f32; 3] {
		let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
		[v[0] / l, v[1] / l, v[2] / l]
	}
}

/// Model mesh struct
pub struct Mesh {
	pub vertexdata: Vec<Vertex>,
	pub indexdata: Vec<u32>,
	
	pub(crate) vertexbuffer: Option<Buffer>,
	pub(crate) instancebuffer: Option<Buffer>,
	pub(crate) indexbuffer: Option<Buffer>,
}

impl Mesh {
	/// Create plane mesh
	pub fn plane() -> Mesh {
		Mesh {
			vertexdata: vec![
				Vertex {
					position: [-1.0, 1.0, 0.0],
					normal: Vertex::normalize([-1.0, 1.0, 0.0]),
					texcoord: [0.0, 1.0],
				},
				Vertex {
					position: [-1.0, -1.0, 0.0],
					normal: Vertex::normalize([-1.0, -1.0, 0.0]),
					texcoord: [0.0, 0.0],
				},
				Vertex {
					position: [1.0, 1.0, 0.0],
					normal: Vertex::normalize([1.0, 1.0, 0.0]),
					texcoord: [1.0, 1.0],
				},
				Vertex {
					position: [1.0, -1.0, 0.0],
					normal: Vertex::normalize([1.0, -1.0, 0.0]),
					texcoord: [1.0, 0.0],
				}
			],
			indexdata: vec![0, 2, 1, 1, 2, 3],
			vertexbuffer: None,
			instancebuffer: None,
			indexbuffer: None,
		}
	}
	
	/// Load model from `.obj` file
	pub fn load_obj<P>(path: P) -> Vec<Mesh>
	where 
		P: AsRef<std::path::Path> + std::fmt::Debug
	{
		let (models, _) = tobj::load_obj(
			path,
			&tobj::LoadOptions::default(),
		).expect("Cannot load OBJ file");
		
		let mut meshes = Vec::<Mesh>::new();
		
		for m in models {
			let mut vertexdata = Vec::<Vertex>::new();
			let indexdata = m.mesh.indices;
			
			Debug::info(format!(
				"Positions: {}; Normals: {}; Texcoords: {}",
				m.mesh.positions.len(),
				m.mesh.normals.len(),
				m.mesh.texcoords.len(),
			).as_str());
			
			for i in 0..m.mesh.positions.len() / 3 {				
				let normal: [f32; 3];
				let texcoord: [f32; 2];
				
				let position = [
					m.mesh.positions[i*3],
					m.mesh.positions[i*3+1],
					m.mesh.positions[i*3+2],
				];
				
				normal = Vertex::normalize(position);
				
				if i*2 < m.mesh.texcoords.len() {
					texcoord = [
						m.mesh.texcoords[i*2],
						m.mesh.texcoords[i*2+1],
					];
				} else {
					texcoord = [0.0; 2];
				}
				
				vertexdata.push(Vertex {
					position,
					normal,
					texcoord,
				});
			}
			
			meshes.push(Mesh {
				vertexdata,
				indexdata,
				
				vertexbuffer: None,
				instancebuffer: None,
				indexbuffer: None,
			});
		}
		
		return meshes;
	}
}

impl std::fmt::Debug for Mesh {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Mesh")
			.field("vertexdata", &self.vertexdata)
			.field("indexdata", &self.indexdata)
			.finish()
	}
}

/// Trait for materials to be used in [`ModelBundle`]
pub trait Material {
	fn pipeline(renderer: &Renderer) -> Pipeline;	
}

/// Default material, which uses standard shader and graphics pipeline
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct DefaultMat {
	pub texture_id: u32,
	pub metallic: f32,
	pub roughness: f32,
}

impl DefaultMat {
	/// Create new instance of default material
	pub fn new(
		texture_id: usize,
		metallic: f32,
		roughness: f32,
	) -> DefaultMat {
		DefaultMat {
			texture_id: texture_id as u32,
			metallic,
			roughness,
		}
	}
}

impl Material for DefaultMat {
	fn pipeline(renderer: &Renderer) -> Pipeline {
		let vertex_shader = vk::ShaderModuleCreateInfo::builder()
			.code(vk_shader_macros::include_glsl!(
				"./shaders/vertex_combined.glsl", 
				kind: vert,
			));
		
		let fragment_shader = vk::ShaderModuleCreateInfo::builder()
			.code(vk_shader_macros::include_glsl!(
				"./shaders/fragment_combined.glsl",
				kind: frag,
			));
			
		let instance_attributes = vec![
			ShaderInputAttribute {
				binding: 1,
				location: 3,
				offset: 0,
				format: vk::Format::R32G32B32A32_SFLOAT,
			},
			ShaderInputAttribute {
				binding: 1,
				location: 4,
				offset: 16,
				format: vk::Format::R32G32B32A32_SFLOAT,
			},
			ShaderInputAttribute {
				binding: 1,
				location: 5,
				offset: 32,
				format: vk::Format::R32G32B32A32_SFLOAT,
			},
			ShaderInputAttribute {
				binding: 1,
				location: 6,
				offset: 48,
				format: vk::Format::R32G32B32A32_SFLOAT,
			},
			ShaderInputAttribute {
				binding: 1,
				location: 7,
				offset: 64,
				format: vk::Format::R32G32B32A32_SFLOAT,
			},
			ShaderInputAttribute {
				binding: 1,
				location: 8,
				offset: 80,
				format: vk::Format::R32G32B32A32_SFLOAT,
			},
			ShaderInputAttribute {
				binding: 1,
				location: 9,
				offset: 96,
				format: vk::Format::R32G32B32A32_SFLOAT,
			},
			ShaderInputAttribute {
				binding: 1,
				location: 10,
				offset: 112,
				format: vk::Format::R32G32B32A32_SFLOAT,
			},
			ShaderInputAttribute{
				binding: 1,
				location: 11,
				offset: 128,
				format: vk::Format::R8G8B8A8_UINT,
			},
			ShaderInputAttribute{
				binding: 1,
				location: 12,
				offset: 132,
				format: vk::Format::R32_SFLOAT,
			},
			ShaderInputAttribute{
				binding: 1,
				location: 13,
				offset: 136,
				format: vk::Format::R32_SFLOAT,
			}
		];
		
		Pipeline::init(
			&renderer,
			&vertex_shader,
			&fragment_shader,
			instance_attributes,
			140,
		)
	}
}

/// ECS model bundle
#[derive(Bundle)]
pub struct ModelBundle<M: Material> {
	pub mesh: Mesh,
	pub material: M,
	pub transform: Transform,
}

//~ impl Model<Vertex, InstanceData> {
	//~ pub fn cube() -> Model<Vertex, InstanceData> {
		//~ let lbf = [-1.0,1.0,0.0]; // Left-bottom-front
		//~ let lbb = [-1.0,1.0,2.0]; // Left-bottom-back
		//~ let ltf = [-1.0,-1.0,0.0];// Left-top-front
		//~ let ltb = [-1.0,-1.0,2.0];// Left-top-back
		//~ let rbf = [1.0,1.0,0.0];  // Right-bottom-front
		//~ let rbb = [1.0,1.0,2.0];  // Right-bottom-back
		//~ let rtf = [1.0,-1.0,0.0]; // Right-top-front
		//~ let rtb = [1.0,-1.0,2.0]; // Right-top-back

		//~ Model {
			//~ vertexdata: vec![
				//~ Vertex {
					//~ position: lbf,
					//~ normal: Vertex::normalize(lbf),
				//~ },
				//~ Vertex {
					//~ position: lbb,
					//~ normal: Vertex::normalize(lbb),
				//~ },
				//~ Vertex {
					//~ position: ltf,
					//~ normal: Vertex::normalize(ltf),
				//~ },
				//~ Vertex {
					//~ position: ltb,
					//~ normal: Vertex::normalize(ltb),
				//~ },
				//~ Vertex {
					//~ position: rbf,
					//~ normal: Vertex::normalize(rbf),
				//~ },
				//~ Vertex {
					//~ position: rbb,
					//~ normal: Vertex::normalize(rbb),
				//~ },
				//~ Vertex {
					//~ position: rtf,
					//~ normal: Vertex::normalize(rtf),
				//~ },
				//~ Vertex {
					//~ position: rtb,
					//~ normal: Vertex::normalize(rtb),
				//~ },
			//~ ],
			//~ indexdata: vec![
				//~ 0, 1, 5, 0, 5, 4, //bottom
				//~ 2, 7, 3, 2, 6, 7, //top
				//~ 0, 6, 2, 0, 4, 6, //front
				//~ 1, 3, 7, 1, 7, 5, //back
				//~ 0, 2, 1, 1, 2, 3, //left
				//~ 4, 5, 6, 5, 7, 6, //right
			//~ ],
			//~ handle_to_index: HashMap::new(),
			//~ handles: Vec::new(),
			//~ instances: Vec::new(),
			//~ first_invisible: 0,
			//~ next_handle: 0,
			//~ vertexbuffer: None,
			//~ instancebuffer: None,
			//~ indexbuffer: None,
		//~ }
	//~ }
	
	//~ pub fn icosahedron() -> Model<Vertex, InstanceData> {
		//~ let phi = (1.0 + 5.0_f32.sqrt()) / 2.0;
		//~ Model {
			//~ vertexdata: vec![
				//~ Vertex {
					//~ position: [phi, -1.0, 0.0],
					//~ normal: Vertex::normalize([phi, -1.0, 0.0]),
				//~ },
				//~ Vertex {
					//~ position: [phi, 1.0, 0.0],
					//~ normal: Vertex::normalize([phi, 1.0, 0.0]),
				//~ },
				//~ Vertex {
					//~ position: [-phi, -1.0, 0.0],
					//~ normal: Vertex::normalize([-phi, -1.0, 0.0]),
				//~ },
				//~ Vertex {
					//~ position: [-phi, 1.0, 0.0],
					//~ normal: Vertex::normalize([-phi, 1.0, 0.0]),
				//~ },
				//~ Vertex {
					//~ position: [1.0, 0.0, -phi],
					//~ normal: Vertex::normalize([1.0, 0.0, -phi]),
				//~ },
				//~ Vertex {
					//~ position: [-1.0, 0.0, -phi],
					//~ normal: Vertex::normalize([-1.0, 0.0, -phi]),
				//~ },
				//~ Vertex {
					//~ position: [1.0, 0.0, phi],
					//~ normal: Vertex::normalize([1.0, 0.0, phi]),
				//~ },
				//~ Vertex {
					//~ position: [-1.0, 0.0, phi],
					//~ normal: Vertex::normalize([-1.0, 0.0, phi]),
				//~ },
				//~ Vertex {
					//~ position: [0.0, -phi, -1.0],
					//~ normal: Vertex::normalize([0.0, -phi, -1.0]),
				//~ },
				//~ Vertex {
					//~ position: [0.0, -phi, 1.0],
					//~ normal: Vertex::normalize([0.0, -phi, 1.0]),
				//~ },
				//~ Vertex {
					//~ position: [0.0, phi, -1.0],
					//~ normal: Vertex::normalize([0.0, phi, -1.0]),
				//~ },
				//~ Vertex {
					//~ position: [0.0, phi, 1.0],
					//~ normal: Vertex::normalize([0.0, phi, 1.0]),
				//~ },
			//~ ],
			//~ indexdata: vec![
				//~ 0,9,8,//
				//~ 0,8,4,//
				//~ 0,4,1,//
				//~ 0,1,6,//
				//~ 0,6,9,//
				//~ 8,9,2,//
				//~ 8,2,5,//
				//~ 8,5,4,//
				//~ 4,5,10,//
				//~ 4,10,1,//
				//~ 1,10,11,//
				//~ 1,11,6,//
				//~ 2,3,5,//
				//~ 2,7,3,//
				//~ 2,9,7,//
				//~ 5,3,10,//
				//~ 3,11,10,//
				//~ 3,7,11,//
				//~ 6,7,9,//
				//~ 6,11,7//
			//~ ],

			//~ handle_to_index: HashMap::new(),
			//~ handles: Vec::new(),
			//~ instances: Vec::new(),
			//~ first_invisible: 0,
			//~ next_handle: 0,
			//~ vertexbuffer: None,
			//~ instancebuffer: None,
			//~ indexbuffer: None,
		//~ }
	//~ }
	
	//~ pub fn sphere(refinements: u32) -> Model<Vertex, InstanceData> {
		//~ // New icosahedron
		//~ let mut model = Model::icosahedron();
		//~ // Subdivide faces
		//~ for _ in 0..refinements{
			//~ model.refine();
		//~ }
		//~ // Align vertices to equal distance to sphere's center
		//~ for v in &mut model.vertexdata {
			//~ v.position = Vertex::normalize(v.position);
		//~ }
		//~ return model;
	//~ }
	
	//~ // Triangle subdividing
	//~ pub fn refine(&mut self) {
		//~ let mut new_indices = vec![];
		//~ let mut midpoints = std::collections::HashMap::<(u32, u32), u32>::new();
		//~ for triangle in self.indexdata.chunks(3) {
			//~ let a = triangle[0];
			//~ let b = triangle[1];
			//~ let c = triangle[2];
			//~ let vertex_a = self.vertexdata[a as usize];
			//~ let vertex_b = self.vertexdata[b as usize];
			//~ let vertex_c = self.vertexdata[c as usize];
			//~ let mab = if let Some(ab) = midpoints.get(&(a, b)) {
				//~ *ab
			//~ } else {
				//~ let vertex_ab = Vertex::midpoint(&vertex_a, &vertex_b);
				//~ let mab = self.vertexdata.len() as u32;
				//~ self.vertexdata.push(vertex_ab);
				//~ midpoints.insert((a, b), mab);
				//~ midpoints.insert((b, a), mab);
				//~ mab
			//~ };
			//~ let mbc = if let Some(bc) = midpoints.get(&(b, c)) {
				//~ *bc
			//~ } else {
				//~ let vertex_bc = Vertex::midpoint(&vertex_b, &vertex_c);
				//~ let mbc = self.vertexdata.len() as u32;
				//~ midpoints.insert((b, c), mbc);
				//~ midpoints.insert((c, b), mbc);
				//~ self.vertexdata.push(vertex_bc);
				//~ mbc
			//~ };
			//~ let mca = if let Some(ca) = midpoints.get(&(c, a)) {
				//~ *ca
			//~ } else {
				//~ let vertex_ca = Vertex::midpoint(&vertex_c, &vertex_a);
				//~ let mca = self.vertexdata.len() as u32;
				//~ midpoints.insert((c, a), mca);
				//~ midpoints.insert((a, c), mca);
				//~ self.vertexdata.push(vertex_ca);
				//~ mca
			//~ };
			//~ new_indices.extend_from_slice(&[mca, a, mab, mab, b, mbc, mbc, c, mca, mab, mbc, mca]);
		//~ }
		//~ self.indexdata = new_indices;
	//~ }
//~ }
