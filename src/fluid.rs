use glam::Vec2;

#[derive(Debug, Clone)]
pub struct FluidSimulation {
    pub width: usize,
    pub height: usize,
    pub density: Vec<f32>,
    pub velocity_x: Vec<f32>,
    pub velocity_y: Vec<f32>,
    pub pressure: Vec<f32>,
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
            pressure: vec![0.0; size],
            diffusion: 0.0001,
            viscosity: 0.0001,
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
        let mut vel_x_temp = self.velocity_x.clone();
        let mut vel_y_temp = self.velocity_y.clone();
        
        self.diffuse(0, &mut vel_x_temp, self.viscosity);
        self.diffuse(1, &mut vel_y_temp, self.viscosity);
        
        self.velocity_x = vel_x_temp;
        self.velocity_y = vel_y_temp;
        
        self.project();
        
        let vel_x_copy = self.velocity_x.clone();
        let vel_y_copy = self.velocity_y.clone();
        
        self.advect(0, &mut self.velocity_x, &vel_x_copy, &vel_x_copy, &vel_y_copy);
        self.advect(1, &mut self.velocity_y, &vel_y_copy, &vel_x_copy, &vel_y_copy);
        self.project();
        
        let mut density_temp = self.density.clone();
        self.diffuse(0, &mut density_temp, self.diffusion);
        self.density = density_temp;
        
        let vel_x_copy = self.velocity_x.clone();
        let vel_y_copy = self.velocity_y.clone();
        self.advect(0, &mut self.density, &self.density, &vel_x_copy, &vel_y_copy);
    }

    fn diffuse(&self, b: usize, x: &mut [f32], diff: f32) {
        let a = self.dt * diff * (self.width * self.height) as f32;
        self.linear_solve(b, x, x, a, 1.0 + 4.0 * a);
    }

    fn project(&mut self) {
        let mut div = vec![0.0; self.width * self.height];
        let mut p = vec![0.0; self.width * self.height];

        let vel_x = self.velocity_x.clone();
        let vel_y = self.velocity_y.clone();

        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                let idx = y * self.width + x;
                div[idx] = -0.5 * (
                    vel_x[idx + 1] - vel_x[idx - 1] +
                    vel_y[idx + self.width] - vel_y[idx - self.width]
                );
                p[idx] = 0.0;
            }
        }

        self.set_bnd(0, &mut div);
        self.set_bnd(0, &mut p);
        self.linear_solve(0, &mut p, &div, 1.0, 4.0);

        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                let idx = y * self.width + x;
                self.velocity_x[idx] -= 0.5 * (p[idx + 1] - p[idx - 1]);
                self.velocity_y[idx] -= 0.5 * (p[idx + self.width] - p[idx - self.width]);
            }
        }

        self.set_bnd(1, &mut self.velocity_x);
        self.set_bnd(2, &mut self.velocity_y);
    }

    fn advect(&self, b: usize, d: &mut [f32], d0: &[f32], vel_x: &[f32], vel_y: &[f32]) {
        let dt0 = self.dt * (self.width - 2) as f32;

        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                let idx = y * self.width + x;
                let x_pos = x as f32 - dt0 * vel_x[idx];
                let y_pos = y as f32 - dt0 * vel_y[idx];

                let x_pos = x_pos.max(0.5).min(self.width as f32 - 1.5);
                let y_pos = y_pos.max(0.5).min(self.height as f32 - 1.5);

                let x0 = x_pos.floor() as usize;
                let x1 = x0 + 1;
                let y0 = y_pos.floor() as usize;
                let y1 = y0 + 1;

                let s1 = x_pos - x0 as f32;
                let s0 = 1.0 - s1;
                let t1 = y_pos - y0 as f32;
                let t0 = 1.0 - t1;

                let idx00 = y0 * self.width + x0;
                let idx01 = y0 * self.width + x1;
                let idx10 = y1 * self.width + x0;
                let idx11 = y1 * self.width + x1;

                d[idx] = s0 * (t0 * d0[idx00] + t1 * d0[idx10]) +
                         s1 * (t0 * d0[idx01] + t1 * d0[idx11]);
            }
        }

        self.set_bnd(b, d);
    }

    fn linear_solve(&self, b: usize, x: &mut [f32], x0: &[f32], a: f32, c: f32) {
        let x0 = x0.to_vec(); // Create a copy to avoid borrowing issues
        
        for _ in 0..20 {
            for y in 1..self.height - 1 {
                for x_pos in 1..self.width - 1 {
                    let idx = y * self.width + x_pos;
                    x[idx] = (x0[idx] + a * (
                        x[idx - 1] + x[idx + 1] +
                        x[idx - self.width] + x[idx + self.width]
                    )) / c;
                }
            }
            self.set_bnd(b, x);
        }
    }

    fn set_bnd(&self, b: usize, x: &mut [f32]) {
        for i in 1..self.width - 1 {
            x[i] = if b == 2 { -x[i + self.width] } else { x[i + self.width] };
            x[i + (self.height - 1) * self.width] = if b == 2 { -x[i + (self.height - 2) * self.width] } else { x[i + (self.height - 2) * self.width] };
        }

        for j in 1..self.height - 1 {
            x[j * self.width] = if b == 1 { -x[j * self.width + 1] } else { x[j * self.width + 1] };
            x[j * self.width + self.width - 1] = if b == 1 { -x[j * self.width + self.width - 2] } else { x[j * self.width + self.width - 2] };
        }

        x[0] = 0.5 * (x[1] + x[self.width]);
        x[self.width - 1] = 0.5 * (x[self.width - 2] + x[2 * self.width - 1]);
        x[(self.height - 1) * self.width] = 0.5 * (x[(self.height - 2) * self.width] + x[(self.height - 1) * self.width + 1]);
        x[(self.height - 1) * self.width + self.width - 1] = 0.5 * (x[(self.height - 2) * self.width + self.width - 1] + x[(self.height - 1) * self.width + self.width - 2]);
    }
}