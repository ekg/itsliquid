//! Core fluid simulation library for itsliquid

pub mod fluid_simple;
pub mod fluid_proper;
pub mod fluid_working;
pub mod fluid_final;
pub mod fluid_interactive;
pub mod render;
pub mod export;
pub mod desktop;
pub mod desktop_interactive;
pub mod analysis;

pub use fluid_simple::FluidSimulation;
pub use fluid_proper::FluidSolver;
pub use fluid_working::WorkingFluid;
pub use fluid_final::FluidFinal;
pub use fluid_interactive::InteractiveFluid;
pub use render::Renderer;
pub use export::ImageExporter;
pub use desktop::DesktopApp;
pub use desktop_interactive::InteractiveApp;
pub use analysis::{FluidMetrics, AnalysisRecorder};