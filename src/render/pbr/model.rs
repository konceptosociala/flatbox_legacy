use std::fmt::{self, Debug};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use serde::{
    Serialize, 
    Deserialize,
    Serializer, 
    Deserializer, 
    de::*,
    de::Error as DeError,
    ser::SerializeStruct,
};
use nalgebra as na;

use crate::{
    render::backend::buffer::Buffer,
    error::FlatboxResult,
};

use crate::assets::AssetHandle;
use crate::ecs::*;
use crate::math::transform::Transform;

/// Struct that handles vertex information
#[repr(C)]
#[derive(Default, Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
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

/// Represents the type of mesh in [`Model`] struct.
/// It indicates whether mesh must be created in runtime,
/// loaded from file (or resource) or created manually
/// with index and vertex buffers.
#[derive(Clone, Default, Debug, PartialEq, Hash, Serialize, Deserialize)]
pub enum MeshType {
    /// Plane mesh (textured)
    Plane,
    /// Cube mesh
    #[default]
    Cube,
    /// Icosphere mesh
    Icosahedron,
    /// Refined icosphere mesh
    Sphere,
    /// Mesh which have been loaded from file or resource
    Loaded(PathBuf),
    /// Custom model type, which neither loaded from file, nor
    /// created in runtime. Unlike other meshes it's (de-)serialized.
    /// Use it when constructing models manually
    Generic,
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
    pub fn new(
        vertexdata: &[Vertex], 
        indexdata: &[u32],
    ) -> Self {
        Mesh {
            vertexdata: vertexdata.to_vec(),
            indexdata: indexdata.to_vec(),
            vertexbuffer: None,
            instancebuffer: None,
            indexbuffer: None,
        }
    }
    /// Create a textured plane mesh
    pub fn plane() -> Self {
        let p1 = na::Point3::new(-1.0, 1.0, 0.0);
        let p2 = na::Point3::new(1.0, 1.0, 0.0);
        let p3 = na::Point3::new(-1.0, -1.0, 0.0);
        
        let v1 = p2 - p1;
        let v2 = p3 - p1;
        let normal: [f32; 3] = v1.cross(&v2).into();
                
        Mesh {
            vertexdata: vec![
                Vertex {
                    position: [-1.0, 1.0, 0.0],
                    normal,
                    texcoord: [0.0, 1.0],
                },
                Vertex {
                    position: [-1.0, -1.0, 0.0],
                    normal,
                    texcoord: [0.0, 0.0],
                },
                Vertex {
                    position: [1.0, 1.0, 0.0],
                    normal,
                    texcoord: [1.0, 1.0],
                },
                Vertex {
                    position: [1.0, -1.0, 0.0],
                    normal,
                    texcoord: [1.0, 0.0],
                }
            ],
            indexdata: vec![0, 2, 1, 1, 2, 3],
            vertexbuffer: None,
            instancebuffer: None,
            indexbuffer: None,
        }
    }
    
