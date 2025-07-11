pub const WORLD_SIZE: usize = 64;

pub struct World {
    pub heightmap: Vec<Vec<f32>>,
}

impl World {
    pub fn new() -> Self {

        let heightmap = vec![vec![0.0; WORLD_SIZE]; WORLD_SIZE];

        Self { heightmap }
    }

    pub fn get_height(&self, x: f32, z: f32) -> f32 {
        let x_clamped = x.max(0.0).min((WORLD_SIZE - 1) as f32);
        let z_clamped = z.max(0.0).min((WORLD_SIZE - 1) as f32);

        self.heightmap[x_clamped.round() as usize][z_clamped.round() as usize]
    }
}