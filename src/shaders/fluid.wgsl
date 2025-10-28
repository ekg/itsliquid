// Fluid simulation compute shaders

struct SimulationParams {
    width: u32,
    height: u32,
    dt: f32,
    viscosity: f32,
    diffusion: f32,
}

@group(0) @binding(0)
var<uniform> params: SimulationParams;

@group(0) @binding(1)
var velocity_texture: texture_storage_2d<rgba32float, read_write>;

@group(0) @binding(2)
var dye_texture: texture_storage_2d<rgba32float, read_write>;

fn texel_coord_to_index(coord: vec2<u32>) -> u32 {
    return coord.y * params.width + coord.x;
}

fn index_to_texel_coord(index: u32) -> vec2<u32> {
    return vec2<u32>(index % params.width, index / params.width);
}

fn sample_velocity(coord: vec2<u32>) -> vec2<f32> {
    let texel = textureLoad(velocity_texture, coord, 0);
    return vec2<f32>(texel.x, texel.y);
}

fn sample_dye(coord: vec2<u32>) -> vec3<f32> {
    let texel = textureLoad(dye_texture, coord, 0);
    return vec3<f32>(texel.x, texel.y, texel.z);
}

fn set_velocity(coord: vec2<u32>, velocity: vec2<f32>) {
    textureStore(velocity_texture, coord, vec4<f32>(velocity.x, velocity.y, 0.0, 1.0));
}

fn set_dye(coord: vec2<u32>, dye: vec3<f32>) {
    textureStore(dye_texture, coord, vec4<f32>(dye.x, dye.y, dye.z, 1.0));
}

// Velocity advection
@compute @workgroup_size(8, 8)
fn advect(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if (global_id.x >= params.width || global_id.y >= params.height) {
        return;
    }
    
    let coord = vec2<u32>(global_id.x, global_id.y);
    let velocity = sample_velocity(coord);
    
    // Backtrace position
    let src_coord_f = vec2<f32>(coord) - velocity * params.dt;
    let src_coord = vec2<u32>(max(min(src_coord_f.x, f32(params.width - 1)), 0.0), 
                             max(min(src_coord_f.y, f32(params.height - 1)), 0.0));
    
    // Advect velocity
    let advected_velocity = sample_velocity(src_coord);
    set_velocity(coord, advected_velocity);
    
    // Advect dye
    let advected_dye = sample_dye(src_coord);
    set_dye(coord, advected_dye);
}

// Velocity diffusion
@compute @workgroup_size(8, 8)
fn diffuse(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if (global_id.x >= params.width || global_id.y >= params.height) {
        return;
    }
    
    let coord = vec2<u32>(global_id.x, global_id.y);
    let x = i32(coord.x);
    let y = i32(coord.y);
    
    if (x <= 0 || x >= i32(params.width - 1) || y <= 0 || y >= i32(params.height - 1)) {
        // Boundary condition
        set_velocity(coord, vec2<f32>(0.0));
        return;
    }
    
    let current_velocity = sample_velocity(coord);
    
    // Sample neighbors
    let left = sample_velocity(vec2<u32>(u32(x - 1), u32(y)));
    let right = sample_velocity(vec2<u32>(u32(x + 1), u32(y)));
    let up = sample_velocity(vec2<u32>(u32(x), u32(y - 1)));
    let down = sample_velocity(vec2<u32>(u32(x), u32(y + 1)));
    
    // Diffusion calculation
    let a = params.dt * params.viscosity * f32(params.width * params.height);
    let diffused_velocity = (current_velocity + a * (left + right + up + down)) / (1.0 + 4.0 * a);
    
    set_velocity(coord, diffused_velocity);
}

// Pressure projection
@compute @workgroup_size(8, 8)
fn project(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if (global_id.x >= params.width || global_id.y >= params.height) {
        return;
    }
    
    let coord = vec2<u32>(global_id.x, global_id.y);
    let x = i32(coord.x);
    let y = i32(coord.y);
    
    if (x <= 0 || x >= i32(params.width - 1) || y <= 0 || y >= i32(params.height - 1)) {
        // Boundary condition
        return;
    }
    
    let velocity = sample_velocity(coord);
    
    // Calculate divergence (simplified)
    let left = sample_velocity(vec2<u32>(u32(x - 1), u32(y)));
    let right = sample_velocity(vec2<u32>(u32(x + 1), u32(y)));
    let up = sample_velocity(vec2<u32>(u32(x), u32(y - 1)));
    let down = sample_velocity(vec2<u32>(u32(x), u32(y + 1)));
    
    let divergence = 0.5 * ((right.x - left.x) + (down.y - up.y));
    
    // Simple pressure correction
    let pressure_correction = divergence * 0.25;
    let corrected_velocity = velocity - vec2<f32>(pressure_correction, pressure_correction);
    
    set_velocity(coord, corrected_velocity);
}