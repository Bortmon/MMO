use crate::camera::OsrsCamera;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::{Key, NamedKey};

#[derive(Default)]
pub struct CameraController {
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_middle_mouse_pressed: bool,
    
    rotation_speed: f32,
    mouse_sensitivity: f32,
    
    mouse_delta_x: f32,
    mouse_delta_y: f32,
    zoom_delta: f32,
}

impl CameraController {
    pub fn new(rotation_speed: f32, mouse_sensitivity: f32) -> Self {
        Self {
            rotation_speed,
            mouse_sensitivity,
            ..Default::default()
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { event: key_event, .. } => {
                match &key_event.logical_key {
                    Key::Named(NamedKey::ArrowLeft) => {
                        self.is_left_pressed = key_event.state == ElementState::Pressed;
                        true
                    }
                    Key::Named(NamedKey::ArrowRight) => {
                        self.is_right_pressed = key_event.state == ElementState::Pressed;
                        true
                    }
                    _ => false,
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.zoom_delta += match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y * -1.0,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32 * -0.1,
                };
                true
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if *button == MouseButton::Middle {
                    self.is_middle_mouse_pressed = *state == ElementState::Pressed;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub fn process_mouse_motion(&mut self, delta_x: f64, delta_y: f64) {
        self.mouse_delta_x += delta_x as f32;
        self.mouse_delta_y += delta_y as f32;
    }

    pub fn update_camera(&mut self, camera: &mut OsrsCamera) {
        let zoom_sensitivity = 0.5;

        if self.is_left_pressed {
            camera.yaw -= self.rotation_speed;
        }
        if self.is_right_pressed {
            camera.yaw += self.rotation_speed;
        }

        if self.is_middle_mouse_pressed {
            camera.yaw += self.mouse_delta_x * self.mouse_sensitivity;
            camera.pitch -= self.mouse_delta_y * self.mouse_sensitivity;
            camera.pitch = camera.pitch.clamp(5.0, 89.0);
        }

        camera.distance += self.zoom_delta * zoom_sensitivity;
        camera.distance = camera.distance.clamp(2.0, 50.0);

        self.zoom_delta = 0.0;
        self.mouse_delta_x = 0.0;
        self.mouse_delta_y = 0.0;
    }
}