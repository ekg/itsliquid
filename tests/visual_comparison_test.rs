/// Visual comparison testing framework
/// This creates side-by-side tests of CPU vs GPU with actual PNG outputs
/// so we can SEE what's broken instead of guessing

use std::fs;
use std::path::Path;

// Test scenario: Add dye and force, run N steps, export frames
#[test]
fn test_cpu_visual_droplet_flow() {
    use itsliquid::InteractiveFluid;

    let mut sim = InteractiveFluid::new(100, 100);

    // Add small droplet with force (like user interaction)
    sim.add_dye(50, 50, (5.0, 0.0, 0.0));
    sim.add_dye(49, 50, (1.5, 0.0, 0.0));
    sim.add_dye(51, 50, (1.5, 0.0, 0.0));
    sim.add_dye(50, 49, (1.5, 0.0, 0.0));
    sim.add_dye(50, 51, (1.5, 0.0, 0.0));

    // Add rightward force
    sim.add_force(50, 50, glam::Vec2::new(20.0, 0.0), 1.0);

    // Create test output directory
    fs::create_dir_all("test_output/cpu").unwrap();

    // Export frames
    for frame in 0..=20 {
        export_cpu_frame(&sim, frame, "test_output/cpu");
        if frame < 20 {
            sim.step();
        }
    }

    println!("\n✅ CPU test frames exported to: test_output/cpu/");
    println!("   View frame_0000.png to frame_0020.png to see behavior");

    // Verify dye exists somewhere
    let mut total_dye = 0.0;
    for y in 0..100 {
        for x in 0..100 {
            let idx = y * sim.width + x;
            total_dye += sim.dye_r[idx] + sim.dye_g[idx] + sim.dye_b[idx];
        }
    }

    assert!(
        total_dye > 1.0,
        "CPU: Dye disappeared! Total dye: {}. Check test_output/cpu/ frames",
        total_dye
    );
}

#[cfg(feature = "gpu")]
#[tokio::test]
async fn test_gpu_visual_droplet_flow() {
    use itsliquid::gpu_functional::FunctionalGPUFluid;

    let mut sim = FunctionalGPUFluid::new(100, 100).await.unwrap();

    // Same scenario as CPU test
    sim.gpu_add_dye(50, 50, (5.0, 0.0, 0.0));
    sim.gpu_add_dye(49, 50, (1.5, 0.0, 0.0));
    sim.gpu_add_dye(51, 50, (1.5, 0.0, 0.0));
    sim.gpu_add_dye(50, 49, (1.5, 0.0, 0.0));
    sim.gpu_add_dye(50, 51, (1.5, 0.0, 0.0));

    // Add rightward force
    sim.gpu_add_force(50, 50, glam::Vec2::new(20.0, 0.0));

    // Create test output directory
    fs::create_dir_all("test_output/gpu").unwrap();

    // Export frames
    for frame in 0..=20 {
        export_gpu_frame(&sim, frame, "test_output/gpu").await.unwrap();
        if frame < 20 {
            sim.step();
        }
    }

    println!("\n✅ GPU test frames exported to: test_output/gpu/");
    println!("   Compare with CPU frames to see differences");

    // Verify dye exists
    let dye_data = sim.read_dye_data().await.unwrap();
    let mut total_dye = 0.0;
    for i in (0..dye_data.len()).step_by(4) {
        total_dye += dye_data[i] + dye_data[i + 1] + dye_data[i + 2];
    }

    assert!(
        total_dye > 1.0,
        "GPU: Dye disappeared! Total dye: {}. Check test_output/gpu/ frames",
        total_dye
    );
}

// Test with NO forces - pure diffusion
#[test]
fn test_cpu_pure_diffusion() {
    use itsliquid::InteractiveFluid;

    let mut sim = InteractiveFluid::new(100, 100);

    // Add concentrated droplet, NO force
    sim.add_dye(50, 50, (10.0, 0.0, 0.0));

    fs::create_dir_all("test_output/cpu_diffusion").unwrap();

    for frame in 0..=20 {
        export_cpu_frame(&sim, frame, "test_output/cpu_diffusion");
        if frame < 20 {
            sim.step();
        }
    }

    println!("\n✅ CPU diffusion test exported to: test_output/cpu_diffusion/");
    println!("   Check if dye spreads naturally");

    // Measure spread - dye should have moved to neighbors
    let center_idx = 50 * sim.width + 50;
    let right_idx = 50 * sim.width + 55;

    let center = sim.dye_r[center_idx];
    let neighbor = sim.dye_r[right_idx];

    println!("   Center dye: {}, Neighbor (5px away): {}", center, neighbor);

    if neighbor < 0.01 {
        println!("   ⚠️  WARNING: Diffusion may not be working!");
    }
}

