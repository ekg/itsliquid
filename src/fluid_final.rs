use glam::Vec2;

#[derive(Debug, Clone)]
pub struct FluidFinal {
    pub width: usize,
    pub height: usize,
    pub density: Vec<f32>,
    pub velocity_x: Vec<f32>,
    pub velocity_y: Vec<f32>,
    pub dt: f32,
}

impl FluidFinal {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        Self {
            width,
            height,
            density: vec![0.0; size],
            velocity_x: vec![0.0; size],
            velocity_y: vec![0.0; size],
            dt: 1.0, // Larger timestep for visible movement
        }
    }

    pub fn add_density(&mut self, x: usize, y: usize, amount: f32) {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            self.density[idx] += amount;
        }
    }

    pub fn add_velocity(&mut self, x: usize, y: usize, velocity: Vec2) {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            self.velocity_x[idx] += velocity.x;
            self.velocity_y[idx] += velocity.y;
        }
    }

    pub fn step(&mut self) {
        // Simple forward advection - move each density cell according to its velocity
        let mut new_density = vec![0.0; self.density.len()];

        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                let idx = y * self.width + x;

                if self.density[idx] > 0.0 {
                    // Calculate movement based on velocity
                    let move_x = (self.velocity_x[idx] * self.dt).round() as i32;
                    let move_y = (self.velocity_y[idx] * self.dt).round() as i32;

                    let new_x = (x as i32 + move_x).max(1).min((self.width - 2) as i32) as usize;
                    let new_y = (y as i32 + move_y).max(1).min((self.height - 2) as i32) as usize;

                    let new_idx = new_y * self.width + new_x;

                    // Move the density
                    new_density[new_idx] += self.density[idx];
                }
            }
        }

        self.density = new_density;

        // Simple velocity damping
        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                let idx = y * self.width + x;
                self.velocity_x[idx] *= 0.9;
                self.velocity_y[idx] *= 0.9;
            }
        }

        // Apply boundary conditions
        self.apply_boundary_conditions();
    }

    fn apply_boundary_conditions(&mut self) {
        // Simple boundary conditions
        for x in 0..self.width {
            self.velocity_y[x] = 0.0;
            self.velocity_y[(self.height - 1) * self.width + x] = 0.0;
        }

        for y in 0..self.height {
            self.velocity_x[y * self.width] = 0.0;
            self.velocity_x[y * self.width + self.width - 1] = 0.0;
        }
    }
}
