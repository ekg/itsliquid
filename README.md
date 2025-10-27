# Liquid - Interactive Fluid Simulation

A real-time fluid simulation engine built in Rust with interactive mouse controls and colorful dye visualization. Experience the beauty of fluid dynamics with intuitive controls and realistic physics.

![Fluid Simulation Demo](docs/demo.gif)

## Features

- **Real-time Navier-Stokes simulation** with proper pressure projection
- **Interactive mouse controls**:
  - **Left-click + drag**: Pull fluid in any direction
  - **Right-click**: Add colorful dye droplets
  - **Right-click + drag**: Create continuous dye streams
  - **Drag release**: Generate vortex effects
- **RGB dye system** with 6 vibrant colors (red, green, blue, yellow, magenta, cyan)
- **Dynamic resolution scaling** (planned feature)
- **Real-time visualization** with configurable cell size
- **Headless testing mode** for automated validation

## Quick Start

### Prerequisites

- Rust and Cargo (latest stable version)
- Git

### Installation

```bash
git clone https://github.com/ekg/liquid.git
cd liquid
cargo run
```

### Controls

- **Left Mouse Button**: Click and drag to pull fluid
- **Right Mouse Button**: Click to add dye, drag to create streams
- **Pause/Resume**: Use the button in the UI
- **Cell Size**: Adjust visualization scale with the slider
- **Dye Colors**: Select from 6 different colors

## Usage

### Interactive Mode (Default)

Run the interactive GUI application:

```bash
cargo run
```

### Headless Test Mode

Run automated tests and generate analysis:

```bash
cargo run -- test
```

This will:
- Run a 20-frame simulation
- Export PNG images of each frame
- Generate quantitative analysis of fluid behavior
- Print debug visualizations

## Project Structure

```
src/
├── fluid_interactive.rs    # Main interactive fluid simulation
├── desktop_interactive.rs  # GUI application with mouse controls
├── fluid_final.rs          # Optimized fluid solver
├── desktop.rs              # Basic desktop application
├── export.rs               # PNG export functionality
├── analysis.rs             # Quantitative analysis tools
├── render.rs               # Visualization utilities
└── lib.rs                  # Module exports
```

## Technical Details

### Physics Engine

The simulation implements the incompressible Navier-Stokes equations:
- **Advection**: Fluid movement through velocity field
- **Diffusion**: Viscosity effects
- **Pressure Projection**: Ensures incompressibility
- **Boundary Conditions**: Proper handling of simulation edges

### Dye System

- RGB channels for colorful visualization
- Separate diffusion and advection for dye
- Interactive injection with falloff patterns
- Real-time color mixing

## Development

### Building

```bash
cargo build
```

### Testing

```bash
# Run headless test with analysis
cargo run -- test

# Run unit tests
cargo test
```

### Code Style

This project follows standard Rust conventions. Please run:

```bash
cargo fmt
cargo clippy
```

## Planned Features

- [ ] Dynamic resolution scaling
- [ ] Performance optimizations
- [ ] Additional simulation parameters
- [ ] Export to video formats
- [ ] WebAssembly build target
- [ ] Multi-threaded simulation

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## License

This project is open source. See LICENSE file for details.

## Acknowledgments

- Based on Jos Stam's "Real-Time Fluid Dynamics for Games"
- Built with the amazing Rust ecosystem
- Uses egui for the user interface