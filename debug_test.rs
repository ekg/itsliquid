extern crate itsliquid;
use itsliquid::FluidSolver;

fn main() {
    let mut sim = FluidSolver::new(10, 10);
    
    // Add fluid and velocity
    sim.add_density(5, 5, 1.0);
    sim.add_velocity(5, 5, glam::Vec2::new(1.0, 0.0));
    
    println!("Initial state:");
    println!("Density at (5,5): {}", sim.density[5 * 10 + 5]);
    println!("Velocity at (5,5): ({}, {})", sim.velocity_x[5 * 10 + 5], sim.velocity_y[5 * 10 + 5]);
    
    // Take a few steps
    for i in 0..5 {
        sim.step();
        println!("\nAfter step {}:", i + 1);
        println!("Density at (5,5): {}", sim.density[5 * 10 + 5]);
        println!("Velocity at (5,5): ({}, {})", sim.velocity_x[5 * 10 + 5], sim.velocity_y[5 * 10 + 5]);
        
        // Check if fluid moved
        for x in 0..10 {
            for y in 0..10 {
                let idx = y * 10 + x;
                if sim.density[idx] > 0.1 {
                    println!("  Fluid at ({},{}): {}", x, y, sim.density[idx]);
                }
            }
        }
    }
}