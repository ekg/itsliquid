use crate::InteractiveFluid;
#[cfg(target_arch = "wasm32")]
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
#[cfg(target_arch = "wasm32")]
use base64::Engine as _;
#[cfg(target_arch = "wasm32")]
use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use serde_json;
#[cfg(target_arch = "wasm32")]
use web_sys;
use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tool {
    Dye,
    Force,
    Eyedropper,
    Attractor,
    Eraser,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PersistentElementType {
    DyeSource { color: (f32, f32, f32), intensity: f32 },
    ForceSource { direction: (f32, f32), intensity: f32 },
    AttractorSource { strength: f32 },
}

#[derive(Debug, Clone, Copy)]
struct PersistentElement {
    element_type: PersistentElementType,
    x: f32,
    y: f32,
    radius: f32,
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
    spiral_angle: f32, // 0-90 degrees: 0=pure inward, 90=pure tangential
    resolution_scale: usize,
    base_width: usize,
    base_height: usize,
    continuous_color_pos: Option<(usize, usize)>,
    last_window_size: Option<egui::Vec2>,
    sampled_color: Option<(f32, f32, f32)>,
    attractor_pos: Option<egui::Pos2>,
    attractor_grid_pos: Option<(f32, f32)>, // Grid coordinates for dye trap
    persistent_elements: Vec<PersistentElement>,
    placement_mode: bool,
    eraser_radius: f32,
    eraser_pos: Option<egui::Pos2>,
    #[cfg(target_arch = "wasm32")]
    url_state_loaded: bool,
    #[cfg(target_arch = "wasm32")]
    last_share_hash: Option<String>,
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
            attractor_strength: 5.0,
            spiral_angle: 70.0, // No longer used - keeping for backward compat
            resolution_scale: 1,
            base_width: width,
            base_height: height,
            continuous_color_pos: None,
            last_window_size: None,
            sampled_color: None,
            attractor_pos: None,
            attractor_grid_pos: None,
            persistent_elements: Vec::new(),
            placement_mode: false,
            eraser_radius: 30.0,
            eraser_pos: None,
            #[cfg(target_arch = "wasm32")]
            url_state_loaded: false,
            #[cfg(target_arch = "wasm32")]
            last_share_hash: None,
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
        // WASM: on first frame, try to load share state from URL
        #[cfg(target_arch = "wasm32")]
        {
            if !self.url_state_loaded {
                self.try_load_share_state_from_url();
                self.url_state_loaded = true;
            }
        }
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
                    if ui.selectable_label(self.selected_tool == Tool::Dye, "🎨").clicked() {
                        self.selected_tool = Tool::Dye;
                    }
                    if ui.selectable_label(self.selected_tool == Tool::Force, "💨").clicked() {
                        self.selected_tool = Tool::Force;
                    }
                    if ui.selectable_label(self.selected_tool == Tool::Eyedropper, "🔍").clicked() {
                        self.selected_tool = Tool::Eyedropper;
                    }
                    if ui.selectable_label(self.selected_tool == Tool::Attractor, "🌀").clicked() {
                        self.selected_tool = Tool::Attractor;
                    }
                    if ui.selectable_label(self.selected_tool == Tool::Eraser, "🗑").clicked() {
                        self.selected_tool = Tool::Eraser;
                    }

                    ui.separator();

                    // Placement mode toggle
                    if ui.selectable_label(self.placement_mode, "📌").clicked() {
                        self.placement_mode = !self.placement_mode;
                    }
                });

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
                            ui.add(egui::Slider::new(&mut self.dye_intensity, 0.1..=100.0)
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
                            ui.add(egui::Slider::new(&mut self.attractor_radius, 1.0..=200.0)
                                .show_value(true)
                                .step_by(1.0));
                        });

                        ui.add_space(4.0);

                        // Attraction strength slider
                        ui.horizontal(|ui| {
                            ui.label("Strength:");
                            ui.add(egui::Slider::new(&mut self.attractor_strength, 0.1..=100.0)
                                .show_value(true)
                                .step_by(0.1));
                        });
                    });
                    ui.add_space(40.0);
                });
            },
            Tool::Eraser => {
                // Eraser controls at the bottom
                egui::TopBottomPanel::bottom("eraser_controls")
                    .min_height(110.0)
                    .show_separator_line(true)
                    .show(ctx, |ui| {
                    ui.add_space(8.0);
                    ui.vertical(|ui| {
                        // Eraser radius slider
                        ui.horizontal(|ui| {
                            ui.label("Radius:");
                            ui.add(egui::Slider::new(&mut self.eraser_radius, 10.0..=100.0)
                                .show_value(true)
                                .step_by(1.0));
                        });
                    });
                    ui.add_space(40.0);
                });
            },
            _ => {}
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
                    if self.placement_mode {
                        // In placement mode: click or drag to place persistent dye sources
                        let is_interacting = response.clicked() || response.dragged();

                        if is_interacting {
                            if let Some(pos) = response.interact_pointer_pos() {
                                let grid_x = ((pos.x - rect.left()) / cell_size) as f32;
                                let grid_y = ((pos.y - rect.top()) / cell_size) as f32;

                                // Only add if not too close to existing elements (avoid overlap)
                                let min_spacing = 5.0; // Grid cells
                                let should_add = self.persistent_elements.iter().all(|elem| {
                                    let dx = elem.x - grid_x;
                                    let dy = elem.y - grid_y;
                                    let dist = (dx * dx + dy * dy).sqrt();
                                    dist > min_spacing
                                });

                                if should_add {
                                    self.persistent_elements.push(PersistentElement {
                                        element_type: PersistentElementType::DyeSource {
                                            color: self.dye_colors[self.current_dye_index],
                                            intensity: self.dye_intensity,
                                        },
                                        x: grid_x,
                                        y: grid_y,
                                        radius: 3.0,
                                    });
                                }
                            }
                        }

                        // Disable placement mode when interaction ends
                        if response.drag_stopped() {
                            self.placement_mode = false;
                        } else if !response.dragged() && !response.is_pointer_button_down_on() {
                            // Also disable if not dragging and pointer is up (handles single click)
                            if is_interacting {
                                self.placement_mode = false;
                            }
                        }
                    } else {
                        // Normal mode: Click/tap to add dye, hold to paint continuously
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
                    }
                },
                Tool::Force => {
                    // Force tool: Click and drag to create force
                    if response.drag_started() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            self.mouse_start_pos = Some(pos);
                            self.mouse_current_pos = Some(pos);
                        }
                    } else if response.dragged() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            self.mouse_current_pos = Some(pos);

                            // Apply force continuously while dragging (only if not in placement mode)
                            if !self.placement_mode {
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
                        }
                    } else if response.drag_stopped() {
                        // In placement mode, create persistent element on drag stop
                        if self.placement_mode {
                            if let (Some(start), Some(current)) = (self.mouse_start_pos, self.mouse_current_pos) {
                                let grid_x = ((start.x - rect.left()) / cell_size) as f32;
                                let grid_y = ((start.y - rect.top()) / cell_size) as f32;

                                let dx = current.x - start.x;
                                let dy = current.y - start.y;

                                self.persistent_elements.push(PersistentElement {
                                    element_type: PersistentElementType::ForceSource {
                                        direction: (dx, dy),
                                        intensity: self.force_intensity,
                                    },
                                    x: grid_x,
                                    y: grid_y,
                                    radius: 3.0,
                                });

                                self.placement_mode = false;
                            }
                        }

                        self.mouse_start_pos = None;
                        self.mouse_current_pos = None;
                    }
                },
                Tool::Eyedropper => {
                    // Eyedropper tool: Click to sample color (no placement mode)
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
                    if self.placement_mode {
                        // In placement mode: click to place persistent attractor
                        if response.clicked() {
                            if let Some(pos) = response.interact_pointer_pos() {
                                let grid_x = ((pos.x - rect.left()) / cell_size) as f32;
                                let grid_y = ((pos.y - rect.top()) / cell_size) as f32;

                                self.persistent_elements.push(PersistentElement {
                                    element_type: PersistentElementType::AttractorSource {
                                        strength: self.attractor_strength,
                                    },
                                    x: grid_x,
                                    y: grid_y,
                                    radius: self.attractor_radius / cell_size,
                                });
                                self.placement_mode = false;
                            }
                        }
                    } else {
                        // Normal mode: Apply temporary attractor while holding
                        if response.clicked() || response.dragged() {
                            if let Some(pos) = response.interact_pointer_pos() {
                                self.attractor_pos = Some(pos);

                                let attractor_x = ((pos.x - rect.left()) / cell_size) as f32;
                                let attractor_y = ((pos.y - rect.top()) / cell_size) as f32;

                                // Store grid position
                                self.attractor_grid_pos = Some((attractor_x, attractor_y));

                                let radius_cells = self.attractor_radius / cell_size;

                                // Point sink with proper fluid dynamics formula
                                let smoothing = 2.0;
                                let dead_zone = radius_cells * 0.2;

                                for y in 0..self.simulation.height {
                                    for x in 0..self.simulation.width {
                                        let dx = x as f32 - attractor_x;
                                        let dy = y as f32 - attractor_y;
                                        let r_squared = dx * dx + dy * dy;
                                        let r = r_squared.sqrt();

                                        if r > dead_zone && r < radius_cells {
                                            let idx = y * self.simulation.width + x;

                                            let factor = -self.attractor_strength /
                                                (2.0 * std::f32::consts::PI * (r_squared + smoothing * smoothing));

                                            self.simulation.velocity_x[idx] += factor * dx;
                                            self.simulation.velocity_y[idx] += factor * dy;

                                            let inner_radius = radius_cells * 0.8;
                                            if r > inner_radius {
                                                let damping_factor = ((r - inner_radius) / (radius_cells - inner_radius)).powi(2);
                                                let damping_coeff = 1.0 - damping_factor * 0.2;

                                                self.simulation.velocity_x[idx] *= damping_coeff;
                                                self.simulation.velocity_y[idx] *= damping_coeff;
                                            }
                                        }
                                    }
                                }
                            }
                        } else if response.drag_stopped() || !response.hovered() {
                            self.attractor_pos = None;
                            self.attractor_grid_pos = None;
                        }
                    }
                },
                Tool::Eraser => {
                    // Eraser tool: Remove persistent elements within radius (no placement mode)
                    if response.clicked() || response.dragged() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            self.eraser_pos = Some(pos);

                            let erase_x = ((pos.x - rect.left()) / cell_size) as f32;
                            let erase_y = ((pos.y - rect.top()) / cell_size) as f32;
                            let erase_radius = self.eraser_radius / cell_size;

                            // Remove elements within eraser radius
                            self.persistent_elements.retain(|elem| {
                                let dx = elem.x - erase_x;
                                let dy = elem.y - erase_y;
                                let dist = (dx * dx + dy * dy).sqrt();
                                dist > erase_radius // Keep if outside eraser radius
                            });
                        }
                    } else if response.drag_stopped() || !response.hovered() {
                        self.eraser_pos = None;
                    }
                },
            }

            // Render simulation
            let painter = ui.painter();

            // Render persistent elements (draw first, under the fluid)
            for elem in &self.persistent_elements {
                let screen_x = rect.left() + elem.x * cell_size;
                let screen_y = rect.top() + elem.y * cell_size;
                let pos = egui::Pos2::new(screen_x, screen_y);

                match elem.element_type {
                    PersistentElementType::DyeSource { color, .. } => {
                        // Render as filled circle with color
                        let color_u8 = egui::Color32::from_rgb(
                            (color.0 * 255.0) as u8,
                            (color.1 * 255.0) as u8,
                            (color.2 * 255.0) as u8,
                        );
                        painter.circle_filled(pos, elem.radius * cell_size, color_u8);
                        painter.circle_stroke(pos, elem.radius * cell_size,
                            egui::Stroke::new(1.0, egui::Color32::WHITE));
                    },
                    PersistentElementType::ForceSource { direction, .. } => {
                        // Render as arrow showing force direction
                        painter.circle_stroke(pos, elem.radius * cell_size,
                            egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 200, 255)));

                        // Draw arrow
                        let arrow_len = 15.0;
                        let dir_len = (direction.0 * direction.0 + direction.1 * direction.1).sqrt();
                        if dir_len > 0.01 {
                            let norm_x = direction.0 / dir_len;
                            let norm_y = direction.1 / dir_len;
                            let end_x = screen_x + norm_x * arrow_len;
                            let end_y = screen_y + norm_y * arrow_len;
                            painter.arrow(pos, egui::Vec2::new(norm_x * arrow_len, norm_y * arrow_len),
                                egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 200, 255)));
                        }
                    },
                    PersistentElementType::AttractorSource { .. } => {
                        // Render as circle with spiral pattern
                        painter.circle_stroke(pos, elem.radius * cell_size,
                            egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 200, 100)));
                        painter.circle_stroke(pos, elem.radius * cell_size * 0.2,
                            egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(255, 200, 100, 128)));
                        painter.circle_filled(pos, 3.0, egui::Color32::from_rgb(255, 200, 100));
                    },
                }
            }

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

            // Draw attractor radius circle
            if let Some(pos) = self.attractor_pos {
                painter.circle_stroke(
                    pos,
                    self.attractor_radius,
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 255, 255))
                );
                painter.circle_filled(pos, 3.0, egui::Color32::from_rgb(255, 255, 255));
            }

            // Draw eraser radius circle
            if let Some(pos) = self.eraser_pos {
                painter.circle_stroke(
                    pos,
                    self.eraser_radius,
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 100, 100))
                );
                painter.circle_filled(pos, 3.0, egui::Color32::from_rgb(255, 100, 100));
            }

            // Update simulation if not paused
            // Run 1 step per frame at all resolutions
            if !self.paused {
                // Apply all persistent elements
                for elem in &self.persistent_elements {
                    match elem.element_type {
                        PersistentElementType::DyeSource { color, intensity } => {
                            let x = elem.x.round() as usize;
                            let y = elem.y.round() as usize;
                            if x < self.simulation.width && y < self.simulation.height {
                                // Check if black (negative dye) is selected
                                let is_negative = color.0 == 0.0 && color.1 == 0.0 && color.2 == 0.0;

                                if is_negative {
                                    // Black removes dye - apply in a small area
                                    for dy in -2..=2 {
                                        for dx in -2..=2 {
                                            let px = (x as i32 + dx) as usize;
                                            let py = (y as i32 + dy) as usize;

                                            if px < self.simulation.width && py < self.simulation.height {
                                                let dist_sq = (dx * dx + dy * dy) as f32;
                                                if dist_sq <= 4.0 {
                                                    let falloff = 1.0 - dist_sq / 4.0;
                                                    let remove_intensity = falloff * intensity * 0.3; // Scale down for persistent

                                                    let idx = py * self.simulation.width + px;
                                                    self.simulation.dye_r[idx] = (self.simulation.dye_r[idx] - remove_intensity).max(0.0);
                                                    self.simulation.dye_g[idx] = (self.simulation.dye_g[idx] - remove_intensity).max(0.0);
                                                    self.simulation.dye_b[idx] = (self.simulation.dye_b[idx] - remove_intensity).max(0.0);
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    // Normal colors add dye
                                    self.simulation.add_dye(x, y, (
                                        color.0 * intensity,
                                        color.1 * intensity,
                                        color.2 * intensity,
                                    ));
                                }
                            }
                        },
                        PersistentElementType::ForceSource { direction, intensity } => {
                            let x = elem.x.round() as usize;
                            let y = elem.y.round() as usize;
                            if x < self.simulation.width && y < self.simulation.height {
                                let force = glam::Vec2::new(
                                    direction.0 * intensity,
                                    direction.1 * intensity,
                                );
                                self.simulation.add_force(x, y, force, elem.radius);
                            }
                        },
                        PersistentElementType::AttractorSource { strength } => {
                            // Apply point sink attractor
                            let smoothing = 2.0;
                            let dead_zone = elem.radius * 0.2;

                            for y in 0..self.simulation.height {
                                for x in 0..self.simulation.width {
                                    let dx = x as f32 - elem.x;
                                    let dy = y as f32 - elem.y;
                                    let r_squared = dx * dx + dy * dy;
                                    let r = r_squared.sqrt();

                                    if r > dead_zone && r < elem.radius {
                                        let idx = y * self.simulation.width + x;

                                        let factor = -strength /
                                            (2.0 * std::f32::consts::PI * (r_squared + smoothing * smoothing));

                                        self.simulation.velocity_x[idx] += factor * dx;
                                        self.simulation.velocity_y[idx] += factor * dy;

                                        // Sponge layer
                                        let inner_radius = elem.radius * 0.8;
                                        if r > inner_radius {
                                            let damping_factor = ((r - inner_radius) / (elem.radius - inner_radius)).powi(2);
                                            let damping_coeff = 1.0 - damping_factor * 0.2;

                                            self.simulation.velocity_x[idx] *= damping_coeff;
                                            self.simulation.velocity_y[idx] *= damping_coeff;
                                        }
                                    }
                                }
                            }
                        },
                    }
                }

                self.simulation.step();
                self.frame_count += 1;
            }
        });

        // WASM: update URL hash if persistent elements changed
        #[cfg(target_arch = "wasm32")]
        {
            self.update_url_hash_if_needed();
        }

        ctx.request_repaint();
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Serialize, Deserialize, Debug)]
struct ShareState {
    v: u8,            // schema version
    w: u32,           // base width at encoding time
    h: u32,           // base height at encoding time
    e: Vec<ShareElem> // elements
}

