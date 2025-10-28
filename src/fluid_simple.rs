use glam::Vec2;

#[derive(Debug, Clone)]
pub struct FluidSimulation {
    pub width: usize,
    pub height: usize,
    pub density: Vec<f32>,
    pub velocity_x: Vec<f32>,
    pub velocity_y: Vec<f32>,
    pub diffusion: f32,
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
            diffusion: 0.0000001, // Much lower diffusion (liquid-like)
            viscosity: 0.00001,   // Slightly higher viscosity for liquid
            dt: 0.02,             // Smaller timestep for liquid stability
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
        // Simple diffusion and advection
        let mut new_density = self.density.clone();
        let mut new_vel_x = self.velocity_x.clone();
        let mut new_vel_y = self.velocity_y.clone();

        // Diffuse density
        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                let idx = y * self.width + x;
                new_density[idx] = self.density[idx]
                    + self.diffusion
                        * (self.density[idx - 1]
                            + self.density[idx + 1]
                            + self.density[idx - self.width]
                            + self.density[idx + self.width]
                            - 4.0 * self.density[idx]);
            }
        }

        // Diffuse velocity
        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                let idx = y * self.width + x;
                new_vel_x[idx] = self.velocity_x[idx]
                    + self.viscosity
                        * (self.velocity_x[idx - 1]
                            + self.velocity_x[idx + 1]
                            + self.velocity_x[idx - self.width]
                            + self.velocity_x[idx + self.width]
                            - 4.0 * self.velocity_x[idx]);
                new_vel_y[idx] = self.velocity_y[idx]
                    + self.viscosity
                        * (self.velocity_y[idx - 1]
                            + self.velocity_y[idx + 1]
                            + self.velocity_y[idx - self.width]
                            + self.velocity_y[idx + self.width]
                            - 4.0 * self.velocity_y[idx]);
            }
        }

        // Simple advection
        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                let idx = y * self.width + x;

                // Backtrace position
                let src_x = (x as f32 - self.dt * new_vel_x[idx])
                    .max(1.0)
                    .min((self.width - 2) as f32);
                let src_y = (y as f32 - self.dt * new_vel_y[idx])
                    .max(1.0)
                    .min((self.height - 2) as f32);

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
                self.density[idx] = (1.0 - sx) * (1.0 - sy) * new_density[idx00]
                    + sx * (1.0 - sy) * new_density[idx01]
                    + (1.0 - sx) * sy * new_density[idx10]
                    + sx * sy * new_density[idx11];

                // Advect velocity
                self.velocity_x[idx] = (1.0 - sx) * (1.0 - sy) * new_vel_x[idx00]
                    + sx * (1.0 - sy) * new_vel_x[idx01]
                    + (1.0 - sx) * sy * new_vel_x[idx10]
                    + sx * sy * new_vel_x[idx11];
                self.velocity_y[idx] = (1.0 - sx) * (1.0 - sy) * new_vel_y[idx00]
                    + sx * (1.0 - sy) * new_vel_y[idx01]
                    + (1.0 - sx) * sy * new_vel_y[idx10]
                    + sx * sy * new_vel_y[idx11];
            }
        }

        // Apply boundary conditions
        self.apply_boundary_conditions();
    }

    fn apply_boundary_conditions(&mut self) {
        // Much gentler boundary conditions
        for x in 0..self.width {
            // Gradually fade density at boundaries
            self.density[x] *= 0.99; // top
            self.density[(self.height - 1) * self.width + x] *= 0.99; // bottom
            // Gentle velocity damping
            self.velocity_x[x] *= 0.995;
            self.velocity_y[x] *= 0.995;
            self.velocity_x[(self.height - 1) * self.width + x] *= 0.995;
            self.velocity_y[(self.height - 1) * self.width + x] *= 0.995;
        }

        for y in 0..self.height {
            self.density[y * self.width] *= 0.99; // left
            self.density[y * self.width + self.width - 1] *= 0.99; // right
            self.velocity_x[y * self.width] *= 0.995;
            self.velocity_y[y * self.width] *= 0.995;
            self.velocity_x[y * self.width + self.width - 1] *= 0.995;
            self.velocity_y[y * self.width + self.width - 1] *= 0.995;
        }
    }
}
