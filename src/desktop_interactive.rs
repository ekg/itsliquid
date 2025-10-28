use crate::InteractiveFluid;
use eframe::egui;

pub struct InteractiveApp {
    simulation: InteractiveFluid,
    paused: bool,
    frame_count: usize,
    cell_size: f32,
    mouse_dragging: bool,
    mouse_start_pos: Option<egui::Pos2>,
    mouse_current_pos: Option<egui::Pos2>,
    dye_colors: Vec<(f32, f32, f32)>,
    current_dye_index: usize,
    resolution_scale: usize, // 1, 2, 4, 8, etc. - factor to scale base resolution
    base_width: usize,
    base_height: usize,
}

impl InteractiveApp {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            simulation: InteractiveFluid::new(width, height),
            paused: false,
            frame_count: 0,
            cell_size: 4.0,
            mouse_dragging: false,
            mouse_start_pos: None,
            mouse_current_pos: None,
            dye_colors: vec![
                (1.0, 0.0, 0.0), // Red
                (0.0, 1.0, 0.0), // Green
                (0.0, 0.0, 1.0), // Blue
                (1.0, 1.0, 0.0), // Yellow
                (1.0, 0.0, 1.0), // Magenta
                (0.0, 1.0, 1.0), // Cyan
            ],
            current_dye_index: 0,
            resolution_scale: 1,
            base_width: width,
            base_height: height,
        }
    }

    fn change_resolution(&mut self, scale: usize) {
        if scale != self.resolution_scale && scale >= 1 && scale <= 8 {
            self.resolution_scale = scale;
            let new_width = self.base_width * scale;
            let new_height = self.base_height * scale;

            // Create new simulation with scaled resolution
            self.simulation = InteractiveFluid::new(new_width, new_height);

            // Reset simulation state
            self.mouse_dragging = false;
            self.mouse_start_pos = None;
            self.mouse_current_pos = None;
        }
    }
}

