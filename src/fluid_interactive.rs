use crate::FluidSimulation;
use glam::Vec2;

#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct InteractiveFluid {
    pub width: usize,
    pub height: usize,
    pub velocity_x: Vec<f32>,
    pub velocity_y: Vec<f32>,
    pub velocity_x_prev: Vec<f32>,
    pub velocity_y_prev: Vec<f32>,
    pub dye_r: Vec<f32>, // Red dye concentration
    pub dye_g: Vec<f32>, // Green dye concentration
    pub dye_b: Vec<f32>, // Blue dye concentration
    pub dye_r_prev: Vec<f32>,
    pub dye_g_prev: Vec<f32>,
    pub dye_b_prev: Vec<f32>,
    pub pressure: Vec<f32>,
    pub divergence: Vec<f32>,
    pub dt: f32,
    pub viscosity: f32,
    pub dye_diffusion: f32,
}

impl FluidSimulation for InteractiveFluid {
    fn step(&mut self) {
        self.step()
    }

    fn add_force(&mut self, x: usize, y: usize, force: glam::Vec2) {
        self.add_force(x, y, force, 3.0)
    }

    fn add_dye(&mut self, x: usize, y: usize, color: (f32, f32, f32)) {
        self.add_dye(x, y, color)
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}

impl InteractiveFluid {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        Self {
            width,
            height,
            velocity_x: vec![0.0; size],
            velocity_y: vec![0.0; size],
            velocity_x_prev: vec![0.0; size],
            velocity_y_prev: vec![0.0; size],
            dye_r: vec![0.0; size],
            dye_g: vec![0.0; size],
            dye_b: vec![0.0; size],
            dye_r_prev: vec![0.0; size],
            dye_g_prev: vec![0.0; size],
            dye_b_prev: vec![0.0; size],
            pressure: vec![0.0; size],
            divergence: vec![0.0; size],
            dt: 0.1,
            viscosity: 0.001,
            dye_diffusion: 0.0001,
        }
    }