    /// Create untextured cube mesh
    pub fn cube() -> Self {
        let lbf = [-1.0,1.0,0.0]; // Left-bottom-front
        let lbb = [-1.0,1.0,2.0]; // Left-bottom-back
        let ltf = [-1.0,-1.0,0.0];// Left-top-front
        let ltb = [-1.0,-1.0,2.0];// Left-top-back
        let rbf = [1.0,1.0,0.0];  // Right-bottom-front
        let rbb = [1.0,1.0,2.0];  // Right-bottom-back
        let rtf = [1.0,-1.0,0.0]; // Right-top-front
        let rtb = [1.0,-1.0,2.0]; // Right-top-back

        Mesh {
            vertexdata: vec![
                Vertex {
                    position: lbf,
                    normal: Vertex::normalize(lbf),
                    ..Default::default()
                },
                Vertex {
                    position: lbb,
                    normal: Vertex::normalize(lbb),
                    ..Default::default()
                },
                Vertex {
                    position: ltf,
                    normal: Vertex::normalize(ltf),
                    ..Default::default()
                },
                Vertex {
                    position: ltb,
                    normal: Vertex::normalize(ltb),
                    ..Default::default()
                },
                Vertex {
                    position: rbf,
                    normal: Vertex::normalize(rbf),
                    ..Default::default()
                },
                Vertex {
                    position: rbb,
                    normal: Vertex::normalize(rbb),
                    ..Default::default()
                },
                Vertex {
                    position: rtf,
                    normal: Vertex::normalize(rtf),
                    ..Default::default()
                },
                Vertex {
                    position: rtb,
                    normal: Vertex::normalize(rtb),
                    ..Default::default()
                },
            ],
            indexdata: vec![
                0, 1, 5, 0, 5, 4, //bottom
                2, 7, 3, 2, 6, 7, //top
                0, 6, 2, 0, 4, 6, //front
                1, 3, 7, 1, 7, 5, //back
                0, 2, 1, 1, 2, 3, //left
                4, 5, 6, 5, 7, 6, //right
            ],
            vertexbuffer: None,
            instancebuffer: None,
            indexbuffer: None,
        }
    }
    
    /// Create ico sphere mesh
    pub fn icosahedron() -> Self {
        let phi = (1.0 + 5.0_f32.sqrt()) / 2.0;
        
        Mesh {
            vertexdata: vec![
                Vertex {
                    position: [phi, -1.0, 0.0],
                    normal: Vertex::normalize([phi, -1.0, 0.0]),
                    ..Default::default()
                },
                Vertex {
                    position: [phi, 1.0, 0.0],
                    normal: Vertex::normalize([phi, 1.0, 0.0]),
                    ..Default::default()
                },
                Vertex {
                    position: [-phi, -1.0, 0.0],
                    normal: Vertex::normalize([-phi, -1.0, 0.0]),
                    ..Default::default()
                },
                Vertex {
                    position: [-phi, 1.0, 0.0],
                    normal: Vertex::normalize([-phi, 1.0, 0.0]),
                    ..Default::default()
                },
                Vertex {
                    position: [1.0, 0.0, -phi],
                    normal: Vertex::normalize([1.0, 0.0, -phi]),
                    ..Default::default()
                },
                Vertex {
                    position: [-1.0, 0.0, -phi],
                    normal: Vertex::normalize([-1.0, 0.0, -phi]),
                    ..Default::default()
                },
                Vertex {
                    position: [1.0, 0.0, phi],
                    normal: Vertex::normalize([1.0, 0.0, phi]),
                    ..Default::default()
                },
                Vertex {
                    position: [-1.0, 0.0, phi],
                    normal: Vertex::normalize([-1.0, 0.0, phi]),
                    ..Default::default()
                },
                Vertex {
                    position: [0.0, -phi, -1.0],
                    normal: Vertex::normalize([0.0, -phi, -1.0]),
                    ..Default::default()
                },
                Vertex {
                    position: [0.0, -phi, 1.0],
                    normal: Vertex::normalize([0.0, -phi, 1.0]),
                    ..Default::default()
                },
                Vertex {
                    position: [0.0, phi, -1.0],
                    normal: Vertex::normalize([0.0, phi, -1.0]),
                    ..Default::default()
                },
                Vertex {
                    position: [0.0, phi, 1.0],
                    normal: Vertex::normalize([0.0, phi, 1.0]),
                    ..Default::default()
                },
            ],
            indexdata: vec![
                0,9,8,
                0,8,4,
                0,4,1,
                0,1,6,
                0,6,9,
                8,9,2,
                8,2,5,
                8,5,4,
                4,5,10,
                4,10,1,
                1,10,11,
                1,11,6,
                2,3,5,
                2,7,3,
                2,9,7,
                5,3,10,
                3,11,10,
                3,7,11,
                6,7,9,
                6,11,7
            ],
            vertexbuffer: None,
            instancebuffer: None,
            indexbuffer: None,
        }
    }
    
