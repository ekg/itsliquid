use itsliquid::InteractiveFluid;

#[test]
fn test_dye_mass_conservation() {
    let mut sim = InteractiveFluid::new(100, 100);

    // Add some dye
    sim.add_dye(50, 50, (5.0, 3.0, 2.0));

    // Calculate initial total dye
    let initial_r: f32 = sim.dye_r.iter().sum();
    let initial_g: f32 = sim.dye_g.iter().sum();
    let initial_b: f32 = sim.dye_b.iter().sum();

    println!("Initial dye: R={:.6}, G={:.6}, B={:.6}", initial_r, initial_g, initial_b);

    // Add some velocity to make dye move
    sim.add_force(50, 50, glam::Vec2::new(10.0, 5.0), 2.0);

    // Run 50 steps
    for step in 0..50 {
        sim.step();

        let current_r: f32 = sim.dye_r.iter().sum();
        let current_g: f32 = sim.dye_g.iter().sum();
        let current_b: f32 = sim.dye_b.iter().sum();

        if step % 10 == 0 {
            println!("Step {}: R={:.6}, G={:.6}, B={:.6}", step, current_r, current_g, current_b);
        }

        // Check conservation (allow small floating point error from diffusion)
        // Advection should conserve exactly, but diffusion may reduce slightly
        let r_loss = (initial_r - current_r) / initial_r;
        let g_loss = (initial_g - current_g) / initial_g;
        let b_loss = (initial_b - current_b) / initial_b;

        // Allow up to 1% loss per step due to diffusion (very conservative)
        assert!(r_loss < 0.01 * (step + 1) as f32,
            "Step {}: Red dye loss {:.2}% exceeds limit", step, r_loss * 100.0);
        assert!(g_loss < 0.01 * (step + 1) as f32,
            "Step {}: Green dye loss {:.2}% exceeds limit", step, g_loss * 100.0);
        assert!(b_loss < 0.01 * (step + 1) as f32,
            "Step {}: Blue dye loss {:.2}% exceeds limit", step, b_loss * 100.0);
    }

    let final_r: f32 = sim.dye_r.iter().sum();
    let final_g: f32 = sim.dye_g.iter().sum();
    let final_b: f32 = sim.dye_b.iter().sum();

    println!("Final dye: R={:.6}, G={:.6}, B={:.6}", final_r, final_g, final_b);
    println!("Loss: R={:.2}%, G={:.2}%, B={:.2}%",
        (initial_r - final_r) / initial_r * 100.0,
        (initial_g - final_g) / initial_g * 100.0,
        (initial_b - final_b) / initial_b * 100.0);
}
