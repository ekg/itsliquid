extern crate itsliquid;
use itsliquid::SimpleFluid;

fn main() {
    let mut sim = SimpleFluid::new(30, 10);
    
    // Add fluid in the middle with horizontal velocity
    for i in 0..5 {
        sim.add_density(10 + i, 5, 1.0);
        sim.add_velocity(10 + i, 5, glam::Vec2::new(2.0, 0.0));
    }
    
    println!("Initial state:");
    visualize(&sim);
    
    // Take a few steps
    for step in 0..10 {
        sim.step();
        println!("\nAfter step {}:", step + 1);
        visualize(&sim);
    }
}

fn visualize(sim: &SimpleFluid) {
    for y in 0..sim.height {
        for x in 0..sim.width {
            let idx = y * sim.width + x;
            let density = sim.density[idx];
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
        println!();
    }
}