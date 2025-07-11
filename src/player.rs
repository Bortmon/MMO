use glam::Vec3;

pub struct Player {
    pub position: Vec3,
    pub target_position: Option<Vec3>, 
}

impl Player {
    pub fn new(position: Vec3) -> Self {
        Self { position, target_position: None }
    }
}