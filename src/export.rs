use std::path::Path;
use crate::render::Renderer;
use crate::{FluidSimulation, FluidSolver, WorkingFluid, FluidFinal};

pub trait FluidData {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn density(&self) -> &[f32];
    fn velocity_x(&self) -> &[f32];
    fn velocity_y(&self) -> &[f32];
}

impl FluidData for FluidSimulation {
    fn width(&self) -> usize { self.width }
    fn height(&self) -> usize { self.height }
    fn density(&self) -> &[f32] { &self.density }
    fn velocity_x(&self) -> &[f32] { &self.velocity_x }
    fn velocity_y(&self) -> &[f32] { &self.velocity_y }
}

impl FluidData for FluidSolver {
    fn width(&self) -> usize { self.width }
    fn height(&self) -> usize { self.height }
    fn density(&self) -> &[f32] { &self.density }
    fn velocity_x(&self) -> &[f32] { &self.velocity_x }
    fn velocity_y(&self) -> &[f32] { &self.velocity_y }
}

impl FluidData for WorkingFluid {
    fn width(&self) -> usize { self.width }
    fn height(&self) -> usize { self.height }
    fn density(&self) -> &[f32] { &self.density }
    fn velocity_x(&self) -> &[f32] { &self.velocity_x }
    fn velocity_y(&self) -> &[f32] { &self.velocity_y }
}

impl FluidData for FluidFinal {
    fn width(&self) -> usize { self.width }
    fn height(&self) -> usize { self.height }
    fn density(&self) -> &[f32] { &self.density }
    fn velocity_x(&self) -> &[f32] { &self.velocity_x }
    fn velocity_y(&self) -> &[f32] { &self.velocity_y }
}

pub struct ImageExporter {
    renderer: Renderer,
}

impl ImageExporter {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            renderer: Renderer::new(width, height),
        }
    }

    pub fn export_density_png(&self, simulation: &impl FluidData, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let img = self.renderer.render_to_image(simulation);
        img.save(path)?;
        Ok(())
    }

    pub fn export_velocity_png(&self, simulation: &impl FluidData, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let img = self.renderer.render_velocity_field(simulation);
        img.save(path)?;
        Ok(())
    }

    pub fn export_frame_sequence(&self, 
        simulation: &mut (impl FluidData + Step), 
        steps: usize, 
        output_dir: &Path,
        prefix: &str
    ) -> Result<(), Box<dyn std::error::Error>> {
        for i in 0..steps {
            simulation.step();
            
            let filename = format!("{}_frame_{:04}.png", prefix, i);
            let path = output_dir.join(filename);
            
            self.export_density_png(simulation, &path)?;
        }
        Ok(())
    }
}

pub trait Step {
    fn step(&mut self);
}

impl Step for FluidSimulation {
    fn step(&mut self) {
        self.step();
    }
}

impl Step for FluidSolver {
    fn step(&mut self) {
        self.step();
    }
}