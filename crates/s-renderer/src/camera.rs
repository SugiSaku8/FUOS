use nalgebra::{Isometry3, Matrix4, Perspective3, Point3, Vector3};

/// Camera used by the renderer.
///
/// This type is independent of wgpu and is responsible only for
/// generating view/projection matrices.
#[derive(Debug, Clone)]
pub struct Camera {
    eye: Point3<f32>,
    target: Point3<f32>,
    up: Vector3<f32>,

    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    /// Creates a new perspective camera.
    #[must_use]
    pub fn new(
        eye: Point3<f32>,
        target: Point3<f32>,
        up: Vector3<f32>,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Self {
            eye,
            target,
            up,
            aspect,
            fovy,
            znear,
            zfar,
        }
    }

    /// Default camera suitable for terrain rendering.
    #[must_use]
    pub fn default_perspective(width: u32, height: u32) -> Self {
        let aspect = if height == 0 {
            1.0
        } else {
            width as f32 / height as f32
        };

        Self {
            eye: Point3::new(0.0, -5000.0, 2500.0),
            target: Point3::origin(),
            up: Vector3::z_axis().into_inner(),

            aspect,
            fovy: 45.0_f32.to_radians(),
            znear: 0.1,
            zfar: 500_000.0,
        }
    }

    #[must_use]
    pub fn eye(&self) -> Point3<f32> {
        self.eye
    }

    pub fn set_eye(&mut self, eye: Point3<f32>) {
        self.eye = eye;
    }

    #[must_use]
    pub fn target(&self) -> Point3<f32> {
        self.target
    }

    pub fn set_target(&mut self, target: Point3<f32>) {
        self.target = target;
    }

    #[must_use]
    pub fn aspect(&self) -> f32 {
        self.aspect
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect.max(0.0001);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = if height == 0 {
            1.0
        } else {
            width as f32 / height as f32
        };
    }

    #[must_use]
    pub fn view_matrix(&self) -> Matrix4<f32> {
        Isometry3::look_at_rh(&self.eye, &self.target, &self.up).to_homogeneous()
    }

    #[must_use]
    pub fn projection_matrix(&self) -> Matrix4<f32> {
        Perspective3::new(self.aspect, self.fovy, self.znear, self.zfar).to_homogeneous()
    }

    #[must_use]
    pub fn view_projection_matrix(&self) -> Matrix4<f32> {
        self.projection_matrix() * self.view_matrix()
    }
}
