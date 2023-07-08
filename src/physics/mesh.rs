use nalgebra as na;
use rapier3d::prelude::ColliderBuilder;

use crate::render::pbr::model::Mesh;

impl From<Mesh> for ColliderBuilder {
    fn from(mesh: Mesh) -> Self {
        let vertices = mesh.vertexdata
            .iter()
            .map(|v| {
                na::Point3::new(v.position[0], v.position[1], v.position[2])
            })
            .collect::<Vec<na::Point3<f32>>>();

        let indices = (0..mesh.indexdata.len() / 3)
            .map(|i| {
                [
                    mesh.indexdata.get(i * 3).unwrap_or(&0).clone(),
                    mesh.indexdata.get(i * 3 + 1).unwrap_or(&0).clone(),
                    mesh.indexdata.get(i * 3 + 2).unwrap_or(&0).clone(),
                ]
            })
            .collect::<Vec<[u32; 3]>>();

        log::debug!("Vertices: {vertices:#?}\nIndices: {indices:#?}");

        ColliderBuilder::trimesh(vertices, indices)
    }
}