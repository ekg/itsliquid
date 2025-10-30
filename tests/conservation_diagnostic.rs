use itsliquid::InteractiveFluid;

#[test]
fn test_conservation_diagnostic() {
    let mut sim = InteractiveFluid::new(100, 100);

    // Add some dye
    sim.add_dye(50, 50, (10.0, 0.0, 0.0));
    sim.add_force(50, 50, glam::Vec2::new(10.0, 5.0), 2.0);

    let initial_r: f32 = sim.dye_r.iter().sum();
    println!("Initial dye: R={:.12}", initial_r);

    // Manual step with diagnostics
    for step in 0..10 {
        // Before step
        let r_before: f32 = sim.dye_r.iter().sum();

        // Copy state
        sim.velocity_x_prev.copy_from_slice(&sim.velocity_x);
        sim.velocity_y_prev.copy_from_slice(&sim.velocity_y);
        sim.dye_r_prev.copy_from_slice(&sim.dye_r);
        sim.dye_g_prev.copy_from_slice(&sim.dye_g);
        sim.dye_b_prev.copy_from_slice(&sim.dye_b);

        let r_after_copy: f32 = sim.dye_r.iter().sum();
        println!("\nStep {}: After copy: R={:.12} (delta={:+.12e})",
            step, r_after_copy, r_after_copy - r_before);

        // Manual diffuse_velocity (affects velocity, not dye)
        // ... skipping for brevity

        // After velocity operations, check dye
        let r_after_velocity: f32 = sim.dye_r.iter().sum();
        println!("Step {}: After velocity ops: R={:.12} (delta={:+.12e})",
            step, r_after_velocity, r_after_velocity - r_after_copy);

        // Just call the full step
        sim.step();

        let r_after: f32 = sim.dye_r.iter().sum();
        let loss = r_before - r_after;
        let loss_pct = (loss / r_before) * 100.0;

        println!("Step {}: After full step: R={:.12}", step, r_after);
        println!("Step {}: Loss this step: {:.12} ({:.6e}%)", step, loss, loss_pct);

        if loss.abs() > 1e-6 {
            println!("WARNING: Significant loss detected!");
        }
    }

    let final_r: f32 = sim.dye_r.iter().sum();
    let total_loss = initial_r - final_r;
    let total_loss_pct = (total_loss / initial_r) * 100.0;

    println!("\n=== SUMMARY ===");
    println!("Initial: {:.12}", initial_r);
    println!("Final:   {:.12}", final_r);
    println!("Total loss: {:.12} ({:.6e}%)", total_loss, total_loss_pct);
}
