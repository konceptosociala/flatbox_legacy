//! # Flatbox glTF format loading support
//! 
//! **Limitations:**
//! 
//! * One scene per file
//! * External assets only (binary are not supported yet)
//! * 

use std::path::{PathBuf, Path};

use thiserror::Error;
use serde::{Serialize, Deserialize};
use gltf::image::Source;
use nalgebra as na;
use gltf::{
    Gltf,
    Mesh as GltfMesh,
    Material as GltfMat,
    Texture as GltfTexture,
    scene::Transform as GltfTransform,
};

use crate::error::FlatboxResult;
use crate::assets::asset_manager::AssetManager;
use crate::{
    render::pbr::{
        model::{Mesh, ModelBundle, Vertex},
        material::DefaultMat,
        texture::Filter,
    },
    math::transform::Transform,
};

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum GltfError {
    #[error("GLTF runtime error")]
    RuntimeError(#[from] gltf::Error),
    #[error("Binary glTF assets are not supported")]
    BinaryAsset,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct GltfCache {
    pub scenes: Vec<PathBuf>,
}

pub fn load_gltf<P: AsRef<Path>>(
    path: P,
    assets: &mut AssetManager,
) -> FlatboxResult<Vec<ModelBundle>> {
    let scene = Gltf::open(path).map_err(|e| GltfError::from(e))?.document;

    let mut materials = vec![];

    for material in scene.materials() {
        materials.push(DefaultMat::from_gltf(material, Filter::Linear, assets));
    }

    Ok(vec![])
}

fn load_texture_path(gltf_texture: GltfTexture) -> FlatboxResult<String> {
    let source = gltf_texture.source().source();
    let Source::Uri { uri: path, mime_type: _ } = source else { 
        return Err(crate::Result::from(GltfError::BinaryAsset)) 
    };

    return Ok(path.to_owned());
}

#[cfg_attr(docsrs, doc(cfg(feature = "gltf")))]
impl DefaultMat {
    pub fn from_gltf(
        gltf_mat: GltfMat, 
        filter: Filter,
        assets: &mut AssetManager,
    ) -> FlatboxResult<Self> {
        let pbr = gltf_mat.pbr_metallic_roughness();

        // Color
        let c = pbr.base_color_factor();
        let color = [c[0], c[1], c[2]];
        let albedo = match pbr.base_color_texture() {
            Some(info) => {
                let path = load_texture_path(info.texture())?;
                assets.create_texture(path, filter).unwrap() as u32
            },
            None => 0u32,
        };

        // Metallic
        let metallic = pbr.metallic_factor();
/**/    let metallic_map = 0;

        // Roughness
        let roughness = pbr.roughness_factor();
/**/    let roughness_map = 0;

        // Normal Map
        let (normal, normal_map) = match gltf_mat.normal_texture() {
            Some(nrm) => {
                let scale = nrm.scale(); 
                let path = load_texture_path(nrm.texture())?;
                let texture = assets.create_texture(path, filter).unwrap() as u32;

                (scale, texture as u32)
            },
            None => (1.0, 1),
        };

        // Ambient Occlusion
        let (ao, ao_map) = match gltf_mat.occlusion_texture() {
            Some(ao) => {
                let strength = ao.strength();
                let path = load_texture_path(ao.texture())?;
                let texture = assets.create_texture(path, filter).unwrap() as u32;
                
                (strength, texture)
            },
            None => (1.0, 0),
        };

        Ok(DefaultMat { 
            color, 
            albedo, 
            metallic, 
            metallic_map, 
            roughness, 
            roughness_map, 
            normal, 
            normal_map, 
            ao, 
            ao_map,
        })
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "gltf")))]
impl Mesh {
    pub fn from_gltf(gltf_mesh: GltfMesh) -> Self {
        let mut mesh = Mesh::new(&[], &[]);

        for prim in gltf_mesh.primitives() {
            
        }

        todo!();
    }
}

impl From<GltfTransform> for Transform {
    fn from(gltf_transform: GltfTransform) -> Self {
        let (t, r, s) = gltf_transform.decomposed();
        
        let translation = na::Vector3::new(t[0], t[1], t[2]);
        let rotation = na::Unit::new_normalize(na::Quaternion::from_vector(
            na::Vector4::new(r[0], r[1], r[2], r[3])
        ));
        let scale = na::Scale3::new(s[0], s[1], s[2]);

        Transform {
            translation,
            rotation,
            scale,
        }
    }
}