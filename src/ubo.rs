use nalgebra::{Matrix4, Vector3};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub(crate) struct Ubo {
    view: Matrix4<f32>,
    projection: Matrix4<f32>,
}

impl Ubo {
    pub(crate) fn new(view: Matrix4<f32>, projection: Matrix4<f32>) -> Self {
        Self { view, projection }
    }
}

impl Default for Ubo {
    fn default() -> Self {
        let view_matrix = Matrix4::look_at_rh(
            &Vector3::new(2.0, 2.0, 2.0).into(),
            &Vector3::zeros().into(),
            &Vector3::new(0.0, 0.0, 1.0),
        );

        let projection_matrix =
            Matrix4::new_perspective(16.0 / 9.0, 45.0f32.to_radians(), 0.1, 10.0);

        Self {
            view: view_matrix,
            projection: projection_matrix,
        }
    }
}
