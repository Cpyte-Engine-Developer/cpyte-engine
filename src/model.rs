use hashbrown::HashMap;
use itertools::Itertools;
use nalgebra::{Vector2, Vector3};
use tobj::{load_obj, GPU_LOAD_OPTIONS};

use crate::{texture::Texture, vertex::Vertex};

#[derive(Default, Clone, Debug)]
pub(crate) struct Model {
    pub(crate) vertices: Vec<Vertex>,
    pub(crate) indices: Vec<u32>,
    pub(crate) texture: Texture,
}

impl Model {
    pub(crate) fn new(model_path: &str, texture_path: Option<&str>) -> Self {
        let texture = Texture::new(texture_path.unwrap_or(""));

        let (models, _) = load_obj(model_path, &GPU_LOAD_OPTIONS).unwrap();

        let vertices = models
            .iter()
            .flat_map(|model| {
                let mesh = &model.mesh;
                let positions = &mesh.positions;
                let texture_uvs = &mesh.texcoords;

                (0..(positions.len()) / 3)
                    .map(|i| {
                        Vertex::new(
                            Vector3::new(
                                positions[3 * i],
                                positions[3 * i + 1],
                                positions[3 * i + 2],
                            ),
                            Vector3::new(1.0, 1.0, 1.0),
                            Vector2::new(texture_uvs[2 * i], 1.0 - texture_uvs[2 * i + 1]),
                        )
                    })
                    .collect_vec()
            })
            .collect_vec();

        let indices = models
            .iter()
            .flat_map(|model| model.mesh.indices.clone())
            .collect_vec();

        Self {
            vertices,
            indices,
            texture,
        }
    }
}