    /// Create untextured sphere mesh with given resolution
    pub fn sphere() -> Self {
        let mut sphere = Mesh::icosahedron();
        
        for _ in 0..2{
            sphere.refine();
        }

        for v in &mut sphere.vertexdata {
            v.position = Vertex::normalize(v.position);
        }
        
        sphere
    }
    
    /// Load model from `.obj` file
    pub fn load_obj<P>(path: P) -> Vec<Self>
    where 
        P: AsRef<Path> + Debug
    {
        let (models, _) = tobj::load_obj(
            path,
            &tobj::LoadOptions {
                single_index: true,
                triangulate: true,
                ignore_points: true,
                ..Default::default()
            },
        ).expect("Cannot load OBJ file");
        
        let mut meshes = Vec::<Mesh>::new();
        
        for m in models {
            let mut vertexdata = Vec::<Vertex>::new();
            let indexdata = m.mesh.indices;
            
            for i in 0..m.mesh.positions.len() / 3 {                
                let texcoord: [f32; 2];
                
                let position = [
                    m.mesh.positions[i*3],
                    m.mesh.positions[i*3+1],
                    m.mesh.positions[i*3+2],
                ];
                
                let normal = [
                    m.mesh.normals[i*3],
                    m.mesh.normals[i*3+1],
                    m.mesh.normals[i*3+2],
                ];
                
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
    
    /// Subdivide mesh triangles
    pub fn refine(&mut self) {
        let mut new_indices = vec![];
        let mut midpoints = HashMap::new();
        
        for triangle in self.indexdata.chunks(3) {
            let a = triangle[0];
            let b = triangle[1];
            let c = triangle[2];
            
            let vertex_a = self.vertexdata[a as usize];
            let vertex_b = self.vertexdata[b as usize];
            let vertex_c = self.vertexdata[c as usize];
            
            let mab = match midpoints.get(&(a, b)){
                Some(ab) => *ab,
                _ => {
                    let vertex_ab = Vertex::midpoint(&vertex_a, &vertex_b);
                    let mab = self.vertexdata.len() as u32;
                    
                    self.vertexdata.push(vertex_ab);
                    
                    midpoints.insert((a, b), mab);
                    midpoints.insert((b, a), mab);
                    
                    mab
                },
            };
            
            let mbc = match midpoints.get(&(b, c)) {
                Some(bc) => *bc,
                _ => {
                    let vertex_bc = Vertex::midpoint(&vertex_b, &vertex_c);
                    let mbc = self.vertexdata.len() as u32;
                    
                    midpoints.insert((b, c), mbc);
                    midpoints.insert((c, b), mbc);
                    
                    self.vertexdata.push(vertex_bc);
                    
                    mbc
                },
            };
            
            let mca = match midpoints.get(&(c, a)){
                Some(ca) => *ca,
                _ => {
                    let vertex_ca = Vertex::midpoint(&vertex_c, &vertex_a);
                    let mca = self.vertexdata.len() as u32;
                    
                    midpoints.insert((c, a), mca);
                    midpoints.insert((a, c), mca);
                    
                    self.vertexdata.push(vertex_ca);
                    
                    mca
                },
            };
            
            new_indices.extend_from_slice(&[mca, a, mab, mab, b, mbc, mbc, c, mca, mab, mbc, mca]);
        }
        
        self.indexdata = new_indices;
    }
}

impl Default for Mesh {
    fn default() -> Self {
        Mesh::cube()
    }
}

impl Clone for Mesh {
    fn clone(&self) -> Self {
        Mesh {
            vertexdata: self.vertexdata.clone(),
            indexdata: self.indexdata.clone(),
            
            vertexbuffer: None,
            instancebuffer: None,
            indexbuffer: None,
        }
    }
}

impl Debug for Mesh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mesh")
            .field("vertexdata", &self.vertexdata)
            .field("indexdata", &self.indexdata)
            .finish()
    }
}

impl Serialize for Mesh {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut mesh = serializer.serialize_struct("Mesh", 2)?;
        mesh.serialize_field("vertexdata", &self.vertexdata)?;
        mesh.serialize_field("indexdata", &self.indexdata)?;
        mesh.end()
    }
}

