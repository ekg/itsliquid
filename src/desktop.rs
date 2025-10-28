use crate::FluidSolver;
use crate::export::ImageExporter;
use crate::render::Renderer;
use eframe::egui;

#[derive(PartialEq)]
pub enum FlowPattern {
    Upward,     // Buoyant flow
    Downward,   // Gravity flow
    Circular,   // Vortex flow
    Horizontal, // Wind flow
    Radial,     // Explosion/implosion
}

pub struct DesktopApp {
    simulation: FluidSolver,
    exporter: ImageExporter,
    paused: bool,
    show_velocity: bool,
    frame_count: usize,
    cell_size: f32,
    flow_pattern: FlowPattern,
    flow_strength: f32,
    projection_angle: f32, // Angle in degrees for fluid projection
    diffusion_strength: f32,
}

impl DesktopApp {
    pub fn new(width: usize, height: usize) -> Self {
        let mut simulation = FluidSolver::new(width, height);

        // Add some initial fluid
        for i in 0..10 {
            simulation.add_density(width / 2 + i, height / 2, 1.0);
            simulation.add_velocity(width / 2 + i, height / 2, glam::Vec2::new(2.0, 0.0));
        }

        Self {
            simulation,
            exporter: ImageExporter::new(width as u32, height as u32),
            paused: false,
            show_velocity: false,
            frame_count: 0,
            cell_size: 4.0, // Make cells larger for better visibility
            flow_pattern: FlowPattern::Circular,
            flow_strength: 2.0,
            projection_angle: 0.0, // Default: straight up
            diffusion_strength: 0.0001,
        }
    }
}

impl eframe::App for DesktopApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("itsliquid - Fluid Simulation");

            ui.horizontal(|ui| {
                if ui.button("Pause/Resume").clicked() {
                    self.paused = !self.paused;
                }

                if ui.button("Add Fluid").clicked() {
                    // Add fluid at a random position with configured flow
                    let x = rand::random::<usize>() % self.simulation.width;
                    let y = rand::random::<usize>() % self.simulation.height;
                    self.simulation.add_density(x, y, 1.0);
                    self.add_velocity_pattern(x, y);
                }

                if ui.button("Export PNG").clicked() {
                    let path = std::path::Path::new("frame.png");
                    if self
                        .exporter
                        .export_density_png(&self.simulation, path)
                        .is_ok()
                    {
                        ui.label("Frame exported!");
                    }
                }

                ui.checkbox(&mut self.show_velocity, "Show Velocity");
            });

            ui.horizontal(|ui| {
                ui.add(egui::Slider::new(&mut self.cell_size, 1.0..=10.0).text("Cell Size"));
                ui.add(egui::Slider::new(&mut self.flow_strength, 0.1..=5.0).text("Flow Strength"));
                ui.add(
                    egui::Slider::new(&mut self.diffusion_strength, 0.00000001..=0.00001)
                        .text("Diffusion"),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Flow Pattern:");
                ui.radio_value(&mut self.flow_pattern, FlowPattern::Upward, "Upward");
                ui.radio_value(&mut self.flow_pattern, FlowPattern::Downward, "Downward");
                ui.radio_value(&mut self.flow_pattern, FlowPattern::Circular, "Circular");
                ui.radio_value(
                    &mut self.flow_pattern,
                    FlowPattern::Horizontal,
                    "Horizontal",
                );
                ui.radio_value(&mut self.flow_pattern, FlowPattern::Radial, "Radial");
            });

            ui.horizontal(|ui| {
                ui.label("Projection Angle:");
                ui.add(egui::Slider::new(&mut self.projection_angle, 0.0..=360.0).text("°"));
                // Visual angle indicator
                let angle_rad = self.projection_angle.to_radians();
                let arrow_x = angle_rad.cos();
                let arrow_y = -angle_rad.sin(); // Negative because y increases downward
                ui.label(format!(
                    "→ ({:.1}°, {:.1}°)",
                    arrow_x * 90.0,
                    arrow_y * 90.0
                ));
            });

            ui.separator();

            // Calculate canvas size based on simulation dimensions and cell size
            let canvas_width = self.simulation.width as f32 * self.cell_size;
            let canvas_height = self.simulation.height as f32 * self.cell_size;

            // Simulation canvas
            let (rect, response) = ui.allocate_exact_size(
                egui::Vec2::new(canvas_width, canvas_height),
                egui::Sense::click_and_drag(),
            );

            // Handle mouse interaction
            if response.dragged() || response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let x = ((pos.x - rect.left()) / self.cell_size) as usize;
                    let y = ((pos.y - rect.top()) / self.cell_size) as usize;

                    if x < self.simulation.width && y < self.simulation.height {
                        // Add fluid with natural flow (upward buoyancy)
                        let amount = 1.0;
                        // Create circular flow pattern from mouse position
                        let dx = x as f32 - (self.simulation.width as f32 / 2.0);
                        let dy = y as f32 - (self.simulation.height as f32 / 2.0);
                        let vel_x = -dy * 0.01; // Rotational flow
                        let vel_y = dx * 0.01; // Rotational flow

                        self.simulation.add_density(x, y, amount);
                        self.simulation
                            .add_velocity(x, y, glam::Vec2::new(vel_x, vel_y));
                    }
                }
            }

            // Render simulation
            let painter = ui.painter();

            for y in 0..self.simulation.height {
                for x in 0..self.simulation.width {
                    let idx = y * self.simulation.width + x;
                    let density = self.simulation.density[idx].min(1.0).max(0.0);

                    let color = if self.show_velocity {
                        let vel_x = self.simulation.velocity_x[idx].abs().min(1.0);
                        let vel_y = self.simulation.velocity_y[idx].abs().min(1.0);
                        egui::Color32::from_rgb((vel_x * 255.0) as u8, (vel_y * 255.0) as u8, 128)
                    } else {
                        // Blue to white gradient based on density
                        let intensity = (density * 255.0) as u8;
                        egui::Color32::from_rgb(intensity, intensity, 255)
                    };

                    let rect = egui::Rect::from_min_size(
                        egui::Pos2::new(
                            rect.left() + x as f32 * self.cell_size,
                            rect.top() + y as f32 * self.cell_size,
                        ),
                        egui::Vec2::new(self.cell_size, self.cell_size),
                    );

                    painter.rect_filled(rect, 0.0, color);
                }
            }

            // Draw grid lines for better visibility
            for x in 0..=self.simulation.width {
                let line_x = rect.left() + x as f32 * self.cell_size;
                painter.line_segment(
                    [
                        egui::Pos2::new(line_x, rect.top()),
                        egui::Pos2::new(line_x, rect.bottom()),
                    ],
                    egui::Stroke::new(0.5, egui::Color32::from_gray(50)),
                );
            }

            for y in 0..=self.simulation.height {
                let line_y = rect.top() + y as f32 * self.cell_size;
                painter.line_segment(
                    [
                        egui::Pos2::new(rect.left(), line_y),
                        egui::Pos2::new(rect.right(), line_y),
                    ],
                    egui::Stroke::new(0.5, egui::Color32::from_gray(50)),
                );
            }

            // Update simulation if not paused
            if !self.paused {
                self.simulation.step();
                self.frame_count += 1;
            }

            ui.label(format!(
                "Frame: {} | Click/drag to add fluid | Cell Size: {:.1}",
                self.frame_count, self.cell_size
            ));
        });

        ctx.request_repaint();
    }
}

