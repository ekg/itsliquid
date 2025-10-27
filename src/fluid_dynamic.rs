use glam::Vec2;

#[derive(Debug, Clone)]
pub struct FluidSimulation {
    pub width: usize,
    pub height: usize,
    pub density: Vec<f32>,
    pub velocity_x: Vec<f32>,
    pub velocity_y: Vec<f32>,
    pub diffusion_rate: f32,
    pub viscosity: f32,
    pub dt: f32,
}

impl FluidSimulation {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        Self {
            width,
            height,
            density: vec![0.0; size],
            velocity_x: vec![0.0; size],
            velocity_y: vec![0.0; size],
            diffusion_rate: 0.0001,  // Proper diffusion for mixing
            viscosity: 0.0001,       // Proper viscosity for flow
            dt: 0.1,
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
        // Step 1: Diffuse velocity (creates smooth flow)
        self.diffuse_velocity();
        
        // Step 2: Project to make incompressible (creates pressure effects)
        self.project();
        
        // Step 3: Advect velocity (fluid carries itself along)
        self.advect_velocity();
        
        // Step 4: Project again
        self.project();
        
        // Step 5: Diffuse density (mixing)
        self.diffuse_density();
        
        // Step 6: Advect density (fluid carries density along)
        self.advect_density();
        
        // Step 7: Apply gentle boundary conditions
        self.apply_boundary_conditions();
    }

    fn diffuse_velocity(&mut self) {
        let mut new_vel_x = self.velocity_x.clone();
        let mut new_vel_y = self.velocity_y.clone();
        
        let alpha = self.viscosity * self.dt;
        
        for y in 1..self.height-1 {
            for x in 1..self.width-1 {
                let idx = y * self.width + x;
                
                new_vel_x[idx] = (self.velocity_x[idx] + alpha * (
                    self.velocity_x[idx-1] + self.velocity_x[idx+1] +
                    self.velocity_x[idx-self.width] + self.velocity_x[idx+self.width]
                )) / (1.0 + 4.0 * alpha);
                
                new_vel_y[idx] = (self.velocity_y[idx] + alpha * (
                    self.velocity_y[idx-1] + self.velocity_y[idx+1] +
                    self.velocity_y[idx-self.width] + self.velocity_y[idx+self.width]
                )) / (1.0 + 4.0 * alpha);
            }
        }
        
        self.velocity_x = new_vel_x;
        self.velocity_y = new_vel_y;
        self.set_velocity_boundaries();
    }

    fn diffuse_density(&mut self) {
        let mut new_density = self.density.clone();
        let alpha = self.diffusion_rate * self.dt;
        
        for y in 1..self.height-1 {
            for x in 1..self.width-1 {
                let idx = y * self.width + x;
                
                new_density[idx] = (self.density[idx] + alpha * (
                    self.density[idx-1] + self.density[idx+1] +
                    self.density[idx-self.width] + self.density[idx+self.width]
                )) / (1.0 + 4.0 * alpha);
            }
        }
        
        self.density = new_density;
        self.set_density_boundaries();
    }

    fn project(&mut self) {
        // Simple pressure projection to make flow divergence-free
        for y in 1..self.height-1 {
            for x in 1..self.width-1 {
                let idx = y * self.width + x;
                
                // Calculate divergence
                let div = (self.velocity_x[idx+1] - self.velocity_x[idx-1] +
                          self.velocity_y[idx+self.width] - self.velocity_y[idx-self.width]) * 0.5;
                
                // Apply pressure correction
                let pressure = div * 0.25;
                
                self.velocity_x[idx-1] += pressure;
                self.velocity_x[idx+1] -= pressure;
                self.velocity_y[idx-self.width] += pressure;
                self.velocity_y[idx+self.width] -= pressure;
            }
        }
        
        self.set_velocity_boundaries();
    }

    fn advect_velocity(&mut self) {
        let vel_x_prev = self.velocity_x.clone();
        let vel_y_prev = self.velocity_y.clone();
        
        for y in 1..self.height-1 {
            for x in 1..self.width-1 {
                let idx = y * self.width + x;
                
                // Trace particle back in time
                let src_x = x as f32 - self.dt * self.velocity_x[idx] * 10.0;
                let src_y = y as f32 - self.dt * self.velocity_y[idx] * 10.0;
                
                let src_x = src_x.max(1.0).min((self.width - 2) as f32);
                let src_y = src_y.max(1.0).min((self.height - 2) as f32);
                
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
                
                self.velocity_x[idx] = (1.0 - sx) * (1.0 - sy) * vel_x_prev[idx00] +
                                      sx * (1.0 - sy) * vel_x_prev[idx01] +
                                      (1.0 - sx) * sy * vel_x_prev[idx10] +
                                      sx * sy * vel_x_prev[idx11];
                
                self.velocity_y[idx] = (1.0 - sx) * (1.0 - sy) * vel_y_prev[idx00] +
                                      sx * (1.0 - sy) * vel_y_prev[idx01] +
                                      (1.0 - sx) * sy * vel_y_prev[idx10] +
                                      sx * sy * vel_y_prev[idx11];
            }
        }
        
        self.set_velocity_boundaries();
    }

    fn advect_density(&mut self) {
        let density_prev = self.density.clone();
        
        for y in 1..self.height-1 {
            for x in 1..self.width-1 {
                let idx = y * self.width + x;
                
                // Trace particle back in time
                let src_x = x as f32 - self.dt * self.velocity_x[idx] * 10.0;
                let src_y = y as f32 - self.dt * self.velocity_y[idx] * 10.0;
                
                let src_x = src_x.max(1.0).min((self.width - 2) as f32);
                let src_y = src_y.max(1.0).min((self.height - 2) as f32);
                
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
                
                self.density[idx] = (1.0 - sx) * (1.0 - sy) * density_prev[idx00] +
                                   sx * (1.0 - sy) * density_prev[idx01] +
                                   (1.0 - sx) * sy * density_prev[idx10] +
                                   sx * sy * density_prev[idx11];
            }
        }
        
        self.set_density_boundaries();
    }

    fn set_velocity_boundaries(&mut self) {
        // Reflective boundaries with some damping
        for x in 0..self.width {
            self.velocity_x[x] *= 0.9;
            self.velocity_y[x] *= 0.9;
            self.velocity_x[(self.height - 1) * self.width + x] *= 0.9;
            self.velocity_y[(self.height - 1) * self.width + x] *= 0.9;
        }
        
        for y in 0..self.height {
            self.velocity_x[y * self.width] *= 0.9;
            self.velocity_y[y * self.width] *= 0.9;
            self.velocity_x[y * self.width + self.width - 1] *= 0.9;
            self.velocity_y[y * self.width + self.width - 1] *= 0.9;
        }
    }

    fn set_density_boundaries(&mut self) {
        // Gentle density fade at boundaries
        for x in 0..self.width {
            self.density[x] *= 0.99;
            self.density[(self.height - 1) * self.width + x] *= 0.99;
        }
        
        for y in 0..self.height {
            self.density[y * self.width] *= 0.99;
            self.density[y * self.width + self.width - 1] *= 0.99;
        }
    }

    fn apply_boundary_conditions(&mut self) {
        self.set_velocity_boundaries();
        self.set_density_boundaries();
    }
}