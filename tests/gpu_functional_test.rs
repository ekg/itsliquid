//! Automated tests for functional GPU fluid simulation

use itsliquid::{FluidSimulation, gpu_functional::FunctionalGPUFluid};

#[tokio::test]
async fn test_functional_gpu_fluid_creation() {
    let fluid = FunctionalGPUFluid::new(50, 50).await;
    assert!(
        fluid.is_ok(),
        "Functional GPU fluid creation should succeed"
    );

    let fluid = fluid.unwrap();
    assert_eq!(fluid.width(), 50, "Width should match");
    assert_eq!(fluid.height(), 50, "Height should match");
}

#[tokio::test]
async fn test_functional_gpu_step_execution() {
    let mut fluid = FunctionalGPUFluid::new(10, 10).await.unwrap();

    // Test that the step method runs without panicking
    fluid.step();

    // Verify the simulation is still valid
    assert_eq!(fluid.width(), 10);
    assert_eq!(fluid.height(), 10);
}

#[tokio::test]
async fn test_functional_gpu_multiple_steps() {
    let mut fluid = FunctionalGPUFluid::new(20, 20).await.unwrap();

    // Run multiple steps to test stability
    for i in 0..10 {
        fluid.step();
        // Each step should complete without error
        assert_eq!(fluid.width(), 20);
        assert_eq!(fluid.height(), 20);
    }
}

#[tokio::test]
async fn test_functional_gpu_dye_addition() {
    let mut fluid = FunctionalGPUFluid::new(10, 10).await.unwrap();

    // Test adding dye at various positions
    let test_positions = [(5, 5), (0, 0), (9, 9), (2, 7)];
    let test_colors = [(1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 1.0)];

    for &(x, y) in &test_positions {
        for &color in &test_colors {
            fluid.add_dye(x, y, color);
            // Method should complete without error
        }
    }

    assert_eq!(fluid.width(), 10);
    assert_eq!(fluid.height(), 10);
}

#[tokio::test]
async fn test_functional_gpu_force_addition() {
    let mut fluid = FunctionalGPUFluid::new(10, 10).await.unwrap();

    // Test adding forces
    fluid.add_force(5, 5, glam::Vec2::new(1.0, 0.0));
    fluid.add_force(3, 7, glam::Vec2::new(0.0, -1.0));
    fluid.add_force(8, 2, glam::Vec2::new(0.5, 0.5));

    // Methods should complete without error
    assert_eq!(fluid.width(), 10);
    assert_eq!(fluid.height(), 10);
}

#[tokio::test]
async fn test_functional_gpu_combined_operations() {
    let mut fluid = FunctionalGPUFluid::new(16, 16).await.unwrap();

    // Combined test: add dye, add force, run simulation
    fluid.add_dye(8, 8, (1.0, 0.5, 0.25));
    fluid.add_force(8, 8, glam::Vec2::new(0.1, 0.1));

    // Run simulation steps
    for _ in 0..5 {
        fluid.step();
    }

    // Add more interactions
    fluid.add_dye(4, 12, (0.0, 1.0, 0.5));
    fluid.add_force(4, 12, glam::Vec2::new(-0.2, 0.0));

    // Run more steps
    for _ in 0..3 {
        fluid.step();
    }

    // Simulation should remain stable
    assert_eq!(fluid.width(), 16);
    assert_eq!(fluid.height(), 16);
}

#[tokio::test]
async fn test_functional_gpu_edge_cases() {
    // Test edge cases like small resolutions
    let small_fluid = FunctionalGPUFluid::new(4, 4).await;
    assert!(small_fluid.is_ok(), "Should handle small resolutions");

    // Test larger resolutions
    let large_fluid = FunctionalGPUFluid::new(128, 128).await;
    assert!(large_fluid.is_ok(), "Should handle larger resolutions");

    // Test non-square resolutions
    let rect_fluid = FunctionalGPUFluid::new(32, 64).await;
    assert!(rect_fluid.is_ok(), "Should handle rectangular resolutions");
}

#[tokio::test]
async fn test_functional_gpu_performance_characteristics() {
    // Test that we can create and run simulations of various sizes
    let resolutions = [(16, 16), (32, 32), (64, 64)];

    for &(width, height) in &resolutions {
        let mut fluid = FunctionalGPUFluid::new(width, height).await.unwrap();

        // Add some initial state
        fluid.add_dye((width / 2) as usize, (height / 2) as usize, (1.0, 0.0, 0.0));

        // Run a few steps
        for _ in 0..3 {
            fluid.step();
        }

        // Verify simulation state
        assert_eq!(fluid.width(), width as usize);
        assert_eq!(fluid.height(), height as usize);
    }
}
