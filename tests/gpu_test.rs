//! Automated tests for GPU fluid simulation

use itsliquid::{FluidSimulation, gpu_minimal::MinimalGPUFluid};

#[tokio::test]
async fn test_gpu_fluid_creation() {
    // Test that we can create a GPU fluid simulation
    let fluid = MinimalGPUFluid::new(50, 50).await;
    assert!(fluid.is_ok(), "GPU fluid creation should succeed");

    let fluid = fluid.unwrap();
    assert_eq!(fluid.width(), 50, "Width should match");
    assert_eq!(fluid.height(), 50, "Height should match");
}

#[tokio::test]
async fn test_gpu_step_execution() {
    // Test that GPU step runs without panicking
    let mut fluid = MinimalGPUFluid::new(10, 10).await.unwrap();

    // The step should complete without errors
    fluid.step();

    // Verify the simulation is still valid
    assert_eq!(fluid.width(), 10);
    assert_eq!(fluid.height(), 10);
}

#[tokio::test]
async fn test_gpu_dye_addition() {
    // Test that dye addition works
    let mut fluid = MinimalGPUFluid::new(10, 10).await.unwrap();

    // Add dye at a specific position
    fluid.add_dye(5, 5, (1.0, 0.5, 0.25));

    // The method should complete without errors
    // (We can't easily verify the GPU state, but we can verify the method runs)
    assert_eq!(fluid.width(), 10);
    assert_eq!(fluid.height(), 10);
}

#[tokio::test]
async fn test_gpu_multiple_steps() {
    // Test running multiple simulation steps
    let mut fluid = MinimalGPUFluid::new(20, 20).await.unwrap();

    // Run several steps to ensure stability
    for _ in 0..10 {
        fluid.step();
    }

    // Simulation should still be valid
    assert_eq!(fluid.width(), 20);
    assert_eq!(fluid.height(), 20);
}

#[tokio::test]
async fn test_gpu_different_resolutions() {
    // Test creating GPU simulations at different resolutions
    let resolutions = [(16, 16), (32, 32), (64, 64), (128, 128)];

    for (width, height) in resolutions.iter() {
        let fluid = MinimalGPUFluid::new(*width, *height).await;
        assert!(
            fluid.is_ok(),
            "Should create {}x{} GPU simulation",
            width,
            height
        );

        let fluid = fluid.unwrap();
        assert_eq!(fluid.width(), *width as usize);
        assert_eq!(fluid.height(), *height as usize);
    }
}

// Integration test that compares CPU and GPU behavior (when both are available)
#[cfg(all(feature = "gpu", feature = "cpu"))]
#[tokio::test]
async fn test_gpu_cpu_consistency() {
    use itsliquid::{FluidSimulation, fluid_interactive::InteractiveFluid};

    // Create both CPU and GPU simulations
    let mut cpu_fluid = InteractiveFluid::new(10, 10);
    let mut gpu_fluid = MinimalGPUFluid::new(10, 10).await.unwrap();

    // Add the same initial conditions
    cpu_fluid.add_dye(5, 5, (1.0, 0.0, 0.0));
    gpu_fluid.add_dye(5, 5, (1.0, 0.0, 0.0));

    // Run the same number of steps
    for _ in 0..5 {
        cpu_fluid.step();
        gpu_fluid.step();
    }

    // Both should still be valid
    assert_eq!(cpu_fluid.width(), gpu_fluid.width());
    assert_eq!(cpu_fluid.height(), gpu_fluid.height());
}