    pub fn add_dye(&mut self, x: usize, y: usize, color: (f32, f32, f32)) {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            self.dye_r[idx] += color.0;
            self.dye_g[idx] += color.1;
            self.dye_b[idx] += color.2;
        }
    }

    pub fn add_force(&mut self, x: usize, y: usize, force: Vec2, radius: f32) {
        if x < self.width && y < self.height {
            // Apply force in a circular area
            let _center_x = x as f32;
            let _center_y = y as f32;

            let r_sq = radius * radius;

            for dy in (-radius as i32)..=(radius as i32) {
                for dx in (-radius as i32)..=(radius as i32) {
                    let px = (x as i32 + dx) as usize;
                    let py = (y as i32 + dy) as usize;

                    if px < self.width && py < self.height {
                        let dist_sq = (dx * dx + dy * dy) as f32;
                        if dist_sq <= r_sq {
                            let idx = py * self.width + px;
                            let falloff = 1.0 - dist_sq / r_sq;

                            self.velocity_x[idx] += force.x * falloff;
                            self.velocity_y[idx] += force.y * falloff;
                        }
                    }
                }
            }
        }
    }

    pub fn step(&mut self) {
        // Save current state
        self.velocity_x_prev.copy_from_slice(&self.velocity_x);
        self.velocity_y_prev.copy_from_slice(&self.velocity_y);
        self.dye_r_prev.copy_from_slice(&self.dye_r);
        self.dye_g_prev.copy_from_slice(&self.dye_g);
        self.dye_b_prev.copy_from_slice(&self.dye_b);

        // Step 1: Diffuse velocity
        self.diffuse_velocity();

        // Step 2: Project velocity (make divergence-free)
        self.project_velocity();

        // Step 3: Advect velocity
        self.advect_velocity();

        // Step 4: Project velocity again
        self.project_velocity();

        // Step 5: Diffuse dye
        self.diffuse_dye();

        // Step 6: Advect dye
        self.advect_dye();

        // Apply boundary conditions
        self.set_boundaries();
    }

    pub fn diffuse_velocity(&mut self) {
        let a = self.dt * self.viscosity * (self.width * self.height) as f32;

        for _ in 0..4 {
            for y in 1..self.height - 1 {
                for x in 1..self.width - 1 {
                    let idx = y * self.width + x;
                    self.velocity_x[idx] = (self.velocity_x_prev[idx]
                        + a * (self.velocity_x[idx - 1]
                            + self.velocity_x[idx + 1]
                            + self.velocity_x[idx - self.width]
                            + self.velocity_x[idx + self.width]))
                        / (1.0 + 4.0 * a);

                    self.velocity_y[idx] = (self.velocity_y_prev[idx]
                        + a * (self.velocity_y[idx - 1]
                            + self.velocity_y[idx + 1]
                            + self.velocity_y[idx - self.width]
                            + self.velocity_y[idx + self.width]))
                        / (1.0 + 4.0 * a);
                }
            }
            self.set_velocity_boundaries();
        }
    }

    pub fn diffuse_dye(&mut self) {
        // Track mass before diffusion
        let total_r_before: f32 = self.dye_r.iter().sum();
        let total_g_before: f32 = self.dye_g.iter().sum();
        let total_b_before: f32 = self.dye_b.iter().sum();

        let a = self.dt * self.dye_diffusion * (self.width * self.height) as f32;

        for _ in 0..2 {
            for y in 1..self.height - 1 {
                for x in 1..self.width - 1 {
                    let idx = y * self.width + x;

                    self.dye_r[idx] = (self.dye_r_prev[idx]
                        + a * (self.dye_r[idx - 1]
                            + self.dye_r[idx + 1]
                            + self.dye_r[idx - self.width]
                            + self.dye_r[idx + self.width]))
                        / (1.0 + 4.0 * a);

                    self.dye_g[idx] = (self.dye_g_prev[idx]
                        + a * (self.dye_g[idx - 1]
                            + self.dye_g[idx + 1]
                            + self.dye_g[idx - self.width]
                            + self.dye_g[idx + self.width]))
                        / (1.0 + 4.0 * a);

                    self.dye_b[idx] = (self.dye_b_prev[idx]
                        + a * (self.dye_b[idx - 1]
                            + self.dye_b[idx + 1]
                            + self.dye_b[idx - self.width]
                            + self.dye_b[idx + self.width]))
                        / (1.0 + 4.0 * a);
                }
            }
            self.set_dye_boundaries();
        }

        // Apply mass conservation correction after diffusion
        let total_r_after: f32 = self.dye_r.iter().sum();
        let total_g_after: f32 = self.dye_g.iter().sum();
        let total_b_after: f32 = self.dye_b.iter().sum();

        if total_r_after > 1e-10 {
            let scale_r = total_r_before / total_r_after;
            for val in self.dye_r.iter_mut() {
                *val *= scale_r;
            }
        }
        if total_g_after > 1e-10 {
            let scale_g = total_g_before / total_g_after;
            for val in self.dye_g.iter_mut() {
                *val *= scale_g;
            }
        }
        if total_b_after > 1e-10 {
            let scale_b = total_b_before / total_b_after;
            for val in self.dye_b.iter_mut() {
                *val *= scale_b;
            }
        }
    }

    pub fn advect_velocity(&mut self) {
        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
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
                self.velocity_x[idx] = (1.0 - sx) * (1.0 - sy) * self.velocity_x_prev[idx00]
                    + sx * (1.0 - sy) * self.velocity_x_prev[idx01]
                    + (1.0 - sx) * sy * self.velocity_x_prev[idx10]
                    + sx * sy * self.velocity_x_prev[idx11];

                self.velocity_y[idx] = (1.0 - sx) * (1.0 - sy) * self.velocity_y_prev[idx00]
                    + sx * (1.0 - sy) * self.velocity_y_prev[idx01]
                    + (1.0 - sx) * sy * self.velocity_y_prev[idx10]
                    + sx * sy * self.velocity_y_prev[idx11];
            }
        }
        self.set_velocity_boundaries();
    }

    pub fn advect_dye(&mut self) {
        // Calculate total dye mass before advection for conservation
        #[cfg(not(target_arch = "wasm32"))]
        let (total_r_before, total_g_before, total_b_before) = {
            self.dye_r_prev.par_iter()
                .zip(&self.dye_g_prev)
                .zip(&self.dye_b_prev)
                .map(|((&r, &g), &b)| (r, g, b))
                .reduce(|| (0.0, 0.0, 0.0), |(r1, g1, b1), (r2, g2, b2)| (r1 + r2, g1 + g2, b1 + b2))
        };

        #[cfg(target_arch = "wasm32")]
        let (total_r_before, total_g_before, total_b_before) = {
            let r: f32 = self.dye_r_prev.iter().sum();
            let g: f32 = self.dye_g_prev.iter().sum();
            let b: f32 = self.dye_b_prev.iter().sum();
            (r, g, b)
        };

        // Advection (serial for WASM compatibility)
        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
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

                // Advect dye with bilinear interpolation
                self.dye_r[idx] = (1.0 - sx) * (1.0 - sy) * self.dye_r_prev[idx00]
                    + sx * (1.0 - sy) * self.dye_r_prev[idx01]
                    + (1.0 - sx) * sy * self.dye_r_prev[idx10]
                    + sx * sy * self.dye_r_prev[idx11];

                self.dye_g[idx] = (1.0 - sx) * (1.0 - sy) * self.dye_g_prev[idx00]
                    + sx * (1.0 - sy) * self.dye_g_prev[idx01]
                    + (1.0 - sx) * sy * self.dye_g_prev[idx10]
                    + sx * sy * self.dye_g_prev[idx11];

                self.dye_b[idx] = (1.0 - sx) * (1.0 - sy) * self.dye_b_prev[idx00]
                    + sx * (1.0 - sy) * self.dye_b_prev[idx01]
                    + (1.0 - sx) * sy * self.dye_b_prev[idx10]
                    + sx * sy * self.dye_b_prev[idx11];
            }
        }

        self.set_dye_boundaries();

        // Apply mass conservation correction
        let total_r_after: f32 = self.dye_r.iter().sum();
        let total_g_after: f32 = self.dye_g.iter().sum();
        let total_b_after: f32 = self.dye_b.iter().sum();

        // Rescale to conserve mass
        if total_r_after > 1e-10 && total_g_after > 1e-10 && total_b_after > 1e-10 {
            let scale_r = total_r_before / total_r_after;
            let scale_g = total_g_before / total_g_after;
            let scale_b = total_b_before / total_b_after;

            for i in 0..self.dye_r.len() {
                self.dye_r[i] *= scale_r;
                self.dye_g[i] *= scale_g;
                self.dye_b[i] *= scale_b;
            }
        }
    }

    pub fn project_velocity(&mut self) {
        let h = 1.0 / self.width as f32;

        // Calculate divergence
        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                let idx = y * self.width + x;
                self.divergence[idx] = -0.5
                    * h
                    * (self.velocity_x[idx + 1] - self.velocity_x[idx - 1]
                        + self.velocity_y[idx + self.width]
                        - self.velocity_y[idx - self.width]);
                self.pressure[idx] = 0.0;
            }
        }

        self.set_pressure_boundaries();

        // Solve for pressure with adaptive convergence
        let convergence_threshold = 0.001;
        let max_iterations = 20;

        for iter in 0..max_iterations {
            let mut max_change = 0.0f32;

            for y in 1..self.height - 1 {
                for x in 1..self.width - 1 {
                    let idx = y * self.width + x;
                    let old_pressure = self.pressure[idx];
                    self.pressure[idx] = (self.divergence[idx]
                        + self.pressure[idx - 1]
                        + self.pressure[idx + 1]
                        + self.pressure[idx - self.width]
                        + self.pressure[idx + self.width])
                        / 4.0;

                    // Track convergence
                    let change = (self.pressure[idx] - old_pressure).abs();
                    if change > max_change {
                        max_change = change;
                    }
                }
            }
            self.set_pressure_boundaries();

            // Early exit if converged
            if iter > 5 && max_change < convergence_threshold {
                break;
            }
        }

        // Subtract pressure gradient
        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                let idx = y * self.width + x;
                self.velocity_x[idx] -= 0.5 * (self.pressure[idx + 1] - self.pressure[idx - 1]) / h;
                self.velocity_y[idx] -=
                    0.5 * (self.pressure[idx + self.width] - self.pressure[idx - self.width]) / h;
            }
        }

        self.set_velocity_boundaries();
    }

    fn set_boundaries(&mut self) {
        self.set_velocity_boundaries();
        self.set_dye_boundaries();
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

    fn set_dye_boundaries(&mut self) {
        for x in 0..self.width {
            self.dye_r[x] = self.dye_r[self.width + x];
            self.dye_g[x] = self.dye_g[self.width + x];
            self.dye_b[x] = self.dye_b[self.width + x];

            self.dye_r[(self.height - 1) * self.width + x] =
                self.dye_r[(self.height - 2) * self.width + x];
            self.dye_g[(self.height - 1) * self.width + x] =
                self.dye_g[(self.height - 2) * self.width + x];
            self.dye_b[(self.height - 1) * self.width + x] =
                self.dye_b[(self.height - 2) * self.width + x];
        }

        for y in 0..self.height {
            self.dye_r[y * self.width] = self.dye_r[y * self.width + 1];
            self.dye_g[y * self.width] = self.dye_g[y * self.width + 1];
            self.dye_b[y * self.width] = self.dye_b[y * self.width + 1];

            self.dye_r[y * self.width + self.width - 1] =
                self.dye_r[y * self.width + self.width - 2];
            self.dye_g[y * self.width + self.width - 1] =
                self.dye_g[y * self.width + self.width - 2];
            self.dye_b[y * self.width + self.width - 1] =
                self.dye_b[y * self.width + self.width - 2];
        }
    }

    fn set_pressure_boundaries(&mut self) {
        for x in 0..self.width {
            self.pressure[x] = self.pressure[self.width + x];
            self.pressure[(self.height - 1) * self.width + x] =
                self.pressure[(self.height - 2) * self.width + x];
        }

        for y in 0..self.height {
            self.pressure[y * self.width] = self.pressure[y * self.width + 1];
            self.pressure[y * self.width + self.width - 1] =
                self.pressure[y * self.width + self.width - 2];
        }
    }
}
