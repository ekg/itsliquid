use glam::Vec2;

#[derive(Debug, Clone)]
pub struct WorkingFluid {
    pub width: usize,
    pub height: usize,
    pub density: Vec<f32>,
    pub density_prev: Vec<f32>,
    pub velocity_x: Vec<f32>,
    pub velocity_y: Vec<f32>,
    pub velocity_x_prev: Vec<f32>,
    pub velocity_y_prev: Vec<f32>,
    pub dt: f32,
    pub viscosity: f32,
    pub diffusion: f32,
}

impl WorkingFluid {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        Self {
            width,
            height,
            density: vec![0.0; size],
            density_prev: vec![0.0; size],
            velocity_x: vec![0.0; size],
            velocity_y: vec![0.0; size],
            velocity_x_prev: vec![0.0; size],
            velocity_y_prev: vec![0.0; size],
            dt: 0.1,
            viscosity: 0.001,
            diffusion: 0.001,
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
        // Save current state
        self.velocity_x_prev.copy_from_slice(&self.velocity_x);
        self.velocity_y_prev.copy_from_slice(&self.velocity_y);
        self.density_prev.copy_from_slice(&self.density);

        // Step 1: Diffuse velocity
        self.diffuse_velocity();
        
        // Step 2: Project velocity (make divergence-free)
        self.project_velocity();
        
        // Step 3: Advect velocity
        self.advect_velocity();
        
        // Step 4: Project velocity again
        self.project_velocity();
        
        // Step 5: Diffuse density
        self.diffuse_density();
        
        // Step 6: Advect density
        self.advect_density();
        
        // Apply boundary conditions
        self.set_boundaries();
    }

    fn diffuse_velocity(&mut self) {
        let a = self.dt * self.viscosity * (self.width * self.height) as f32;
        
        for _ in 0..4 {
            for y in 1..self.height-1 {
                for x in 1..self.width-1 {
                    let idx = y * self.width + x;
                    self.velocity_x[idx] = (self.velocity_x_prev[idx] + a * (
                        self.velocity_x[idx-1] + self.velocity_x[idx+1] +
                        self.velocity_x[idx-self.width] + self.velocity_x[idx+self.width]
                    )) / (1.0 + 4.0 * a);
                    
                    self.velocity_y[idx] = (self.velocity_y_prev[idx] + a * (
                        self.velocity_y[idx-1] + self.velocity_y[idx+1] +
                        self.velocity_y[idx-self.width] + self.velocity_y[idx+self.width]
                    )) / (1.0 + 4.0 * a);
                }
            }
            self.set_velocity_boundaries();
        }
    }

    fn diffuse_density(&mut self) {
        let a = self.dt * self.diffusion * (self.width * self.height) as f32;
        
        for _ in 0..4 {
            for y in 1..self.height-1 {
                for x in 1..self.width-1 {
                    let idx = y * self.width + x;
                    self.density[idx] = (self.density_prev[idx] + a * (
                        self.density[idx-1] + self.density[idx+1] +
                        self.density[idx-self.width] + self.density[idx+self.width]
                    )) / (1.0 + 4.0 * a);
                }
            }
            self.set_density_boundaries();
        }
    }

    fn advect_velocity(&mut self) {
        for y in 1..self.height-1 {
            for x in 1..self.width-1 {
                let idx = y * self.width + x;
                
                // Backtrace using previous velocity field
                let src_x = x as f32 - self.dt * self.velocity_x_prev[idx];
                let src_y = y as f32 - self.dt * self.velocity_y_prev[idx];
                
                // Clamp to valid range
                let src_x = src_x.max(0.5).min((self.width - 1) as f32 - 0.5);
                let src_y = src_y.max(0.5).min((self.height - 1) as f32 - 0.5);
                
                // Bilinear interpolation
                let x0 = src_x.floor() as usize;
                let x1 = x0 + 1;
                let y0 = src_y.floor() as usize;
                let y1 = y0 + 1;
                
                let sx = src_x - x0 as f32;
                let sy = src_y - y0 as f32;
                
                let idx00 = y0 * self.width + x0;
                let idx01 = y0 * self.width + x1;
                let idx10 = y1 * self.width + x0;
                let idx11 = y1 * self.width + x1;
                
                // Advect velocity
                self.velocity_x[idx] = (1.0 - sx) * (1.0 - sy) * self.velocity_x_prev[idx00] +
                                     sx * (1.0 - sy) * self.velocity_x_prev[idx01] +
                                     (1.0 - sx) * sy * self.velocity_x_prev[idx10] +
                                     sx * sy * self.velocity_x_prev[idx11];
                
                self.velocity_y[idx] = (1.0 - sx) * (1.0 - sy) * self.velocity_y_prev[idx00] +
                                     sx * (1.0 - sy) * self.velocity_y_prev[idx01] +
                                     (1.0 - sx) * sy * self.velocity_y_prev[idx10] +
                                     sx * sy * self.velocity_y_prev[idx11];
            }
        }
        self.set_velocity_boundaries();
    }

