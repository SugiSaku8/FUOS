use nalgebra::{Matrix4, Perspective3, Point3, Vector3};

pub struct Camera {
    pub position: Point3<f32>,

    pub yaw: f32,

    pub pitch: f32,

    pub aspect: f32,

    pub fovy: f32,

    pub znear: f32,

    pub zfar: f32,
}

impl Camera {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            position: Point3::new(50000.0, 50000.0, 80000.0),

            yaw: (-30.0_f32).to_radians(),

            pitch: (-55.0_f32).to_radians(),

            aspect: width as f32 / height as f32,

            fovy: 45.0_f32.to_radians(),

            znear: 1.0,

            zfar: 200000.0,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn direction(&self) -> Vector3<f32> {
        Vector3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize()
    }

    pub fn right(&self) -> Vector3<f32> {
        self.direction().cross(&Vector3::y_axis()).normalize()
    }

    pub fn move_forward(&mut self, amount: f32) {
        self.position += self.direction() * amount;
    }

    pub fn move_right(&mut self, amount: f32) {
        self.position += self.right() * amount;
    }

    pub fn move_vertical(&mut self, amount: f32) {
        self.position.y += amount;
    }

    pub fn rotate(&mut self, dx: f32, dy: f32) {
        let sensitivity = 0.002;

        self.yaw += dx * sensitivity;

        self.pitch -= dy * sensitivity;

        self.pitch = self
            .pitch
            .clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());
    }

    pub fn build_matrix(&self) -> Matrix4<f32> {
        let direction = self.direction();

        let target = self.position + direction;

        let view = Matrix4::look_at_rh(&self.position, &target, &Vector3::y_axis());

        let projection =
            Perspective3::new(self.aspect, self.fovy, self.znear, self.zfar).to_homogeneous();

        projection * view
    }
}
