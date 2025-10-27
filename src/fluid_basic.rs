use glam::Vec2;

#[derive(Debug, Clone)]
pub struct BasicFluid {
    pub width: usize,
    pub height: usize,
    pub density: Vec<f32>,
    pub velocity_x: Vec<f32>,
    pub velocity_y: Vec<f32>,
    pub dt: f32,
}

impl BasicFluid {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        Self {
            width,
            height,
            density: vec![0.0; size],
            velocity_x: vec![0.0; size],
            velocity_y: vec![0.0; size],
            dt: 0.5, // Larger timestep for visible movement
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
        // Simple forward advection - move density according to velocity
        let mut new_density = vec![0.0; self.density.len()];
        
        for y in 1..self.height-1 {
            for x in 1..self.width-1 {
                let idx = y * self.width + x;
                
                if self.density[idx] > 0.0 {
                    // Calculate new position based on velocity
                    let new_x = (x as f32 + self.velocity_x[idx] * self.dt).max(1.0).min((self.width - 2) as f32);
                    let new_y = (y as f32 + self.velocity_y[idx] * self.dt).max(1.0).min((self.height - 2) as f32);
                    
                    // Round to nearest grid cell
                    let target_x = new_x.round() as usize;
                    let target_y = new_y.round() as usize;
                    
                    let target_idx = target_y * self.width + target_x;
                    
                    // Move density to new position
                    new_density[target_idx] += self.density[idx];
                }
            }
        }
        
        self.density = new_density;
        
        // Simple velocity damping
        for y in 1..self.height-1 {
            for x in 1..self.width-1 {
                let idx = y * self.width + x;
                self.velocity_x[idx] *= 0.95;
                self.velocity_y[idx] *= 0.95;
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