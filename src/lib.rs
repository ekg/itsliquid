//! Core fluid simulation library for itsliquid

pub mod analysis;
pub mod desktop;
pub mod desktop_interactive;
pub mod export;
pub mod fluid_final;
pub mod fluid_interactive;
pub mod fluid_proper;
pub mod fluid_simple;
pub mod fluid_working;
pub mod render;

#[cfg(feature = "gpu")]
pub mod gpu_minimal;

#[cfg(feature = "gpu")]
pub mod gpu_functional;

#[cfg(feature = "gpu")]
pub mod desktop_gpu;

// Unified fluid simulation trait
pub trait FluidSimulation {
    fn step(&mut self);
    fn add_force(&mut self, x: usize, y: usize, force: glam::Vec2);
    fn add_dye(&mut self, x: usize, y: usize, color: (f32, f32, f32));
    fn width(&self) -> usize;
    fn height(&self) -> usize;
}

// Feature-based implementation selection
#[cfg(feature = "cpu")]
pub type DefaultFluid = fluid_interactive::InteractiveFluid;

#[cfg(all(feature = "gpu", not(feature = "cpu")))]
pub type DefaultFluid = gpu_functional::FunctionalGPUFluid;

pub use analysis::{AnalysisRecorder, FluidMetrics};
pub use desktop::DesktopApp;
pub use desktop_interactive::InteractiveApp;
pub use export::ImageExporter;
pub use fluid_final::FluidFinal;
pub use fluid_interactive::InteractiveFluid;
pub use fluid_proper::FluidSolver;
pub use fluid_working::WorkingFluid;
pub use render::Renderer;

#[cfg(feature = "gpu")]
pub use desktop_gpu::GPUInteractiveApp;
