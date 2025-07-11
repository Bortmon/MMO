use crate::camera::OsrsCamera;
use winit::event::{ElementState, KeyEvent, MouseScrollDelta};
use winit::keyboard::{Key, NamedKey};

pub struct CameraController {
    is_left_pressed: bool,
    is_right_pressed: bool,
    rotation_speed: f32,
    zoom_speed: f32,
}

impl CameraController {
    pub fn new(rotation_speed: f32, zoom_speed: f32) -> Self {
        Self {
            is_left_pressed: false,
            is_right_pressed: false,
            rotation_speed,
            zoom_speed,
        }
    }

    pub fn process_keyboard(&mut self, event: &KeyEvent) -> bool {
        match &event.logical_key {
            Key::Named(NamedKey::ArrowLeft) => {
                self.is_left_pressed = event.state == ElementState::Pressed;
                true
            }
            Key::Named(NamedKey::ArrowRight) => {
                self.is_right_pressed = event.state == ElementState::Pressed;
                true
            }
            _ => false,
        }
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        let scroll_amount = match delta {
            MouseScrollDelta::LineDelta(_, y) => *y,
            MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
        };
        self.zoom_speed = -scroll_amount * 0.5;
    }

    pub fn update_camera(&mut self, camera: &mut OsrsCamera) {
        if self.is_left_pressed {
            camera.yaw -= self.rotation_speed;
        }
        if self.is_right_pressed {
            camera.yaw += self.rotation_speed;
        }

        camera.distance += self.zoom_speed;
        if camera.distance < 2.0 {
            camera.distance = 2.0;
        }
        if camera.distance > 50.0 {
            camera.distance = 50.0;
        }

        
        self.zoom_speed = 0.0;
    }
}