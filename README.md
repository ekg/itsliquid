# itsliquid

A real-time fluid simulation that runs in your browser. Paint with dye, push the fluid around, create vortexes, and watch physics do its thing.

**🌐 [Try it live!](https://hypervolu.me/~erik/itsliquid)**

![Fluid Simulation Demo](docs/demo.gif)

## What is this?

itsliquid simulates incompressible fluids using the Navier-Stokes equations. It's built in Rust, compiles to WebAssembly, and runs at 60fps in your browser. No installation needed.

You can:
- Paint colorful dye into the fluid
- Push and pull the fluid around with forces
- Create swirling attractors that trap and spin the dye
- Place persistent elements that keep pumping dye and forces
- Mix colors and watch them diffuse naturally
- Erase things when you want to start fresh

Everything is interactive and responds to your touch/mouse in real-time.

## Tools

- **🎨 Dye** - Click/drag to paint colored dye into the fluid
- **💨 Force** - Drag to push the fluid around
- **🔍 Eyedropper** - Sample colors from the simulation
- **🌀 Attractor** - Create swirling vortexes that pull dye inward
- **🗑 Eraser** - Remove persistent elements you've placed
- **📌 Pin Mode** - Toggle to place persistent sources
  - With dye: drag to paint a line of continuous dye sources
  - With force: drag to set direction, creates persistent force
  - With attractor: click to place permanent vortex

## Controls

- **Left click/tap + drag** - Use the selected tool
- **Color swatches** - Pick your dye color (black removes dye!)
- **Sliders** - Adjust intensity, radius, and strength
- **⏸ Pause/▶ Resume** - Freeze/unfreeze the simulation
- **🗑 Clear** - Reset everything to blank
- **1x/2x/4x/8x** - Change grid resolution

## Features

- **Perfect mass conservation** - Dye doesn't mysteriously vanish (<0.001% loss)
- **HDR rendering** - Reinhard tone mapping handles super bright dye concentrations
- **Persistent elements** - Place dye sources, forces, and attractors that run continuously
- **Real Navier-Stokes physics** - Advection, diffusion, pressure projection, the whole deal
- **Runs in your browser** - WebAssembly means native performance, no plugins
- **Touch-friendly** - Works great on phones and tablets

## Running locally

Want to hack on it or run the desktop version?

```bash
git clone https://github.com/ekg/itsliquid.git
cd itsliquid
cargo run --release
```

Build the web version:

```bash
# One-time setup
cargo install wasm-pack

# Build and deploy
./deploy-web-bust-cache.sh
```

## How it works

The simulation solves the incompressible Navier-Stokes equations:

1. **Advection** - Fluid carries dye and velocity along with it
2. **Diffusion** - Things spread out over time (viscosity)
3. **Pressure projection** - Makes the fluid incompressible (divergence-free velocity field)
4. **Boundary conditions** - Keeps everything contained at the edges

The dye is separate from the velocity field but gets carried along by it. RGB channels mean you get real color mixing.

## Project structure

```
src/
├── fluid_interactive.rs    # Main fluid solver with perfect mass conservation
├── desktop_interactive.rs  # Interactive GUI with all the tools
├── fluid_final.rs          # Optimized pressure solver
├── export.rs               # PNG export for screenshots
├── analysis.rs             # Metrics and debugging
└── lib.rs                  # Module exports and WASM entry point
```

## Performance

On a 100×100 grid:
- **50×50**: ~500 µs per timestep
- **100×100**: ~2 ms per timestep
- **200×200**: ~7 ms per timestep

Runs at 60fps on most devices. The adaptive pressure solver converges early when it can, saving ~30-40% compute on average.

## Testing

There's automated browser testing with Playwright:

```bash
./test-web.sh               # Headless Firefox
./test-web.sh --headed      # Watch it run
./test-web.sh --ui          # Interactive mode
```

Tests verify WASM loading, user interactions, console logs, and visual rendering.

## Technical details

- **Point sink attractors** - Uses `v = -σ/(2πr²) × direction` for realistic vortex behavior
- **Sponge layer** - Exponential damping near attractor boundaries traps dye without hard walls
- **Adaptive convergence** - Pressure solver exits early when residual is small
- **Mass renormalization** - After diffusion and advection, total dye mass is preserved
- **Cache busting** - Deployment script adds version timestamps to force browser updates

## Contributing

Pull requests welcome! This started as a learning project and grew into something fun.

## License

MIT - see LICENSE file

## Credits

- Fluid simulation based on Jos Stam's "[Real-Time Fluid Dynamics for Games](https://www.dgp.toronto.edu/public_user/stam/reality/Research/pdf/GDC03.pdf)"
- Built with [Rust](https://www.rust-lang.org/), [egui](https://github.com/emilk/egui), and [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen)
- Testing with [Playwright](https://playwright.dev/)