#[cfg(feature = "gpu")]
#[tokio::test]
async fn test_gpu_pure_diffusion() {
    use itsliquid::gpu_functional::FunctionalGPUFluid;

    let mut sim = FunctionalGPUFluid::new(100, 100).await.unwrap();

    // Add concentrated droplet, NO force
    sim.gpu_add_dye(50, 50, (10.0, 0.0, 0.0));

    fs::create_dir_all("test_output/gpu_diffusion").unwrap();

    for frame in 0..=20 {
        export_gpu_frame(&sim, frame, "test_output/gpu_diffusion").await.unwrap();
        if frame < 20 {
            sim.step();
        }
    }

    println!("\n✅ GPU diffusion test exported to: test_output/gpu_diffusion/");
    println!("   Compare with CPU diffusion to see differences");
}

// Metrics test - quantify what's happening
#[test]
fn test_cpu_metrics() {
    use itsliquid::InteractiveFluid;

    let mut sim = InteractiveFluid::new(100, 100);

    // Add droplet with force
    sim.add_dye(50, 50, (5.0, 0.0, 0.0));
    sim.add_force(50, 50, glam::Vec2::new(10.0, 0.0), 1.0);

    println!("\n=== CPU Simulation Metrics ===");

    for frame in 0..=10 {
        // Measure total dye
        let mut total_dye = 0.0;
        for idx in 0..sim.dye_r.len() {
            total_dye += sim.dye_r[idx] + sim.dye_g[idx] + sim.dye_b[idx];
        }

        // Measure total velocity magnitude
        let mut total_vel = 0.0;
        for idx in 0..sim.velocity_x.len() {
            total_vel += (sim.velocity_x[idx].powi(2) + sim.velocity_y[idx].powi(2)).sqrt();
        }

        // Measure center of mass
        let mut com_x = 0.0;
        let mut com_y = 0.0;
        let mut mass = 0.0;
        for y in 0..sim.height {
            for x in 0..sim.width {
                let idx = y * sim.width + x;
                let dye = sim.dye_r[idx] + sim.dye_g[idx] + sim.dye_b[idx];
                com_x += x as f32 * dye;
                com_y += y as f32 * dye;
                mass += dye;
            }
        }
        if mass > 0.0 {
            com_x /= mass;
            com_y /= mass;
        }

        println!("Frame {}: Dye={:.2}, Vel={:.2}, COM=({:.1}, {:.1})",
                 frame, total_dye, total_vel, com_x, com_y);

        if frame < 10 {
            sim.step();
        }
    }

    println!("\nExpected behavior:");
    println!("  - Dye should remain > 1.0 (conservation)");
    println!("  - COM should move right (force was rightward)");
    println!("  - Velocity should decrease gradually (viscosity)");
}

fn export_cpu_frame(sim: &itsliquid::InteractiveFluid, frame: usize, dir: &str) {
    use image::{ImageBuffer, Rgba};

    let width = sim.width as u32;
    let height = sim.height as u32;
    let mut img = ImageBuffer::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            let r = (sim.dye_r[idx] * 255.0).clamp(0.0, 255.0) as u8;
            let g = (sim.dye_g[idx] * 255.0).clamp(0.0, 255.0) as u8;
            let b = (sim.dye_b[idx] * 255.0).clamp(0.0, 255.0) as u8;
            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }

    let path = format!("{}/frame_{:04}.png", dir, frame);
    img.save(&path).unwrap();
}

#[cfg(feature = "gpu")]
async fn export_gpu_frame(
    sim: &itsliquid::gpu_functional::FunctionalGPUFluid,
    frame: usize,
    dir: &str
) -> Result<(), Box<dyn std::error::Error>> {
    use image::{ImageBuffer, Rgba};

    let dye_data = sim.read_dye_data().await?;
    let width = sim.gpu_width();
    let height = sim.gpu_height();

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

    let path = format!("{}/frame_{:04}.png", dir, frame);
    img.save(&path)?;
    Ok(())
}
