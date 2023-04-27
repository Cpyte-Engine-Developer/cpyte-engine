use hashbrown::HashMap;
use rand::prelude::*;
use pyo3::prelude::*;
use numpy::nalgebra::{Vector3, UnitQuaternion, Matrix4};

use crate::model::Model;

#[derive(Clone, Debug)]
#[pyclass]
pub(crate) struct Entity {
    pub(crate) id: usize,
    #[pyo3(get, set)]
    pub(crate) position: Vector3<f32>,
    #[pyo3(get, set)]
    pub(crate) rotation: UnitQuaternion<f32>,
    #[pyo3(get, set)]
    pub(crate) scale: Vector3<f32>,
    pub(crate) model: Model,
    #[pyo3(get, set)]
    pub(crate) parent: Option<Box<Self>>,
    #[pyo3(get, set)]
    pub(crate) children: HashMap<String, Self>,
}

#[pymethods]
impl Entity {
    #[new]
    pub(crate) fn new(
        position: Vector3<f32>,
        rotation: UnitQuaternion<f32>,
        scale: Vector3<f32>,
        model_path: &str,
        texture_path: Option<&str>,
    ) -> Self {
        Self {
            id: thread_rng().gen::<usize>(),
            position,
            rotation,
            scale,
            model: Model::new(model_path, texture_path),
            parent: None,
            children: HashMap::new()
        }
    }

    pub(crate) fn on_update(&mut self, exec_time: f32) {
        self.rotation *= UnitQuaternion::from_axis_angle(&Vector3::z_axis(), 1.0f32.to_radians());
    }

    pub(crate) fn transform_matrix(&self) -> Matrix4<f32> {
        Matrix4::new_nonuniform_scaling(&self.scale)
            * self.rotation.to_homogeneous()
            * Matrix4::new_translation(&self.position)
    }
}

unsafe impl Send for Entity {}