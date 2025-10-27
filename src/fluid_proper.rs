use glam::Vec2;

#[derive(Debug, Clone)]
pub struct FluidSolver {
    pub width: usize,
    pub height: usize,
    pub density: Vec<f32>,
    pub density_prev: Vec<f32>,
    pub velocity_x: Vec<f32>,
    pub velocity_y: Vec<f32>,
    pub velocity_x_prev: Vec<f32>,
    pub velocity_y_prev: Vec<f32>,
    pub pressure: Vec<f32>,
    pub divergence: Vec<f32>,
    pub diffusion: f32,
    pub viscosity: f32,
    pub dt: f32,
    pub iterations: usize,
}

impl FluidSolver {
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
            pressure: vec![0.0; size],
            divergence: vec![0.0; size],
            diffusion: 0.000001,   // Much lower diffusion to preserve mass
            viscosity: 0.00001,    // Lower viscosity for more fluid movement
            dt: 0.05,             // Smaller timestep for stability
            iterations: 10,        // Fewer iterations for performance
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
        // Save previous state BEFORE any modifications
        self.velocity_x_prev.copy_from_slice(&self.velocity_x);
        self.velocity_y_prev.copy_from_slice(&self.velocity_y);
        self.density_prev.copy_from_slice(&self.density);
        
        // Step 1: Add external forces (gravity/buoyancy)
        self.add_buoyancy_forces();
        
        // Step 2: Diffuse velocity
        self.diffuse_velocity();
        
        // Step 3: Project velocity to make it divergence-free
        self.project_velocity();
        
        // Step 4: Advect velocity using the PREVIOUS velocity field
        self.advect_velocity();
        
        // Step 5: Project velocity again after advection
        self.project_velocity();
        
        // Step 6: Advect density using the FINAL velocity field
        self.advect_density();
        
        // Apply boundary conditions
        self.apply_boundary_conditions();
    }
    
    fn add_buoyancy_forces(&mut self) {
        // Simple buoyancy: dense fluid sinks, light fluid rises
        for y in 1..self.height-1 {
            for x in 1..self.width-1 {
                let idx = y * self.width + x;
                // Add upward force proportional to density (buoyancy)
                self.velocity_y[idx] -= self.density[idx] * 0.01;
            }
        }
    }

    fn diffuse_velocity(&mut self) {
        let a = self.dt * self.viscosity;
        
        for _ in 0..self.iterations {
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
            self.set_velocity_boundary();
        }
    }

    // Density diffusion is disabled to preserve mass
    // fn diffuse_density(&mut self) {
    //     // Skip density diffusion to preserve mass
    //     // Density diffusion is causing mass loss
    //     // self.density.copy_from_slice(&self.density_prev);
    // }

    fn advect_velocity(&mut self) {
        for y in 1..self.height-1 {
            for x in 1..self.width-1 {
                let idx = y * self.width + x;
                
                // Backtrace position using CURRENT velocity field (after diffusion/projection)
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
        self.set_velocity_boundary();
    }

    fn advect_density(&mut self) {
        for y in 1..self.height-1 {
            for x in 1..self.width-1 {
                let idx = y * self.width + x;
                
                // Backtrace position using CURRENT velocity field (after all processing)
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
        self.set_density_boundary();
    }

    fn project_velocity(&mut self) {
        // Calculate divergence
        let h = 1.0 / self.width as f32;
        for y in 1..self.height-1 {
            for x in 1..self.width-1 {
                let idx = y * self.width + x;
                self.divergence[idx] = -0.5 * h * (
                    self.velocity_x[idx+1] - self.velocity_x[idx-1] +
                    self.velocity_y[idx+self.width] - self.velocity_y[idx-self.width]
                );
                self.pressure[idx] = 0.0;
            }
        }
        
        self.set_pressure_boundary();
        
        // Solve for pressure using Gauss-Seidel
        for _ in 0..self.iterations {
            for y in 1..self.height-1 {
                for x in 1..self.width-1 {
                    let idx = y * self.width + x;
                    self.pressure[idx] = (
                        self.divergence[idx] +
                        self.pressure[idx-1] + self.pressure[idx+1] +
                        self.pressure[idx-self.width] + self.pressure[idx+self.width]
                    ) / 4.0;
                }
            }
            self.set_pressure_boundary();
        }
        
        // Subtract pressure gradient to make velocity divergence-free
        // Use a temporary velocity field to avoid feedback issues
        let mut temp_vel_x = self.velocity_x.clone();
        let mut temp_vel_y = self.velocity_y.clone();
        
        for y in 1..self.height-1 {
            for x in 1..self.width-1 {
                let idx = y * self.width + x;
                temp_vel_x[idx] -= 0.5 * (self.pressure[idx+1] - self.pressure[idx-1]) / h;
                temp_vel_y[idx] -= 0.5 * (self.pressure[idx+self.width] - self.pressure[idx-self.width]) / h;
            }
        }
        
        self.velocity_x = temp_vel_x;
        self.velocity_y = temp_vel_y;
        
        self.set_velocity_boundary();
    }

    fn set_velocity_boundary(&mut self) {
        // Set boundary conditions for velocity (free-slip boundaries - much gentler)
        for x in 0..self.width {
            // Top boundary: reflect vertical component, allow horizontal
            self.velocity_y[x] = -self.velocity_y[x + self.width];
            // Bottom boundary: reflect vertical component, allow horizontal  
            self.velocity_y[(self.height - 1) * self.width + x] = -self.velocity_y[(self.height - 2) * self.width + x];
        }
        
        for y in 0..self.height {
            // Left boundary: reflect horizontal component, allow vertical
            self.velocity_x[y * self.width] = -self.velocity_x[y * self.width + 1];
            // Right boundary: reflect horizontal component, allow vertical
            self.velocity_x[y * self.width + self.width - 1] = -self.velocity_x[y * self.width + self.width - 2];
        }
    }

    fn set_density_boundary(&mut self) {
        // Set boundary conditions for density (no-flux)
        for x in 0..self.width {
            self.density[x] = self.density[self.width + x]; // top
            self.density[(self.height - 1) * self.width + x] = self.density[(self.height - 2) * self.width + x]; // bottom
        }
        
        for y in 0..self.height {
            self.density[y * self.width] = self.density[y * self.width + 1]; // left
            self.density[y * self.width + self.width - 1] = self.density[y * self.width + self.width - 2]; // right
        }
    }

    fn set_pressure_boundary(&mut self) {
        // Set boundary conditions for pressure
        for x in 0..self.width {
            self.pressure[x] = self.pressure[self.width + x]; // top
            self.pressure[(self.height - 1) * self.width + x] = self.pressure[(self.height - 2) * self.width + x]; // bottom
        }
        
        for y in 0..self.height {
            self.pressure[y * self.width] = self.pressure[y * self.width + 1]; // left
            self.pressure[y * self.width + self.width - 1] = self.pressure[y * self.width + self.width - 2]; // right
        }
    }

    fn apply_boundary_conditions(&mut self) {
        // Additional boundary damping for better fluid containment
        for x in 0..self.width {
            // Damp near boundaries
            let damp_factor = 0.95;
            self.density[x] *= damp_factor; // top
            self.density[(self.height - 1) * self.width + x] *= damp_factor; // bottom
        }
        
        for y in 0..self.height {
            self.density[y * self.width] *= 0.95; // left
            self.density[y * self.width + self.width - 1] *= 0.95; // right
        }
    }
}