#[cfg(target_arch = "wasm32")]
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "t")]
enum ShareElem {
    #[serde(rename = "d")]
    Dye { x: f32, y: f32, r: f32, c: [f32; 3], i: f32 },
    #[serde(rename = "f")]
    Force { x: f32, y: f32, r: f32, d: [f32; 2], i: f32 },
    #[serde(rename = "a")]
    Attr { x: f32, y: f32, r: f32, s: f32 },
}

#[cfg(target_arch = "wasm32")]
impl InteractiveApp {
    // Encode current persistent elements to a base64url string
    fn encode_share_state(&self) -> Option<String> {
        // Nothing to share
        if self.persistent_elements.is_empty() {
            return Some(String::from("s="));
        }

        let width = self.simulation.width as f32;
        let height = self.simulation.height as f32;
        let cell_size = 8.0_f32; // matches UI layout assumptions

        let mut elems: Vec<ShareElem> = Vec::with_capacity(self.persistent_elements.len());
        for elem in &self.persistent_elements {
            match elem.element_type {
                PersistentElementType::DyeSource { color, intensity } => {
                    elems.push(ShareElem::Dye {
                        x: (elem.x / width).clamp(0.0, 1.0),
                        y: (elem.y / height).clamp(0.0, 1.0),
                        r: (elem.radius / width).min(elem.radius / height),
                        c: [color.0, color.1, color.2],
                        i: intensity,
                    });
                }
                PersistentElementType::ForceSource { direction, intensity } => {
                    // Store direction in grid-cell units for portability
                    let dir_cells = [direction.0 as f32 / cell_size, direction.1 as f32 / cell_size];
                    elems.push(ShareElem::Force {
                        x: (elem.x / width).clamp(0.0, 1.0),
                        y: (elem.y / height).clamp(0.0, 1.0),
                        r: (elem.radius / width).min(elem.radius / height),
                        d: dir_cells,
                        i: intensity,
                    });
                }
                PersistentElementType::AttractorSource { strength } => {
                    elems.push(ShareElem::Attr {
                        x: (elem.x / width).clamp(0.0, 1.0),
                        y: (elem.y / height).clamp(0.0, 1.0),
                        r: (elem.radius / width).min(elem.radius / height),
                        s: strength,
                    });
                }
            }
        }

        let state = ShareState {
            v: 1,
            w: self.base_width as u32,
            h: self.base_height as u32,
            e: elems,
        };

        if let Ok(json) = serde_json::to_string(&state) {
            let b64 = URL_SAFE_NO_PAD.encode(json.as_bytes());
            Some(format!("s={}", b64))
        } else {
            None
        }
    }