    fn advect_density(&mut self) {
        for y in 1..self.height-1 {
            for x in 1..self.width-1 {
                let idx = y * self.width + x;
                
                // Backtrace using current velocity field
                let src_x = x as f32 - self.dt * self.velocity_x[idx];
                let src_y = y as f32 - self.dt * self.velocity_y[idx];
                
                // Clamp to valid range
                let src_x = src_x.max(0.5).min((self.width - 1) as f32 - 0.5);
                let src_y = src_y.max(0.5).min((self.height - 1) as f32 - 0.5);
                
                // Bilinear interpolation
                let x0 = src_x.floor() as usize;
                let x1 = x0 + 1;
                let y0 = src_y.floor() as usize;
                let y1 = y0 + 1;
                
                let sx = src_x - x0 as f32;
                let sy = src_y - y0 as f32;
                
                let idx00 = y0 * self.width + x0;
                let idx01 = y0 * self.width + x1;
                let idx10 = y1 * self.width + x0;
                let idx11 = y1 * self.width + x1;
                
                // Advect density
                self.density[idx] = (1.0 - sx) * (1.0 - sy) * self.density_prev[idx00] +
                                  sx * (1.0 - sy) * self.density_prev[idx01] +
                                  (1.0 - sx) * sy * self.density_prev[idx10] +
                                  sx * sy * self.density_prev[idx11];
            }
        }
        self.set_density_boundaries();
    }

    fn project_velocity(&mut self) {
        let h = 1.0 / self.width as f32;
        let mut divergence = vec![0.0; self.width * self.height];
        let mut pressure = vec![0.0; self.width * self.height];
        
        // Calculate divergence
        for y in 1..self.height-1 {
            for x in 1..self.width-1 {
                let idx = y * self.width + x;
                divergence[idx] = -0.5 * h * (
                    self.velocity_x[idx+1] - self.velocity_x[idx-1] +
                    self.velocity_y[idx+self.width] - self.velocity_y[idx-self.width]
                );
            }
        }
        
        // Solve for pressure
        for _ in 0..20 {
            for y in 1..self.height-1 {
                for x in 1..self.width-1 {
                    let idx = y * self.width + x;
                    pressure[idx] = (divergence[idx] + 
                        pressure[idx-1] + pressure[idx+1] +
                        pressure[idx-self.width] + pressure[idx+self.width]) / 4.0;
                }
            }
            self.set_pressure_boundaries(&mut pressure);
        }
        
        // Subtract pressure gradient
        for y in 1..self.height-1 {
            for x in 1..self.width-1 {
                let idx = y * self.width + x;
                self.velocity_x[idx] -= 0.5 * (pressure[idx+1] - pressure[idx-1]) / h;
                self.velocity_y[idx] -= 0.5 * (pressure[idx+self.width] - pressure[idx-self.width]) / h;
            }
        }
        
        self.set_velocity_boundaries();
    }

    fn set_boundaries(&mut self) {
        self.set_velocity_boundaries();
        self.set_density_boundaries();
    }

    fn set_velocity_boundaries(&mut self) {
        for x in 0..self.width {
            self.velocity_x[x] = 0.0;
            self.velocity_y[x] = 0.0;
            self.velocity_x[(self.height - 1) * self.width + x] = 0.0;
            self.velocity_y[(self.height - 1) * self.width + x] = 0.0;
        }
        
        for y in 0..self.height {
            self.velocity_x[y * self.width] = 0.0;
            self.velocity_y[y * self.width] = 0.0;
            self.velocity_x[y * self.width + self.width - 1] = 0.0;
            self.velocity_y[y * self.width + self.width - 1] = 0.0;
        }
    }

    fn set_density_boundaries(&mut self) {
        for x in 0..self.width {
            self.density[x] = self.density[self.width + x];
            self.density[(self.height - 1) * self.width + x] = self.density[(self.height - 2) * self.width + x];
        }
        
        for y in 0..self.height {
            self.density[y * self.width] = self.density[y * self.width + 1];
            self.density[y * self.width + self.width - 1] = self.density[y * self.width + self.width - 2];
        }
    }

    fn set_pressure_boundaries(&mut self, pressure: &mut Vec<f32>) {
        for x in 0..self.width {
            pressure[x] = pressure[self.width + x];
            pressure[(self.height - 1) * self.width + x] = pressure[(self.height - 2) * self.width + x];
        }
        
        for y in 0..self.height {
            pressure[y * self.width] = pressure[y * self.width + 1];
            pressure[y * self.width + self.width - 1] = pressure[y * self.width + self.width - 2];
        }
    }
}