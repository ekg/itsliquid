use image::{ImageBuffer, Rgb, RgbImage};
use crate::export::FluidData;

pub struct Renderer {
    width: u32,
    height: u32,
}

impl Renderer {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn render_to_image(&self, simulation: &impl FluidData) -> RgbImage {
        let mut img = ImageBuffer::new(self.width, self.height);
        
        // Calculate scaling factors
        let scale_x = self.width as f32 / simulation.width() as f32;
        let scale_y = self.height as f32 / simulation.height() as f32;
        
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let sim_x = (x as f32 / scale_x) as usize;
            let sim_y = (y as f32 / scale_y) as usize;
            
            if sim_x < simulation.width() && sim_y < simulation.height() {
                let idx = sim_y * simulation.width() + sim_x;
                let density = simulation.density()[idx].min(1.0).max(0.0);
                
                // Create a proper fluid visualization
                // Blue for low density, white for high density
                let intensity = (density * 255.0) as u8;
                *pixel = Rgb([intensity, intensity, 255]);
            } else {
                *pixel = Rgb([0, 0, 0]);
            }
        }
        
        img
    }

    pub fn render_velocity_field(&self, simulation: &impl FluidData) -> RgbImage {
        let mut img = ImageBuffer::new(self.width, self.height);
        
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let sim_x = (x as f32 / self.width as f32 * simulation.width() as f32) as usize;
            let sim_y = (y as f32 / self.height as f32 * simulation.height() as f32) as usize;
            
            if sim_x < simulation.width() && sim_y < simulation.height() {
                let idx = sim_y * simulation.width() + sim_x;
                let vel_x = simulation.velocity_x()[idx];
                let vel_y = simulation.velocity_y()[idx];
                
                // Map velocity to color (red for x, green for y)
                let r = ((vel_x.abs() * 255.0).min(255.0)) as u8;
                let g = ((vel_y.abs() * 255.0).min(255.0)) as u8;
                let b = 128;
                
                *pixel = Rgb([r, g, b]);
            } else {
                *pixel = Rgb([0, 0, 0]);
            }
        }
        
        img
    }
}