    // Try to load share state from window.location.hash
    fn try_load_share_state_from_url(&mut self) {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return,
        };
        let location = window.location();
        let hash = location.hash().unwrap_or_default();
        // Expect forms: "#s=..." or "s=..."
        let trimmed = hash.strip_prefix('#').unwrap_or(hash.as_str());
        if trimmed.is_empty() {
            return;
        }
        // Find s= parameter (support multiple params)
        let mut b64 = None;
        for part in trimmed.split('&') {
            if let Some(val) = part.strip_prefix("s=") {
                if !val.is_empty() {
                    b64 = Some(val);
                    break;
                }
            }
        }
        let Some(b64val) = b64 else { return; };
        let data = match URL_SAFE_NO_PAD.decode(b64val) {
            Ok(d) => d,
            Err(_) => return,
        };
        let Ok(state) = serde_json::from_slice::<ShareState>(&data) else { return; };
        self.apply_share_state(state);
        log::info!("Applied share state from URL: {} elements", self.persistent_elements.len());
    }

    fn apply_share_state(&mut self, state: ShareState) {
        let width = self.simulation.width as f32;
        let height = self.simulation.height as f32;
        let cell_size = 8.0_f32;

        self.persistent_elements.clear();
        for se in state.e.into_iter() {
            match se {
                ShareElem::Dye { x, y, r, c, i } => {
                    self.persistent_elements.push(PersistentElement {
                        element_type: PersistentElementType::DyeSource { color: (c[0], c[1], c[2]), intensity: i },
                        x: (x * width).clamp(0.0, width - 1.0),
                        y: (y * height).clamp(0.0, height - 1.0),
                        radius: (r * width).max(1e-3),
                    });
                }
                ShareElem::Force { x, y, r, d, i } => {
                    // Convert direction from cells back to pixel delta to preserve current behavior
                    let dir_pixels = (d[0] * cell_size, d[1] * cell_size);
                    self.persistent_elements.push(PersistentElement {
                        element_type: PersistentElementType::ForceSource { direction: dir_pixels, intensity: i },
                        x: (x * width).clamp(0.0, width - 1.0),
                        y: (y * height).clamp(0.0, height - 1.0),
                        radius: (r * width).max(1e-3),
                    });
                }
                ShareElem::Attr { x, y, r, s } => {
                    self.persistent_elements.push(PersistentElement {
                        element_type: PersistentElementType::AttractorSource { strength: s },
                        x: (x * width).clamp(0.0, width - 1.0),
                        y: (y * height).clamp(0.0, height - 1.0),
                        radius: (r * width).max(1e-3),
                    });
                }
            }
        }
    }

    fn update_url_hash_if_needed(&mut self) {
        let Some(hash) = self.encode_share_state() else { return; };
        if self.last_share_hash.as_ref() == Some(&hash) {
            return;
        }
        if let Some(window) = web_sys::window() {
            // Avoid growing history: use replaceState with updated hash fragment
            if let Some(history) = window.history().ok() {
                let _ = history.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&format!("#{}", hash)));
            } else {
                let _ = window.location().set_hash(&hash);
            }
            self.last_share_hash = Some(hash);
        }
    }
}
