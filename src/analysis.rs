use crate::export::FluidData;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct FluidMetrics {
    pub total_mass: f32,
    pub max_density: f32,
    pub avg_density: f32,
    pub total_kinetic_energy: f32,
    pub max_velocity: f32,
    pub avg_velocity: f32,
    pub density_entropy: f32,
    pub velocity_divergence: f32,
    pub vorticity: f32,
    pub frame: usize,
}

impl FluidMetrics {
    pub fn analyze(simulation: &impl FluidData, frame: usize) -> Self {
        let mut total_mass: f32 = 0.0;
        let mut max_density: f32 = 0.0;
        let mut total_kinetic_energy: f32 = 0.0;
        let mut max_velocity: f32 = 0.0;
        let mut velocity_sum: f32 = 0.0;
        let mut density_histogram = HashMap::new();
        let mut total_divergence = 0.0;
        let mut total_vorticity = 0.0;

        let size = simulation.width() * simulation.height();

        for y in 1..simulation.height() - 1 {
            for x in 1..simulation.width() - 1 {
                let idx = y * simulation.width() + x;
                let density = simulation.density()[idx];
                let vel_x = simulation.velocity_x()[idx];
                let vel_y = simulation.velocity_y()[idx];

                total_mass += density;
                max_density = max_density.max(density);

                let velocity_magnitude = (vel_x * vel_x + vel_y * vel_y).sqrt();
                total_kinetic_energy += 0.5 * density * velocity_magnitude * velocity_magnitude;
                max_velocity = max_velocity.max(velocity_magnitude);
                velocity_sum += velocity_magnitude;

                // Quantize density for entropy calculation
                let quantized_density = (density * 10.0).floor() as usize;
                *density_histogram.entry(quantized_density).or_insert(0) += 1;

                // Calculate divergence (∇·v)
                let divergence = (simulation.velocity_x()[idx + 1]
                    - simulation.velocity_x()[idx - 1]
                    + simulation.velocity_y()[idx + simulation.width()]
                    - simulation.velocity_y()[idx - simulation.width()])
                    / 2.0;
                total_divergence += divergence.abs();

                // Calculate vorticity (∇×v)
                let vorticity = (simulation.velocity_y()[idx + 1]
                    - simulation.velocity_y()[idx - 1]
                    - simulation.velocity_x()[idx + simulation.width()]
                    - simulation.velocity_x()[idx - simulation.width()])
                    / 2.0;
                total_vorticity += vorticity.abs();
            }
        }

        let avg_density = total_mass / size as f32;
        let avg_velocity = velocity_sum / size as f32;

        // Calculate entropy of density distribution
        let mut entropy = 0.0;
        for &count in density_histogram.values() {
            let probability = count as f32 / size as f32;
            if probability > 0.0 {
                entropy -= probability * probability.log2();
            }
        }

        let velocity_divergence = total_divergence / size as f32;
        let vorticity = total_vorticity / size as f32;

        Self {
            total_mass,
            max_density,
            avg_density,
            total_kinetic_energy,
            max_velocity,
            avg_velocity,
            density_entropy: entropy,
            velocity_divergence,
            vorticity,
            frame,
        }
    }

    pub fn print_summary(&self) {
        println!("Frame {} Metrics:", self.frame);
        println!("  Total Mass: {:.6}", self.total_mass);
        println!("  Max Density: {:.6}", self.max_density);
        println!("  Avg Density: {:.6}", self.avg_density);
        println!("  Kinetic Energy: {:.6}", self.total_kinetic_energy);
        println!("  Max Velocity: {:.6}", self.max_velocity);
        println!("  Avg Velocity: {:.6}", self.avg_velocity);
        println!("  Density Entropy: {:.6}", self.density_entropy);
        println!("  Velocity Divergence: {:.6}", self.velocity_divergence);
        println!("  Vorticity: {:.6}", self.vorticity);
        println!();
    }
}

pub struct AnalysisRecorder {
    pub metrics_history: Vec<FluidMetrics>,
}

impl AnalysisRecorder {
    pub fn new() -> Self {
        Self {
            metrics_history: Vec::new(),
        }
    }

    pub fn record_frame(&mut self, simulation: &impl FluidData, frame: usize) {
        let metrics = FluidMetrics::analyze(simulation, frame);
        self.metrics_history.push(metrics);
    }

    pub fn print_trends(&self) {
        if self.metrics_history.len() < 2 {
            return;
        }

        let first = &self.metrics_history[0];
        let last = &self.metrics_history[self.metrics_history.len() - 1];

        println!("=== TREND ANALYSIS ===");
        println!(
            "Mass change: {:.6} -> {:.6} ({:+.3}%)",
            first.total_mass,
            last.total_mass,
            (last.total_mass - first.total_mass) / first.total_mass * 100.0
        );
        println!(
            "Kinetic Energy change: {:.6} -> {:.6} ({:+.3}%)",
            first.total_kinetic_energy,
            last.total_kinetic_energy,
            (last.total_kinetic_energy - first.total_kinetic_energy)
                / first.total_kinetic_energy.max(0.001)
                * 100.0
        );
        println!(
            "Entropy change: {:.6} -> {:.6} ({:+.3}%)",
            first.density_entropy,
            last.density_entropy,
            (last.density_entropy - first.density_entropy) / first.density_entropy.max(0.001)
                * 100.0
        );
    }
}
