use crate::InteractiveFluid;
use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tool {
    Dye,
    Force,
    Eyedropper,
    Attractor,
}

pub struct InteractiveApp {
    simulation: InteractiveFluid,
    paused: bool,
    frame_count: usize,
    selected_tool: Tool,
    mouse_start_pos: Option<egui::Pos2>,
    mouse_current_pos: Option<egui::Pos2>,
    dye_colors: Vec<(f32, f32, f32)>,
    current_dye_index: usize,
    dye_intensity: f32,
    force_intensity: f32,
    attractor_radius: f32,
    attractor_strength: f32,
    resolution_scale: usize,
    base_width: usize,
    base_height: usize,
    continuous_color_pos: Option<(usize, usize)>,
    last_window_size: Option<egui::Vec2>,
    sampled_color: Option<(f32, f32, f32)>,
}

impl InteractiveApp {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            simulation: InteractiveFluid::new(width, height),
            paused: false,
            frame_count: 0,
            selected_tool: Tool::Dye,
            mouse_start_pos: None,
            mouse_current_pos: None,
            dye_colors: vec![
                (1.0, 0.0, 0.0), // Red
                (0.0, 1.0, 0.0), // Green
                (0.0, 0.0, 1.0), // Blue
                (1.0, 1.0, 0.0), // Yellow
                (1.0, 0.0, 1.0), // Magenta
                (0.0, 1.0, 1.0), // Cyan
                (1.0, 1.0, 1.0), // White
                (0.0, 0.0, 0.0), // Black (negative dye - removes color)
            ],
            current_dye_index: 0,
            dye_intensity: 0.5,
            force_intensity: 0.5,
            attractor_radius: 50.0,
            attractor_strength: 0.3,
            resolution_scale: 1,
            base_width: width,
            base_height: height,
            continuous_color_pos: None,
            last_window_size: None,
            sampled_color: None,
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
            self.mouse_start_pos = None;
            self.mouse_current_pos = None;
            self.continuous_color_pos = None;
        }
    }
}

