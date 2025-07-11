use glam::{Mat4, Quat, Vec3};

pub struct OsrsCamera {
    pub focus_point: Vec3, 
    pub yaw: f32,         
    pub pitch: f32,        
    pub distance: f32,     
}

impl OsrsCamera {
    pub fn new(focus_point: Vec3) -> Self {
        Self {
            focus_point,
            yaw: 0.0,
            pitch: 50.0, 
            distance: 10.0,
        }
    }

    pub fn eye_position(&self) -> Vec3 {
        let yaw_rad = self.yaw.to_radians();
        let pitch_rad = self.pitch.to_radians();
        let rotation = Quat::from_rotation_y(yaw_rad) * Quat::from_rotation_x(-pitch_rad);
        let position_offset = rotation * Vec3::new(0.0, 0.0, self.distance);
        self.focus_point + position_offset
    }

    pub fn build_view_matrix(&self) -> Mat4 {
        let yaw_rad = self.yaw.to_radians();
        let pitch_rad = self.pitch.to_radians();

        let rotation = Quat::from_rotation_y(yaw_rad) * Quat::from_rotation_x(-pitch_rad);
        
        let position_offset = rotation * Vec3::new(0.0, 0.0, self.distance);
        let eye_position = self.focus_point + position_offset;

        Mat4::look_at_rh(eye_position, self.focus_point, Vec3::Y)
    }
}

pub struct Projection {
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new(width: u32, height: u32, fovy_degrees: f32, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy_degrees.to_radians(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn build_projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar)
    }
}