impl<'de> Deserialize<'de> for Mesh {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum MeshField { 
            VertexData, 
            IndexData
        }
    
        struct MeshVisitor;
        
        impl<'de> Visitor<'de> for MeshVisitor {
            type Value = Mesh;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Mesh")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Mesh, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let vertexdata = seq.next_element()?.ok_or_else(|| DeError::invalid_length(0, &self))?;
                let indexdata = seq.next_element()?.ok_or_else(|| DeError::invalid_length(1, &self))?;
                
                Ok(Mesh {
                    vertexdata,
                    indexdata,
                    
                    vertexbuffer: None,
                    instancebuffer: None,
                    indexbuffer: None,
                })
            }

            fn visit_map<V>(self, mut map: V) -> Result<Mesh, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut vertexdata = None;
                let mut indexdata = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        MeshField::VertexData => {
                            if vertexdata.is_some() {
                                return Err(DeError::duplicate_field("vertexdata"));
                            }
                            vertexdata = Some(map.next_value()?);
                        }
                        MeshField::IndexData => {
                            if indexdata.is_some() {
                                return Err(DeError::duplicate_field("indexdata"));
                            }
                            indexdata = Some(map.next_value()?);
                        }
                    }
                }
                let vertexdata = vertexdata.ok_or_else(|| DeError::missing_field("vertexdata"))?;
                let indexdata = indexdata.ok_or_else(|| DeError::missing_field("indexdata"))?;
                
                Ok(Mesh {
                    vertexdata,
                    indexdata,
                    
                    vertexbuffer: None,
                    instancebuffer: None,
                    indexbuffer: None,
                })
            }
        }
        
        const FIELDS: &'static [&'static str] = &["vertexdata", "indexdata"];
        deserializer.deserialize_struct("Mesh", FIELDS, MeshVisitor)
    }
}

#[derive(Debug, Clone, Default)]
#[readonly::make]
pub struct Model {
    /// Model mesh type. It can be selected manually and is
    /// readonly during future use
    #[readonly]
    pub mesh_type: MeshType,
    pub mesh: Option<Mesh>,
}

impl Model {
    pub fn load_obj<P>(path: P) -> FlatboxResult<Self> 
    where 
        P: AsRef<Path> + Debug
    {
        let error = format!(
            "Error loading model `{}`: invalid file extension!", 
            path.as_ref().display()
        );

        let extension = path.as_ref()
            .extension()
            .ok_or(crate::Result::yell(&error))?
            .to_str().unwrap();

        let mesh = match extension {
            "obj" => Mesh::load_obj(path.as_ref()).swap_remove(0),
            _ => return Err(crate::Result::yell(&error)),
        };

        Ok(Model {
            mesh_type: MeshType::Loaded(path.as_ref().to_owned()),
            mesh: Some(mesh),
        })
    }

    pub fn plane() -> Self {
        Model {
            mesh_type: MeshType::Plane,
            mesh: Some(Mesh::plane()),
        }
    }

    pub fn cube() -> Self {
        Model {
            mesh_type: MeshType::Cube,
            mesh: Some(Mesh::cube()),
        }
    }

    pub fn icosahedron() -> Self {
        Model {
            mesh_type: MeshType::Icosahedron,
            mesh: Some(Mesh::icosahedron()),
        }
    }

    pub fn sphere() -> Self {
        Model {
            mesh_type: MeshType::Sphere,
            mesh: Some(Mesh::sphere()),
        }
    }
}

impl Serialize for Model {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut model = serializer.serialize_struct("Model", 2)?;
        model.serialize_field("mesh_type", &self.mesh_type)?;

        match self.mesh_type {
            MeshType::Generic => {
                model.serialize_field("mesh", &self.mesh)?;
            },
            _ => {
                model.serialize_field("mesh", &Option::<Mesh>::None)?;
            }
        }