impl eframe::App for InteractiveApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Detect window resize (or first load)
        let current_size = ctx.screen_rect().size();
        let should_resize = if let Some(last_size) = self.last_window_size {
            // Check if size changed significantly
            (current_size.x - last_size.x).abs() > 10.0 || (current_size.y - last_size.y).abs() > 10.0
        } else {
            // First frame - always resize to fit window
            true
        };

        if should_resize {
            // Window resized - recalculate simulation dimensions
            // Keep cell size consistent, but use all available vertical space
            let cell_size = 8.0;
            let new_width = (current_size.x / cell_size).max(50.0) as usize;
            // Account for toolbar (~90px) and bottom panel + safe area (~170px) = ~260px total
            let new_height = ((current_size.y - 260.0) / cell_size).max(50.0) as usize;
            self.simulation = InteractiveFluid::new(new_width, new_height);
            self.base_width = new_width;
            self.base_height = new_height;
        }
        self.last_window_size = Some(current_size);

        // Toolbar at the top - organized in multiple rows to prevent overflow
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.vertical(|ui| {
                // Row 1: Title and Help
                ui.horizontal(|ui| {
                    ui.heading("itsliquid");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.hyperlink_to("📖 Help", "https://github.com/ekg/itsliquid#readme").clicked() {
                            // Link opens in browser
                        }
                    });
                });

                ui.add_space(2.0);

                // Row 2: Tool selection
                ui.horizontal(|ui| {
                    ui.label("Tool:");
                    if ui.selectable_label(self.selected_tool == Tool::Dye, "🎨 Dye").clicked() {
                        self.selected_tool = Tool::Dye;
                    }
                    if ui.selectable_label(self.selected_tool == Tool::Force, "💨 Force").clicked() {
                        self.selected_tool = Tool::Force;
                    }
                    if ui.selectable_label(self.selected_tool == Tool::Eyedropper, "🔍 Eyedropper").clicked() {
                        self.selected_tool = Tool::Eyedropper;
                    }
                    if ui.selectable_label(self.selected_tool == Tool::Attractor, "🧲 Attractor").clicked() {
                        self.selected_tool = Tool::Attractor;
                    }
                });

                ui.add_space(2.0);

                // Row 3: Controls
                ui.horizontal(|ui| {
                    if ui.button(if self.paused { "▶ Resume" } else { "⏸ Pause" }).clicked() {
                        self.paused = !self.paused;
                    }

                    if ui.button("🗑 Clear").clicked() {
                        // Clear all dye and velocity
                        for i in 0..self.simulation.dye_r.len() {
                            self.simulation.dye_r[i] = 0.0;
                            self.simulation.dye_g[i] = 0.0;
                            self.simulation.dye_b[i] = 0.0;
                            self.simulation.velocity_x[i] = 0.0;
                            self.simulation.velocity_y[i] = 0.0;
                        }
                    }

                    ui.separator();

                    for &scale in &[1, 2, 4, 8] {
                        if ui.selectable_label(self.resolution_scale == scale, format!("{}x", scale)).clicked() {
                            self.change_resolution(scale);
                        }
                    }

                    ui.separator();

                    ui.label(format!("Grid: {}x{}", self.simulation.width, self.simulation.height));
                });
            });
        });

        // Tool-specific bottom panels - show BEFORE CentralPanel to reserve space
        match self.selected_tool {
            Tool::Dye => {
                // Color picker at the bottom for Dye tool
                egui::TopBottomPanel::bottom("color_picker")
                    .min_height(150.0)
                    .show_separator_line(true)
                    .show(ctx, |ui| {
                    ui.add_space(8.0);
                    ui.vertical(|ui| {
                        // Color swatches - one row
                        ui.horizontal(|ui| {
                            ui.label("Color:");
                            for (i, &color) in self.dye_colors.iter().enumerate() {
                                let color_32 = egui::Color32::from_rgb(
                                    (color.0 * 255.0) as u8,
                                    (color.1 * 255.0) as u8,
                                    (color.2 * 255.0) as u8
                                );
                                let size = egui::Vec2::new(26.0, 26.0);
                                let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

                                // Always draw a gray border so black/white are visible
                                ui.painter().rect_stroke(rect, 1.5, egui::Stroke::new(1.2, egui::Color32::GRAY));

                                // Fill with color
                                ui.painter().rect_filled(rect.shrink(2.0), 1.5, color_32);

                                // Selection indicator on top
                                if self.current_dye_index == i {
                                    ui.painter().rect_stroke(rect.shrink(0.5), 1.5, egui::Stroke::new(2.0, egui::Color32::WHITE));
                                }

                                if response.clicked() {
                                    self.current_dye_index = i;
                                }
                            }
                        });

                        ui.add_space(4.0);

                        // Dye intensity slider
                        ui.horizontal(|ui| {
                            ui.label("Intensity:");
                            ui.add(egui::Slider::new(&mut self.dye_intensity, 0.1..=10.0)
                                .show_value(true)
                                .step_by(0.1));
                        });
                    });
                    ui.add_space(40.0);
                });
            },
            Tool::Force => {
                // Force intensity slider at the bottom for Force tool
                egui::TopBottomPanel::bottom("force_controls")
                    .min_height(110.0)
                    .show_separator_line(true)
                    .show(ctx, |ui| {
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label("Force Intensity:");
                        ui.add(egui::Slider::new(&mut self.force_intensity, 0.01..=3.0)
                            .show_value(true)
                            .step_by(0.01));
                    });
                    ui.add_space(40.0);
                });
            },
            Tool::Eyedropper => {
                // Display sampled color info at the bottom for Eyedropper tool
                egui::TopBottomPanel::bottom("eyedropper_info")
                    .min_height(110.0)
                    .show_separator_line(true)
                    .show(ctx, |ui| {
                    ui.add_space(8.0);
                    ui.vertical(|ui| {
                        if let Some((r, g, b)) = self.sampled_color {
                            // Apply Reinhard tone mapping for display
                            let r_display = r / (1.0 + r);
                            let g_display = g / (1.0 + g);
                            let b_display = b / (1.0 + b);

                            // Convert to 0-255 range for display
                            let r_255 = (r_display * 255.0).round() as u8;
                            let g_255 = (g_display * 255.0).round() as u8;
                            let b_255 = (b_display * 255.0).round() as u8;

                            ui.horizontal(|ui| {
                                // Color preview swatch - same size as dye swatches
                                let color_32 = egui::Color32::from_rgb(r_255, g_255, b_255);
                                let size = egui::Vec2::new(26.0, 26.0);
                                let (rect, _response) = ui.allocate_exact_size(size, egui::Sense::hover());
                                ui.painter().rect_stroke(rect, 1.5, egui::Stroke::new(1.2, egui::Color32::GRAY));
                                ui.painter().rect_filled(rect.shrink(2.0), 1.5, color_32);

                                ui.separator();

                                // RGB values
                                ui.label(format!("RGB: ({}, {}, {})", r_255, g_255, b_255));

                                ui.separator();

                                // Hex value - selectable
                                let hex_string = format!("#{:02X}{:02X}{:02X}", r_255, g_255, b_255);
                                ui.label("Hex:");
                                let mut hex_text = hex_string.clone();
                                ui.add(egui::TextEdit::singleline(&mut hex_text)
                                    .desired_width(80.0)
                                    .interactive(false));

                                ui.separator();

                                // Raw HDR values
                                ui.label(format!("HDR: ({:.3}, {:.3}, {:.3})", r, g, b));
                            });
                        } else {
                            ui.label("Click on a cell to sample its color");
                        }
                    });
                    ui.add_space(40.0);
                });
            },
            Tool::Attractor => {
                // Attractor controls at the bottom
                egui::TopBottomPanel::bottom("attractor_controls")
                    .min_height(110.0)
                    .show_separator_line(true)
                    .show(ctx, |ui| {
                    ui.add_space(8.0);
                    ui.vertical(|ui| {
                        // Attraction radius slider
                        ui.horizontal(|ui| {
                            ui.label("Radius:");
                            ui.add(egui::Slider::new(&mut self.attractor_radius, 10.0..=200.0)
                                .show_value(true)
                                .step_by(5.0));
                        });

                        ui.add_space(4.0);

                        // Attraction strength slider
                        ui.horizontal(|ui| {
                            ui.label("Strength:");
                            ui.add(egui::Slider::new(&mut self.attractor_strength, 0.01..=2.0)
                                .show_value(true)
                                .step_by(0.01));
                        });
                    });
                    ui.add_space(40.0);
                });
            }
        }

        // Canvas fills the entire remaining space
        egui::CentralPanel::default().show(ctx, |ui| {
            // Use all available space
            let available_size = ui.available_size();

            // Calculate cell size based on canvas size to fit simulation
            let cell_size_x = available_size.x / self.simulation.width as f32;
            let cell_size_y = available_size.y / self.simulation.height as f32;
            let cell_size = cell_size_x.min(cell_size_y);

            // Calculate actual canvas size based on simulation grid and cell size
            let canvas_width = self.simulation.width as f32 * cell_size;
            let canvas_height = self.simulation.height as f32 * cell_size;

            // Simulation canvas - centered in available space
            let (rect, response) = ui.allocate_exact_size(
                egui::Vec2::new(canvas_width, canvas_height),
                egui::Sense::click_and_drag()
            );

            // TOOL-BASED INTERACTION
            match self.selected_tool {
                Tool::Dye => {
                    // Dye tool: Click/tap to add dye, hold to paint continuously
                    if response.clicked() || response.dragged() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            let x = ((pos.x - rect.left()) / cell_size) as usize;
                            let y = ((pos.y - rect.top()) / cell_size) as usize;

                            if x < self.simulation.width && y < self.simulation.height {
                                let dye_color = self.dye_colors[self.current_dye_index];

                                // Check if black (negative dye) is selected
                                let is_negative = dye_color.0 == 0.0 && dye_color.1 == 0.0 && dye_color.2 == 0.0;

                                // Add/remove dye in a small circular pattern
                                for dy in -2..=2 {
                                    for dx in -2..=2 {
                                        let px = (x as i32 + dx) as usize;
                                        let py = (y as i32 + dy) as usize;

                                        if px < self.simulation.width && py < self.simulation.height {
                                            let dist_sq = (dx * dx + dy * dy) as f32;
                                            if dist_sq <= 4.0 {
                                                let falloff = 1.0 - dist_sq / 4.0;
                                                let drag_factor = if response.dragged() { 0.6 } else { 1.0 };
                                                let intensity = falloff * self.dye_intensity * drag_factor;

                                                let idx = py * self.simulation.width + px;

                                                if is_negative {
                                                    // Black removes dye
                                                    self.simulation.dye_r[idx] = (self.simulation.dye_r[idx] - intensity).max(0.0);
                                                    self.simulation.dye_g[idx] = (self.simulation.dye_g[idx] - intensity).max(0.0);
                                                    self.simulation.dye_b[idx] = (self.simulation.dye_b[idx] - intensity).max(0.0);
                                                } else {
                                                    // Normal colors add dye
                                                    self.simulation.add_dye(px, py, (
                                                        dye_color.0 * intensity,
                                                        dye_color.1 * intensity,
                                                        dye_color.2 * intensity
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                Tool::Force => {
                    // Force tool: Click and drag to create continuous force
                    if response.drag_started() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            self.mouse_start_pos = Some(pos);
                            self.mouse_current_pos = Some(pos);
                        }
                    } else if response.dragged() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            self.mouse_current_pos = Some(pos);

                            // Apply force continuously while dragging
                            if let Some(start) = self.mouse_start_pos {
                                let x = ((start.x - rect.left()) / cell_size) as usize;
                                let y = ((start.y - rect.top()) / cell_size) as usize;

                                if x < self.simulation.width && y < self.simulation.height {
                                    let force_vec = pos - start;
                                    let force = glam::Vec2::new(force_vec.x * self.force_intensity, force_vec.y * self.force_intensity);

                                    // Apply force at start location
                                    self.simulation.add_force(x, y, force, 3.0);
                                }
                            }
                        }
                    } else if response.drag_stopped() {
                        self.mouse_start_pos = None;
                        self.mouse_current_pos = None;
                    }
                },
                Tool::Eyedropper => {
                    // Eyedropper tool: Click to sample color
                    if response.clicked() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            let x = ((pos.x - rect.left()) / cell_size) as usize;
                            let y = ((pos.y - rect.top()) / cell_size) as usize;

                            if x < self.simulation.width && y < self.simulation.height {
                                let idx = y * self.simulation.width + x;
                                let r = self.simulation.dye_r[idx];
                                let g = self.simulation.dye_g[idx];
                                let b = self.simulation.dye_b[idx];

                                // Store the raw color values for display
                                self.sampled_color = Some((r, g, b));
                            }
                        }
                    }
                },
                Tool::Attractor => {
                    // Attractor tool: Click/drag to pull dye toward a point
                    if response.clicked() || response.dragged() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            let attractor_x = ((pos.x - rect.left()) / cell_size) as f32;
                            let attractor_y = ((pos.y - rect.top()) / cell_size) as f32;

                            // Apply radial inward force within radius
                            for y in 0..self.simulation.height {
                                for x in 0..self.simulation.width {
                                    let dx = attractor_x - x as f32;
                                    let dy = attractor_y - y as f32;
                                    let distance = (dx * dx + dy * dy).sqrt();

                                    // Only apply force within attraction radius
                                    if distance > 0.1 && distance < self.attractor_radius / cell_size {
                                        // Calculate direction toward attractor
                                        let dir_x = dx / distance;
                                        let dir_y = dy / distance;

                                        // Apply force with falloff based on distance
                                        let falloff = 1.0 - (distance / (self.attractor_radius / cell_size));
                                        let force_magnitude = self.attractor_strength * falloff;

                                        let force = glam::Vec2::new(
                                            dir_x * force_magnitude,
                                            dir_y * force_magnitude
                                        );

                                        self.simulation.add_force(x, y, force, 1.0);
                                    }
                                }
                            }
                        }
                    }
                },
            }

            // Render simulation
            let painter = ui.painter();

            // Render each cell
            for y in 0..self.simulation.height {
                for x in 0..self.simulation.width {
                    let idx = y * self.simulation.width + x;

                    // Get dye color with Reinhard tone mapping for HDR values
                    // Maps [0, ∞) to [0, 1) smoothly
                    let r_raw = self.simulation.dye_r[idx];
                    let g_raw = self.simulation.dye_g[idx];
                    let b_raw = self.simulation.dye_b[idx];

                    // Reinhard tone mapping: x / (1 + x)
                    let r = (r_raw / (1.0 + r_raw)).max(0.0);
                    let g = (g_raw / (1.0 + g_raw)).max(0.0);
                    let b = (b_raw / (1.0 + b_raw)).max(0.0);

                    // Create color based on dye concentration
                    let color = egui::Color32::from_rgb(
                        (r * 255.0) as u8,
                        (g * 255.0) as u8,
                        (b * 255.0) as u8
                    );

                    let cell_rect = egui::Rect::from_min_size(
                        egui::Pos2::new(rect.left() + x as f32 * cell_size,
                                       rect.top() + y as f32 * cell_size),
                        egui::Vec2::new(cell_size.ceil() + 0.5, cell_size.ceil() + 0.5)
                    );

                    painter.rect_filled(cell_rect, 0.0, color);
                }
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
            // Run 1 step per frame at all resolutions
            if !self.paused {
                self.simulation.step();
                self.frame_count += 1;
            }
        });

        ctx.request_repaint();
    }
}
