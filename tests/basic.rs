use itsliquid::{FluidFinal, InteractiveFluid};

#[test]
fn test_interactive_fluid_creation() {
    let fluid = InteractiveFluid::new(50, 50);
    assert_eq!(fluid.width, 50);
    assert_eq!(fluid.height, 50);
    assert_eq!(fluid.velocity_x.len(), 2500);
    assert_eq!(fluid.dye_r.len(), 2500);
}

#[test]
fn test_fluid_final_creation() {
    let fluid = FluidFinal::new(50, 50);
    assert_eq!(fluid.width, 50);
    assert_eq!(fluid.height, 50);
    assert_eq!(fluid.velocity_x.len(), 2500);
    assert_eq!(fluid.density.len(), 2500);
}

#[test]
fn test_dye_addition() {
    let mut fluid = InteractiveFluid::new(10, 10);
    fluid.add_dye(5, 5, (1.0, 0.5, 0.25));

    let idx = 5 * 10 + 5;
    assert!(fluid.dye_r[idx] > 0.0);
    assert!(fluid.dye_g[idx] > 0.0);
    assert!(fluid.dye_b[idx] > 0.0);
}

#[test]
fn test_fluid_step() {
    let mut fluid = InteractiveFluid::new(10, 10);
    fluid.add_dye(5, 5, (1.0, 0.0, 0.0));

    // Just verify that step runs without panicking
    fluid.step();

    // Basic sanity check - fluid should still be valid
    assert_eq!(fluid.width, 10);
    assert_eq!(fluid.height, 10);
}
