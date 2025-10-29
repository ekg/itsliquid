use itsliquid::{AnalysisRecorder, FluidFinal, FluidMetrics, ImageExporter};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "test" {
        // Run headless test and export PNGs
        run_headless_test()?;
    } else if args.len() > 1 && args[1] == "gpu-test" {
        // Run GPU test
        #[cfg(feature = "gpu")]
        run_gpu_test()?;

        #[cfg(not(feature = "gpu"))]
        {
            eprintln!("GPU feature not enabled. Build with --features gpu");
            std::process::exit(1);
        }
    } else {
        // Run GUI application
        run_gui_app();
    }

    Ok(())
}

fn run_headless_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("Running headless fluid simulation test with quantitative analysis...");

    // Use larger simulation for better visualization
    let mut simulation = FluidFinal::new(200, 200);
    let exporter = ImageExporter::new(800, 800);
    let mut recorder = AnalysisRecorder::new();

    // Add initial fluid as a horizontal line with velocity
    println!("Initializing simulation with horizontal fluid line...");
    for i in 0..40 {
        simulation.add_density(100 + i, 100, 1.0);
        simulation.add_velocity(100 + i, 100, glam::Vec2::new(3.0, 0.0));
    }

    // Record initial state
    recorder.record_frame(&simulation, 0);
    let initial_metrics = FluidMetrics::analyze(&simulation, 0);
    initial_metrics.print_summary();

    // Export initial state
    exporter.export_density_png(&simulation, Path::new("test_frame_0000.png"))?;
    exporter.export_velocity_png(&simulation, Path::new("test_velocity_0000.png"))?;

    // Run simulation and export frames
    for frame in 1..=20 {
        simulation.step();
        recorder.record_frame(&simulation, frame);

        let density_path = format!("test_frame_{:04}.png", frame);
        let velocity_path = format!("test_velocity_{:04}.png", frame);

        exporter.export_density_png(&simulation, Path::new(&density_path))?;
        exporter.export_velocity_png(&simulation, Path::new(&velocity_path))?;

        // Print metrics every 5 frames
        if frame % 5 == 0 {
            let metrics = FluidMetrics::analyze(&simulation, frame);
            metrics.print_summary();
        }

        // Debug: print simple density and velocity visualization for first few frames
        if frame <= 3 {
            println!("Frame {} density visualization:", frame);
            debug_visualize_density(&simulation);
            println!("Frame {} velocity visualization:", frame);
            debug_visualize_velocity(&simulation);
        }
    }

    // Print overall trends
    recorder.print_trends();

    println!("Test completed! Generated 21 frames with detailed analysis.");
    Ok(())
}

fn debug_visualize_density(simulation: &FluidFinal) {
    let width = simulation.width;
    let height = simulation.height;

    // Show a wider section to see horizontal movement
    for y in 95..105 {
        if y < height {
            for x in 80..120 {
                if x < width {
                    let idx = y * width + x;
                    let density = simulation.density[idx];
                    if density > 0.5 {
                        print!("██");
                    } else if density > 0.1 {
                        print!("▓▓");
                    } else if density > 0.01 {
                        print!("▒▒");
                    } else if density > 0.001 {
                        print!("░░");
                    } else {
                        print!("  ");
                    }
                }
            }
            println!();
        }
    }
    println!();
}

fn debug_visualize_velocity(simulation: &FluidFinal) {
    let width = simulation.width;
    let height = simulation.height;

    // Show velocity magnitude
    for y in 95..105 {
        if y < height {
            for x in 80..120 {
                if x < width {
                    let idx = y * width + x;
                    let vel_x = simulation.velocity_x[idx];
                    let vel_y = simulation.velocity_y[idx];
                    let vel_mag = (vel_x * vel_x + vel_y * vel_y).sqrt();

                    if vel_mag > 0.5 {
                        print!("→→");
                    } else if vel_mag > 0.1 {
                        print!("→");
                    } else if vel_mag > 0.01 {
                        print!(".");
                    } else {
                        print!("  ");
                    }
                }
            }
            println!();
        }
    }
    println!();
}

fn run_gui_app() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 800.0])
            .with_title("itsliquid - Interactive Fluid Simulation"),
        ..Default::default()
    };

    // Use GPU version if feature is enabled, otherwise use CPU version
    #[cfg(feature = "gpu")]
    {
        eframe::run_native(
            "itsliquid",
            options,
            Box::new(|_cc| Box::new(itsliquid::GPUInteractiveApp::new(100, 100))),
        )
        .unwrap();
    }

    #[cfg(not(feature = "gpu"))]
    {
        eframe::run_native(
            "itsliquid",
            options,
            Box::new(|_cc| Box::new(itsliquid::InteractiveApp::new(100, 100))),
        )
        .unwrap();
    }
}

#[cfg(feature = "gpu")]
fn run_gpu_test() -> Result<(), Box<dyn std::error::Error>> {
    use itsliquid::gpu_functional::FunctionalGPUFluid;
    use itsliquid::FluidSimulation;

    println!("Running GPU fluid simulation test...");

    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async {
        // Create GPU simulation
        let width = 200;
        let height = 200;
        let mut simulation = FunctionalGPUFluid::new(width, height).await?;

        println!("GPU simulation initialized: {}x{}", width, height);

        // Add initial fluid WITHOUT velocity to test if advection preserves dye
        println!("Adding initial dye (NO velocity)...");
        for i in 0..40 {
            simulation.add_dye(100 + i, 100, (1.0, 0.0, 0.0)); // Red dye
            // NO velocity - advection should preserve dye in place
        }

        // Export initial state
        export_gpu_frame(&simulation, 0).await?;

        // Run simulation and export frames
        for frame in 1..=20 {
            println!("Simulating frame {}...", frame);
            simulation.step();
            export_gpu_frame(&simulation, frame).await?;
        }

        println!("GPU test completed! Generated 21 frames.");
        Ok::<(), Box<dyn std::error::Error>>(())
    })?;

    Ok(())
}

#[cfg(feature = "gpu")]
async fn export_gpu_frame(
    simulation: &itsliquid::gpu_functional::FunctionalGPUFluid,
    frame: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    use image::{ImageBuffer, Rgba};

    // Read dye data from GPU
    let dye_data = simulation.read_dye_data().await?;
    let width = simulation.gpu_width();
    let height = simulation.gpu_height();

    // Create image from dye data
    let mut img = ImageBuffer::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            if idx + 3 < dye_data.len() {
                let r = (dye_data[idx] * 255.0).clamp(0.0, 255.0) as u8;
                let g = (dye_data[idx + 1] * 255.0).clamp(0.0, 255.0) as u8;
                let b = (dye_data[idx + 2] * 255.0).clamp(0.0, 255.0) as u8;
                img.put_pixel(x, y, Rgba([r, g, b, 255]));
            }
        }
    }

    let filename = format!("gpu_test_frame_{:04}.png", frame);
    img.save(&filename)?;
    println!("  Exported: {}", filename);

    Ok(())
}