impl eframe::App for InteractiveApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("itsliquid - Interactive Fluid Simulation");

            ui.horizontal(|ui| {
                if ui.button("Pause/Resume").clicked() {
                    self.paused = !self.paused;
                }

                ui.add(egui::Slider::new(&mut self.cell_size, 1.0..=10.0).text("Cell Size"));

                ui.label("Dye Color:");
                for (i, _) in self.dye_colors.iter().enumerate() {
                    if ui.radio_value(&mut self.current_dye_index, i, format!("Color {}", i + 1)).clicked() {
                        self.current_dye_index = i;
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Resolution Scale:");

                // Resolution scale buttons (1x, 2x, 4x, 8x)
                for &scale in &[1, 2, 4, 8] {
                    let label = format!("{}x", scale);
                    let is_current = self.resolution_scale == scale;

                    if ui.selectable_label(is_current, label).clicked() {
                        self.change_resolution(scale);
                    }
                }

                ui.label(format!(" ({}x{} cells)", self.simulation.width, self.simulation.height));
            });

            ui.separator();

            // Calculate canvas size
            let canvas_width = self.simulation.width as f32 * self.cell_size;
            let canvas_height = self.simulation.height as f32 * self.cell_size;

            // Simulation canvas
            let (rect, response) = ui.allocate_exact_size(
                egui::Vec2::new(canvas_width, canvas_height),
                egui::Sense::click_and_drag()
            );

            // Handle left-click drag for fluid pulling
            if response.dragged_by(egui::PointerButton::Primary) {
                if let Some(pos) = response.interact_pointer_pos() {
                    let x = ((pos.x - rect.left()) / self.cell_size) as usize;
                    let y = ((pos.y - rect.top()) / self.cell_size) as usize;

                    if x < self.simulation.width && y < self.simulation.height {
                        if !self.mouse_dragging {
                            // Start dragging
                            self.mouse_dragging = true;
                            self.mouse_start_pos = Some(pos);
                        }

                        self.mouse_current_pos = Some(pos);

                        // Calculate drag direction and apply force
                        if let (Some(start), Some(current)) = (self.mouse_start_pos, self.mouse_current_pos) {
                            let drag_vec = current - start;
                            let force_strength = 5.0;
                            let force = glam::Vec2::new(drag_vec.x * force_strength, drag_vec.y * force_strength);

                            // Apply force in a circular area
                            self.simulation.add_force(x, y, force, 3.0);
                        }
                    }
                }
            } else if response.drag_stopped_by(egui::PointerButton::Primary) {
                // Release left button - create vortex effect
                if let Some(pos) = response.interact_pointer_pos() {
                    let x = ((pos.x - rect.left()) / self.cell_size) as usize;
                    let y = ((pos.y - rect.top()) / self.cell_size) as usize;

                    if x < self.simulation.width && y < self.simulation.height {
                        // Create vortex by applying rotational force
                        let vortex_strength = 10.0;

                        // Apply vortex force in a larger area
                        for dy in -5..=5 {
                            for dx in -5..=5 {
                                let px = (x as i32 + dx) as usize;
                                let py = (y as i32 + dy) as usize;

                                if px < self.simulation.width && py < self.simulation.height {
                                    let dist_sq = (dx * dx + dy * dy) as f32;
                                    if dist_sq <= 25.0 {
                                        // Rotational force (perpendicular to radius)
                                        let force_x = -dy as f32 * vortex_strength;
                                        let force_y = dx as f32 * vortex_strength;
                                        let falloff = 1.0 - dist_sq / 25.0;

                                        let idx = py * self.simulation.width + px;
                                        self.simulation.velocity_x[idx] += force_x * falloff;
                                        self.simulation.velocity_y[idx] += force_y * falloff;
                                    }
                                }
                            }
                        }
                    }
                }

                self.mouse_dragging = false;
                self.mouse_start_pos = None;
                self.mouse_current_pos = None;
            }

            // Handle right-click for dye injection
            if response.secondary_clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let x = ((pos.x - rect.left()) / self.cell_size) as usize;
                    let y = ((pos.y - rect.top()) / self.cell_size) as usize;

                    if x < self.simulation.width && y < self.simulation.height {
                        // Add dye droplet
                        let dye_color = self.dye_colors[self.current_dye_index];

                        // Add dye in a small circular pattern
                        for dy in -2..=2 {
                            for dx in -2..=2 {
                                let px = (x as i32 + dx) as usize;
                                let py = (y as i32 + dy) as usize;

                                if px < self.simulation.width && py < self.simulation.height {
                                    let dist_sq = (dx * dx + dy * dy) as f32;
                                    if dist_sq <= 4.0 {
                                        let falloff = 1.0 - dist_sq / 4.0;
                                        self.simulation.add_dye(px, py, (
                                            dye_color.0 * falloff,
                                            dye_color.1 * falloff,
                                            dye_color.2 * falloff
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Continuous dye injection while right button is held and dragged
            if response.dragged_by(egui::PointerButton::Secondary) {
                if let Some(pos) = response.interact_pointer_pos() {
                    let x = ((pos.x - rect.left()) / self.cell_size) as usize;
                    let y = ((pos.y - rect.top()) / self.cell_size) as usize;

                    if x < self.simulation.width && y < self.simulation.height {
                        // Add dye droplet
                        let dye_color = self.dye_colors[self.current_dye_index];

                        // Add dye in a small circular pattern
                        for dy in -2..=2 {
                            for dx in -2..=2 {
                                let px = (x as i32 + dx) as usize;
                                let py = (y as i32 + dy) as usize;

                                if px < self.simulation.width && py < self.simulation.height {
                                    let dist_sq = (dx * dx + dy * dy) as f32;
                                    if dist_sq <= 4.0 {
                                        let falloff = 1.0 - dist_sq / 4.0;
                                        self.simulation.add_dye(px, py, (
                                            dye_color.0 * falloff * 0.3, // Reduce intensity for continuous stream
                                            dye_color.1 * falloff * 0.3,
                                            dye_color.2 * falloff * 0.3
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Render simulation
            let painter = ui.painter();

            for y in 0..self.simulation.height {
                for x in 0..self.simulation.width {
                    let idx = y * self.simulation.width + x;

                    // Get dye color
                    let r = self.simulation.dye_r[idx].min(1.0).max(0.0);
                    let g = self.simulation.dye_g[idx].min(1.0).max(0.0);
                    let b = self.simulation.dye_b[idx].min(1.0).max(0.0);

                    // Create color based on dye concentration
                    let color = egui::Color32::from_rgb(
                        (r * 255.0) as u8,
                        (g * 255.0) as u8,
                        (b * 255.0) as u8
                    );

                    let rect = egui::Rect::from_min_size(
                        egui::Pos2::new(rect.left() + x as f32 * self.cell_size,
                                       rect.top() + y as f32 * self.cell_size),
                        egui::Vec2::new(self.cell_size, self.cell_size)
                    );

                    painter.rect_filled(rect, 0.0, color);
                }
            }

            // Draw grid lines
            for x in 0..=self.simulation.width {
                let line_x = rect.left() + x as f32 * self.cell_size;
                painter.line_segment(
                    [egui::Pos2::new(line_x, rect.top()), egui::Pos2::new(line_x, rect.bottom())],
                    egui::Stroke::new(0.5, egui::Color32::from_gray(30))
                );
            }

            for y in 0..=self.simulation.height {
                let line_y = rect.top() + y as f32 * self.cell_size;
                painter.line_segment(
                    [egui::Pos2::new(rect.left(), line_y), egui::Pos2::new(rect.right(), line_y)],
                    egui::Stroke::new(0.5, egui::Color32::from_gray(30))
                );
            }

            // Draw drag indicator if dragging
            if let (Some(start), Some(current)) = (self.mouse_start_pos, self.mouse_current_pos) {
                painter.line_segment(
                    [start, current],
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 255, 255))
                );

                painter.circle_filled(current, 3.0, egui::Color32::from_rgb(255, 255, 255));
            }

            // Update simulation if not paused
            if !self.paused {
                self.simulation.step();
                self.frame_count += 1;
            }

            ui.label(format!("Frame: {} | Resolution: {}x{} | Left-click+drag: Pull fluid | Right-click+hold: Stream dye | Cell Size: {:.1}",
                self.frame_count, self.simulation.width, self.simulation.height, self.cell_size));
        });

        ctx.request_repaint();
    }
}
