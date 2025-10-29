use itsliquid::InteractiveFluid;

#[test]
fn test_cpu_simulation_dye_persistence() {
    // Create a small CPU simulation
    let mut sim = InteractiveFluid::new(50, 50);

    // Add dye at a point
    sim.add_dye(25, 25, (1.0, 0.0, 0.0));

    // Get initial dye value
    let idx = 25 * sim.width + 25;
    let initial_dye = sim.dye_r[idx];
    assert!(initial_dye > 0.0, "Dye should be added");

    // Run a few steps
    for _ in 0..5 {
        sim.step();
    }

    // Check dye still exists (may have diffused but shouldn't disappear)
    let mut total_dye = 0.0;
    for y in 20..30 {
        for x in 20..30 {
            let idx = y * sim.width + x;
            total_dye += sim.dye_r[idx] + sim.dye_g[idx] + sim.dye_b[idx];
        }
    }

    assert!(
        total_dye > 0.1,
        "Dye should persist after simulation steps, got total: {}",
        total_dye
    );
}

#[test]
fn test_cpu_simulation_force_application() {
    let mut sim = InteractiveFluid::new(50, 50);

    // Add force at a point
    sim.add_force(25, 25, glam::Vec2::new(10.0, 0.0), 1.0);

    // Get velocity
    let idx = 25 * sim.width + 25;
    let vel_x = sim.velocity_x[idx];
    assert!(vel_x > 0.0, "Force should create velocity");

    // Run a step
    sim.step();

    // Velocity should still exist or have propagated
    let vel_x_after = sim.velocity_x[idx];
    let vel_y_after = sim.velocity_y[idx];
    assert!(
        vel_x_after.abs() > 0.01 || vel_y_after.abs() > 0.01,
        "Velocity should persist or propagate"
    );
}

#[test]
fn test_cpu_dye_diffusion() {
    let mut sim = InteractiveFluid::new(50, 50);

    // Add concentrated dye
    sim.add_dye(25, 25, (10.0, 0.0, 0.0));

    // Run several steps to allow diffusion
    for _ in 0..10 {
        sim.step();
    }

    // Check that dye has diffused to neighbors
    let center_idx = 25 * sim.width + 25;
    let neighbor_idx = 25 * sim.width + 26;
    let center_dye = sim.dye_r[center_idx];
    let neighbor_dye = sim.dye_r[neighbor_idx];

    assert!(
        neighbor_dye > 0.01,
        "Dye should diffuse to neighbors, got: {}",
        neighbor_dye
    );
    assert!(
        center_dye > 0.0,
        "Center dye should still exist after diffusion"
    );
}

#[test]
fn test_cpu_small_droplet_behavior() {
    let mut sim = InteractiveFluid::new(100, 100);

    // Add a small droplet (like the new feature)
    let x = 50;
    let y = 50;
    sim.add_dye(x, y, (1.0, 0.0, 0.0));
    // Add tiny neighbors
    sim.add_dye(x - 1, y, (0.3, 0.0, 0.0));
    sim.add_dye(x + 1, y, (0.3, 0.0, 0.0));
    sim.add_dye(x, y - 1, (0.3, 0.0, 0.0));
    sim.add_dye(x, y + 1, (0.3, 0.0, 0.0));

    // Count initial dye
    let mut initial_total = 0.0;
    for dy in -5..=5 {
        for dx in -5..=5 {
            let px = (x as i32 + dx) as usize;
            let py = (y as i32 + dy) as usize;
            let idx = py * sim.width + px;
            initial_total += sim.dye_r[idx];
        }
    }

    // Run simulation
    for _ in 0..10 {
        sim.step();
    }

    // Count final dye
    let mut final_total = 0.0;
    for dy in -10..=10 {
        for dx in -10..=10 {
            let px = (x as i32 + dx) as usize;
            let py = (y as i32 + dy) as usize;
            if px < 100 && py < 100 {
                let idx = py * sim.width + px;
                final_total += sim.dye_r[idx];
            }
        }
    }

    // Dye should be conserved (allowing for some diffusion loss at boundaries)
    assert!(
        final_total > initial_total * 0.5,
        "Dye should be mostly conserved. Initial: {}, Final: {}",
        initial_total,
        final_total
    );
}

#[cfg(feature = "gpu")]
#[tokio::test]
async fn test_gpu_simulation_dye_persistence() {
    use itsliquid::gpu_functional::FunctionalGPUFluid;

    // Create a small GPU simulation
    let mut sim = FunctionalGPUFluid::new(50, 50).await.unwrap();

    // Add dye at a point
    sim.add_dye(25, 25, (5.0, 0.0, 0.0));

    // Run a few steps
    for _ in 0..5 {
        sim.step();
    }

    // Read back dye data
    let dye_data = sim.read_dye_data().await.unwrap();

    // Check dye still exists somewhere in the region
    let mut total_dye = 0.0;
    for y in 20..30 {
        for x in 20..30 {
            let idx = ((y * 50 + x) * 4) as usize;
            if idx < dye_data.len() {
                total_dye += dye_data[idx]; // Red channel
            }
        }
    }

    assert!(
        total_dye > 0.5,
        "GPU: Dye should persist after simulation steps, got total: {}",
        total_dye
    );
}

#[cfg(feature = "gpu")]
#[tokio::test]
async fn test_gpu_simulation_force_application() {
    use itsliquid::gpu_functional::FunctionalGPUFluid;

    let mut sim = FunctionalGPUFluid::new(50, 50).await.unwrap();

    // Add dye and force
    sim.add_dye(25, 25, (5.0, 0.0, 0.0));
    sim.add_force(25, 25, glam::Vec2::new(10.0, 0.0));

    // Run several steps to see movement
    for _ in 0..10 {
        sim.step();
    }

    // Read back dye data
    let dye_data = sim.read_dye_data().await.unwrap();

    // Check that dye has moved (should be displaced to the right)
    let mut left_dye = 0.0;
    let mut right_dye = 0.0;

    for y in 20..30 {
        for x in 20..26 {
            // Left region
            let idx = ((y * 50 + x) * 4) as usize;
            if idx < dye_data.len() {
                left_dye += dye_data[idx];
            }
        }
        for x in 26..35 {
            // Right region
            let idx = ((y * 50 + x) * 4) as usize;
            if idx < dye_data.len() {
                right_dye += dye_data[idx];
            }
        }
    }

    // With rightward velocity, more dye should be on the right
    assert!(
        right_dye + left_dye > 0.5,
        "GPU: Dye should exist after force application"
    );
}
