use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use itsliquid::InteractiveFluid;

fn benchmark_step(c: &mut Criterion) {
    let mut group = c.benchmark_group("simulation_step");

    // Test different grid sizes
    for size in [50, 100, 200].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut sim = InteractiveFluid::new(size, size);

            // Add some initial state
            sim.add_dye(size/2, size/2, (1.0, 0.0, 0.0));
            sim.add_force(size/2, size/2, glam::Vec2::new(5.0, 0.0), 1.0);

            b.iter(|| {
                black_box(sim.step());
            });
        });
    }
    group.finish();
}

fn benchmark_full_scenario(c: &mut Criterion) {
    c.bench_function("full_100x100_20steps", |b| {
        b.iter(|| {
            let mut sim = InteractiveFluid::new(100, 100);

            // Add droplet with force
            sim.add_dye(50, 50, (5.0, 0.0, 0.0));
            sim.add_force(50, 50, glam::Vec2::new(10.0, 0.0), 1.0);

            // Run 20 steps
            for _ in 0..20 {
                black_box(sim.step());
            }
        });
    });
}

fn benchmark_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("operations");
    let size = 100;

    group.bench_function("diffuse_velocity", |b| {
        let mut sim = InteractiveFluid::new(size, size);
        sim.add_force(50, 50, glam::Vec2::new(10.0, 0.0), 1.0);

        b.iter(|| {
            // Just diffusion
            black_box(&mut sim);
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_step, benchmark_full_scenario, benchmark_operations);
criterion_main!(benches);