impl DesktopApp {
    fn add_velocity_pattern(&mut self, x: usize, y: usize) {
        let angle_rad = self.projection_angle.to_radians();
        let strength = self.flow_strength;

        match self.flow_pattern {
            FlowPattern::Upward => {
                self.simulation
                    .add_velocity(x, y, glam::Vec2::new(0.0, -strength));
            }
            FlowPattern::Downward => {
                self.simulation
                    .add_velocity(x, y, glam::Vec2::new(0.0, strength));
            }
            FlowPattern::Circular => {
                // Create vortex around the center
                let dx = x as f32 - (self.simulation.width as f32 / 2.0);
                let dy = y as f32 - (self.simulation.height as f32 / 2.0);
                let vel_x = -dy * strength * 0.01;
                let vel_y = dx * strength * 0.01;
                self.simulation
                    .add_velocity(x, y, glam::Vec2::new(vel_x, vel_y));
            }
            FlowPattern::Horizontal => {
                self.simulation
                    .add_velocity(x, y, glam::Vec2::new(strength, 0.0));
            }
            FlowPattern::Radial => {
                // Radial flow from center
                let dx = x as f32 - (self.simulation.width as f32 / 2.0);
                let dy = y as f32 - (self.simulation.height as f32 / 2.0);
                let dist = (dx * dx + dy * dy).sqrt().max(0.1);
                let vel_x = (dx / dist) * strength;
                let vel_y = (dy / dist) * strength;
                self.simulation
                    .add_velocity(x, y, glam::Vec2::new(vel_x, vel_y));
            }
        }

        // Also add the directional projection
        let proj_x = angle_rad.cos() * strength;
        let proj_y = -angle_rad.sin() * strength; // Negative because y increases downward
        self.simulation
            .add_velocity(x, y, glam::Vec2::new(proj_x, proj_y));
    }
}