        model.end()
    }
}

impl<'de> Deserialize<'de> for Model {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum ModelField { 
            MeshType,
            Mesh,
        }

        struct ModelVisitor;

        impl<'de> Visitor<'de> for ModelVisitor {
            type Value = Model;
            
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Model")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Model, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mesh_type: MeshType = seq.next_element()?.ok_or_else(|| DeError::invalid_length(0, &self))?;

                let mesh = match mesh_type {
                    MeshType::Cube => { Some(Mesh::cube()) },
                    MeshType::Icosahedron => { Some(Mesh::icosahedron()) },
                    MeshType::Sphere => { Some(Mesh::sphere()) },
                    MeshType::Plane => { Some(Mesh::plane()) },
                    MeshType::Loaded(path) => {
                        return Ok(Model::load_obj(path)
                            .expect("Cannot load deserialized model from path"));
                    },
                    MeshType::Generic => { 
                        seq.next_element()?.ok_or_else(|| DeError::invalid_length(1, &self))? 
                    },
                };

                Ok(Model {
                    mesh_type,
                    mesh,
                })
            }

            fn visit_map<V>(self, mut map: V) -> Result<Model, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut mesh_type: Option<MeshType> = None;
                let mut mesh: Option<Option<Mesh>> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        ModelField::MeshType => {
                            if mesh_type.is_some() {
                                return Err(DeError::duplicate_field("mesh_type"));
                            }
                            mesh_type = Some(map.next_value()?);
                        },
                        ModelField::Mesh => {
                            if mesh.is_some() {
                                return Err(DeError::duplicate_field("mesh"));
                            }
                            mesh = Some(map.next_value()?);
                        },
                    }
                }

                let mesh_type = mesh_type.ok_or_else(|| DeError::missing_field("mesh_type"))?;

                let mesh = match mesh_type {
                    MeshType::Cube => { Some(Mesh::cube()) },
                    MeshType::Icosahedron => { Some(Mesh::icosahedron()) },
                    MeshType::Sphere => { Some(Mesh::sphere()) },
                    MeshType::Plane => { Some(Mesh::plane()) },
                    MeshType::Loaded(path) => {
                        return Ok(Model::load_obj(path)
                            .expect("Cannot load deserialized model from path"));
                    },
                    MeshType::Generic => { 
                        mesh.ok_or_else(|| DeError::missing_field("mesh"))?
                    },
                };

                Ok(Model {
                    mesh_type,
                    mesh,
                })
            }
        }

        const FIELDS: &'static [&'static str] = &[
            "mesh_type",
            "mesh"
        ];
        deserializer.deserialize_struct("Model", FIELDS, ModelVisitor)
    }
}

/// ECS model bundle
#[derive(Bundle, Debug, Clone)]
pub struct ModelBundle {
    pub model: Model,
    pub material: AssetHandle<'M'>,
    pub transform: Transform,
}

impl ModelBundle {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn builder() -> ModelBundleBuilder {
        ModelBundleBuilder::new()
    }
}

impl Default for ModelBundle {
    fn default() -> Self {
        ModelBundle {
            model: Model::cube(),
            material: AssetHandle::new(),
            transform: Transform::default(),
        }
    }
}

pub struct ModelBundleBuilder {
    model: Model,
    material: AssetHandle<'M'>,
    transform: Transform,
}

impl ModelBundleBuilder {
    pub fn new() -> Self {
        ModelBundleBuilder {
            model: Model::cube(),
            material: AssetHandle::default(),
            transform: Transform::default(),
        }
    }
    
    pub fn model(mut self, model: Model) -> Self {
        self.model = model;
        self
    }
    
    pub fn material(mut self, material: AssetHandle<'M'>) -> Self {
        self.material = material;
        self
    }
    
    pub fn transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }
    
    pub fn build(self) -> ModelBundle {
        ModelBundle {
            model: self.model,
            material: self.material,
            transform: self.transform,
        }
    }
}