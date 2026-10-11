use nalgebra::Vector3;

use winit::{
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use crate::camera::Camera;

pub struct CameraController {
    speed: f32,
    sensitivity: f32,

    forward: bool,
    backward: bool,
    left: bool,
    right: bool,

    up: bool,
    down: bool,

    mouse_pressed: bool,
    last_mouse: Option<(f64, f64)>,

    yaw_delta: f32,
    pitch_delta: f32,

    zoom: f32,
}

impl CameraController {
    pub fn new() -> Self {
        Self {
            // 駿河湾のワールド座標に合わせた移動速度。
            speed: 5000.0,

            sensitivity: 0.003,

            forward: false,
            backward: false,
            left: false,
            right: false,

            up: false,
            down: false,

            mouse_pressed: false,
            last_mouse: None,

            yaw_delta: 0.0,
            pitch_delta: 0.0,

            zoom: 0.0,
        }
    }

    /// ウィンドウイベントを処理する。
    pub fn process(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                let pressed = event.state == ElementState::Pressed;

                match event.physical_key {
                    PhysicalKey::Code(KeyCode::KeyW) => {
                        self.forward = pressed;
                    }

                    PhysicalKey::Code(KeyCode::KeyS) => {
                        self.backward = pressed;
                    }

                    PhysicalKey::Code(KeyCode::KeyA) => {
                        self.left = pressed;
                    }

                    PhysicalKey::Code(KeyCode::KeyD) => {
                        self.right = pressed;
                    }

                    PhysicalKey::Code(KeyCode::Space) => {
                        self.up = pressed;
                    }

                    PhysicalKey::Code(KeyCode::ShiftLeft) => {
                        self.down = pressed;
                    }

                    _ => {}
                }
            }

            WindowEvent::MouseInput { state, button, .. } => {
                if *button == MouseButton::Right {
                    self.mouse_pressed = *state == ElementState::Pressed;

                    self.last_mouse = None;
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                if self.mouse_pressed {
                    if let Some((last_x, last_y)) = self.last_mouse {
                        self.yaw_delta += (position.x - last_x) as f32 * self.sensitivity;

                        self.pitch_delta += (position.y - last_y) as f32 * self.sensitivity;
                    }

                    self.last_mouse = Some((position.x, position.y));
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let y = match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y,

                    MouseScrollDelta::PixelDelta(position) => position.y as f32 / 50.0,
                };

                self.zoom += y;
            }

            _ => {}
        }
    }

    /// 既存コードとの互換性を保つための更新関数。
    pub fn update(&mut self, camera: &mut Camera) {
        self.update_with_dt(camera, 1.0 / 60.0);
    }

    /// フレーム経過時間を考慮してカメラを更新する。
    pub fn update_with_dt(&mut self, camera: &mut Camera, dt_seconds: f32) {
        let dt = dt_seconds.clamp(0.0, 0.05);

        // ----------------------------------------------
        // 視点回転
        // ----------------------------------------------

        camera.yaw -= self.yaw_delta;
        camera.pitch -= self.pitch_delta;

        camera.pitch = camera
            .pitch
            .clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());

        self.yaw_delta = 0.0;
        self.pitch_delta = 0.0;

        // ----------------------------------------------
        // ホイールによる前後移動
        // ----------------------------------------------

        if self.zoom != 0.0 {
            let forward = Vector3::new(camera.yaw.cos(), 0.0, camera.yaw.sin()).normalize();

            camera.position += forward * self.zoom * 10000.0;

            self.zoom = 0.0;
        }

        // ----------------------------------------------
        // 水平移動
        // ----------------------------------------------

        let forward = Vector3::new(camera.yaw.cos(), 0.0, camera.yaw.sin()).normalize();

        let right = Vector3::new(-forward.z, 0.0, forward.x).normalize();

        let mut direction = Vector3::zeros();

        if self.forward {
            direction += forward;
        }

        if self.backward {
            direction -= forward;
        }

        if self.left {
            direction -= right;
        }

        if self.right {
            direction += right;
        }

        // ----------------------------------------------
        // 垂直移動
        // ----------------------------------------------

        if self.up {
            direction.y += 1.0;
        }

        if self.down {
            direction.y -= 1.0;
        }

        // ----------------------------------------------
        // 移動量の適用
        // ----------------------------------------------

        if direction.norm_squared() > 0.0 {
            camera.position += direction.normalize() * self.speed * dt;
        }
    }
}

impl Default for CameraController {
    fn default() -> Self {
        Self::new()
    }
}
