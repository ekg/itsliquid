//! Functional GPU fluid simulation with actual computation

use bytemuck::{Pod, Zeroable};
use glam::Vec2;
use std::num::NonZeroU64;
use tokio::sync::oneshot;
use wgpu::util::DeviceExt;
use wgpu::{
    BindGroup, BindGroupLayout, Buffer, ComputePipeline, Device, Queue, Texture, TextureView,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct SimulationParams {
    width: u32,
    height: u32,
    dt: f32,
    viscosity: f32,
    diffusion: f32,
    _padding: [u32; 2],
}

pub struct FunctionalGPUFluid {
    device: Device,
    queue: Queue,
    width: u32,
    height: u32,

    // Simulation parameters buffer
    params_buffer: Buffer,

    // Textures for simulation state
    velocity_texture: Texture,
    velocity_view: TextureView,
    velocity_prev_texture: Texture,
    velocity_prev_view: TextureView,
    dye_texture: Texture,
    dye_view: TextureView,
    dye_prev_texture: Texture,
    dye_prev_view: TextureView,

    // Compute pipelines
    diffuse_velocity_pipeline: ComputePipeline,
    diffuse_dye_pipeline: ComputePipeline,
    advect_velocity_pipeline: ComputePipeline,
    advect_dye_pipeline: ComputePipeline,
    set_velocity_boundaries_pipeline: ComputePipeline,
    set_dye_boundaries_pipeline: ComputePipeline,
    project_velocity_pipeline: ComputePipeline,
    copy_velocity_to_prev_pipeline: ComputePipeline,
    copy_dye_to_prev_pipeline: ComputePipeline,

    // Bind groups
    bind_group: BindGroup,
}

impl FunctionalGPUFluid {
    pub async fn new(width: u32, height: u32) -> Result<Self, Box<dyn std::error::Error>> {
        let instance = wgpu::Instance::default();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or("No GPU adapter found")?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Functional Fluid GPU"),
                    required_features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                    required_limits: wgpu::Limits::downlevel_defaults(),
                },
                None,
            )
            .await?;

        // Create simulation parameters buffer
        let params = SimulationParams {
            width,
            height,
            dt: 0.5,  // Moderate timestep for stable simulation
            viscosity: 0.0001,  // Low viscosity to preserve velocity
            diffusion: 0.000001,  // Very low diffusion to preserve dye
            _padding: [0, 0],
        };

        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Simulation Parameters"),
            contents: bytemuck::cast_slice(&[params]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create textures
        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let velocity_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Velocity Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let velocity_view = velocity_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let velocity_prev_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Velocity Prev Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let velocity_prev_view = velocity_prev_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let dye_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Dye Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let dye_view = dye_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let dye_prev_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Dye Prev Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let dye_prev_view = dye_prev_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Initialize all textures to zero
        let zero_data = vec![0.0f32; (width * height * 4) as usize];

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &velocity_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&zero_data),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(width * 4 * std::mem::size_of::<f32>() as u32),
                rows_per_image: Some(height),
            },
            texture_size,
        );

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &velocity_prev_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&zero_data),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(width * 4 * std::mem::size_of::<f32>() as u32),
                rows_per_image: Some(height),
            },
            texture_size,
        );

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &dye_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&zero_data),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(width * 4 * std::mem::size_of::<f32>() as u32),
                rows_per_image: Some(height),
            },
            texture_size,
        );

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &dye_prev_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&zero_data),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(width * 4 * std::mem::size_of::<f32>() as u32),
                rows_per_image: Some(height),
            },
            texture_size,
        );

        // Create complete fluid simulation shader matching CPU algorithm
        let shader_source = r"
            // Helper functions
            fn floor(x: f32) -> f32 {
                return f32(i32(x));
            }
            
            fn max(a: f32, b: f32) -> f32 {
                return select(b, a, a >= b);
            }
            
            fn min(a: f32, b: f32) -> f32 {
                return select(a, b, a <= b);
            }
            
            fn select(a: f32, b: f32, condition: bool) -> f32 {
                if (condition) {
                    return a;
                } else {
                    return b;
                }
            }
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
            var velocity_prev_texture: texture_storage_2d<rgba32float, read_write>;

            @group(0) @binding(3)
            var dye_texture: texture_storage_2d<rgba32float, read_write>;

            @group(0) @binding(4)
            var dye_prev_texture: texture_storage_2d<rgba32float, read_write>;
            
            fn sample_velocity(coord: vec2<u32>) -> vec2<f32> {
                let texel = textureLoad(velocity_texture, coord);
                return vec2<f32>(texel.x, texel.y);
            }
            
            fn sample_velocity_prev(coord: vec2<u32>) -> vec2<f32> {
                let texel = textureLoad(velocity_prev_texture, coord);
                return vec2<f32>(texel.x, texel.y);
            }
            
            fn sample_dye(coord: vec2<u32>) -> vec3<f32> {
                let texel = textureLoad(dye_texture, coord);
                return vec3<f32>(texel.x, texel.y, texel.z);
            }

            fn sample_dye_prev(coord: vec2<u32>) -> vec3<f32> {
                let texel = textureLoad(dye_prev_texture, coord);
                return vec3<f32>(texel.x, texel.y, texel.z);
            }

            fn set_velocity(coord: vec2<u32>, velocity: vec2<f32>) {
                textureStore(velocity_texture, coord, vec4<f32>(velocity.x, velocity.y, 0.0, 1.0));
            }

            fn set_dye(coord: vec2<u32>, dye: vec3<f32>) {
                textureStore(dye_texture, coord, vec4<f32>(dye.x, dye.y, dye.z, 1.0));
            }
            
            // Velocity diffusion matching CPU implementation
            @compute @workgroup_size(8, 8)
            fn diffuse_velocity(@builtin(global_invocation_id) global_id: vec3<u32>) {
                if (global_id.x >= params.width || global_id.y >= params.height) {
                    return;
                }
                
                let coord = vec2<u32>(global_id.x, global_id.y);
                let x = i32(coord.x);
                let y = i32(coord.y);
                
                // Skip boundaries (handled separately)
                if (x <= 0 || x >= i32(params.width - 1) || y <= 0 || y >= i32(params.height - 1)) {
                    return;
                }
                
                // Sample neighbors
                let left = sample_velocity_prev(vec2<u32>(u32(x - 1), u32(y)));
                let right = sample_velocity_prev(vec2<u32>(u32(x + 1), u32(y)));
                let up = sample_velocity_prev(vec2<u32>(u32(x), u32(y - 1)));
                let down = sample_velocity_prev(vec2<u32>(u32(x), u32(y + 1)));
                
                // Velocity diffusion with CPU scaling (no width*height factor)
                let a = params.dt * params.viscosity;
                let current = sample_velocity_prev(coord);
                let diffused = (current + a * (left + right + up + down)) / (1.0 + 4.0 * a);
                
                set_velocity(coord, diffused);
            }
            
            // Dye diffusion matching CPU implementation
            @compute @workgroup_size(8, 8)
            fn diffuse_dye(@builtin(global_invocation_id) global_id: vec3<u32>) {
                if (global_id.x >= params.width || global_id.y >= params.height) {
                    return;
                }

                let coord = vec2<u32>(global_id.x, global_id.y);
                let x = i32(coord.x);
                let y = i32(coord.y);

                // Skip boundaries (handled separately)
                if (x <= 0 || x >= i32(params.width - 1) || y <= 0 || y >= i32(params.height - 1)) {
                    return;
                }

                // Sample neighbors from PREVIOUS dye buffer
                let dye_left = sample_dye_prev(vec2<u32>(u32(x - 1), u32(y)));
                let dye_right = sample_dye_prev(vec2<u32>(u32(x + 1), u32(y)));
                let dye_up = sample_dye_prev(vec2<u32>(u32(x), u32(y - 1)));
                let dye_down = sample_dye_prev(vec2<u32>(u32(x), u32(y + 1)));

                // Dye diffusion with CPU scaling (no width*height factor)
                let b = params.dt * params.diffusion;
                let current = sample_dye_prev(coord);
                let diffused = (current + b * (dye_left + dye_right + dye_up + dye_down)) / (1.0 + 4.0 * b);

                set_dye(coord, diffused);
            }
            
            // Velocity advection using previous velocity field (like CPU)
            @compute @workgroup_size(8, 8)
            fn advect_velocity(@builtin(global_invocation_id) global_id: vec3<u32>) {
                if (global_id.x >= params.width || global_id.y >= params.height) {
                    return;
                }
                
                let coord = vec2<u32>(global_id.x, global_id.y);
                let x = i32(coord.x);
                let y = i32(coord.y);
                
                // Skip boundaries
                if (x <= 0 || x >= i32(params.width - 1) || y <= 0 || y >= i32(params.height - 1)) {
                    return;
                }
                
                // Sample previous velocity (like CPU version)
                let velocity_prev = sample_velocity_prev(coord);
                
                // Backtrace position matching CPU scaling (no width*height factor)
                let src_x = f32(x) - params.dt * velocity_prev.x;
                let src_y = f32(y) - params.dt * velocity_prev.y;
                
                // Clamp to valid range with border (same as CPU)
                let clamped_x = max(0.5, min(src_x, f32(params.width - 1) - 0.5));
                let clamped_y = max(0.5, min(src_y, f32(params.height - 1) - 0.5));
                
                // Bilinear interpolation matching CPU
                let x0 = u32(floor(clamped_x));
                let x1 = u32(min(f32(params.width - 1), f32(x0) + 1.0));
                let y0 = u32(floor(clamped_y));
                let y1 = u32(min(f32(params.height - 1), f32(y0) + 1.0));
                
                let tx = clamped_x - f32(x0);
                let ty = clamped_y - f32(y0);
                
                // Advect velocity using previous velocity field (like CPU)
                let v00 = sample_velocity_prev(vec2<u32>(x0, y0));
                let v01 = sample_velocity_prev(vec2<u32>(x1, y0));
                let v10 = sample_velocity_prev(vec2<u32>(x0, y1));
                let v11 = sample_velocity_prev(vec2<u32>(x1, y1));
                
                let advected_velocity = (1.0 - tx) * (1.0 - ty) * v00
                    + tx * (1.0 - ty) * v01
                    + (1.0 - tx) * ty * v10
                    + tx * ty * v11;
                
                set_velocity(coord, advected_velocity);
            }
            
            // Dye advection using current velocity field (like CPU)
            @compute @workgroup_size(8, 8)
            fn advect_dye(@builtin(global_invocation_id) global_id: vec3<u32>) {
                if (global_id.x >= params.width || global_id.y >= params.height) {
                    return;
                }

                let coord = vec2<u32>(global_id.x, global_id.y);
                let x = f32(global_id.x);
                let y = f32(global_id.y);

                // Get velocity at current position
                let vel = sample_velocity(coord);

                // Backtrace to find source position
                let src_x = x - params.dt * vel.x;
                let src_y = y - params.dt * vel.y;

                // Clamp to valid range
                let clamped_x = max(0.0, min(src_x, f32(params.width - 1)));
                let clamped_y = max(0.0, min(src_y, f32(params.height - 1)));

                // Get integer coordinates for bilinear interpolation
                let ix0 = u32(clamped_x);
                let iy0 = u32(clamped_y);
                var ix1 = ix0 + 1u;
                if (ix1 >= params.width) {
                    ix1 = params.width - 1u;
                }
                var iy1 = iy0 + 1u;
                if (iy1 >= params.height) {
                    iy1 = params.height - 1u;
                }

                // Get fractional parts
                let fx = clamped_x - f32(ix0);
                let fy = clamped_y - f32(iy0);

                // Bilinear interpolation
                let d00 = sample_dye_prev(vec2<u32>(ix0, iy0));
                let d10 = sample_dye_prev(vec2<u32>(ix1, iy0));
                let d01 = sample_dye_prev(vec2<u32>(ix0, iy1));
                let d11 = sample_dye_prev(vec2<u32>(ix1, iy1));

                let d0 = d00 * (1.0 - fx) + d10 * fx;
                let d1 = d01 * (1.0 - fx) + d11 * fx;
                let result = d0 * (1.0 - fy) + d1 * fy;

                set_dye(coord, result);
            }
            
            // Boundary conditions for velocity
            @compute @workgroup_size(8, 8)
            fn set_velocity_boundaries(@builtin(global_invocation_id) global_id: vec3<u32>) {
                if (global_id.x >= params.width || global_id.y >= params.height) {
                    return;
                }
                
                let coord = vec2<u32>(global_id.x, global_id.y);
                let x = i32(coord.x);
                let y = i32(coord.y);
                
                // Set boundary velocity to zero (like CPU)
                if (x == 0 || x == i32(params.width - 1) || y == 0 || y == i32(params.height - 1)) {
                    set_velocity(coord, vec2<f32>(0.0));
                }
            }
            
            // Boundary conditions for dye - read from previous buffer to avoid race conditions
            @compute @workgroup_size(8, 8)
            fn set_dye_boundaries(@builtin(global_invocation_id) global_id: vec3<u32>) {
                if (global_id.x >= params.width || global_id.y >= params.height) {
                    return;
                }

                let coord = vec2<u32>(global_id.x, global_id.y);
                let x = i32(coord.x);
                let y = i32(coord.y);

                // Set dye boundaries - read from dye (current after diffusion/advection)
                if (x == 0) {
                    let right = sample_dye(vec2<u32>(1, u32(y)));
                    set_dye(coord, right);
                } else if (x == i32(params.width - 1)) {
                    let left = sample_dye(vec2<u32>(u32(params.width - 2), u32(y)));
                    set_dye(coord, left);
                } else if (y == 0) {
                    let down = sample_dye(vec2<u32>(u32(x), 1));
                    set_dye(coord, down);
                } else if (y == i32(params.height - 1)) {
                    let up = sample_dye(vec2<u32>(u32(x), u32(params.height - 2)));
                    set_dye(coord, up);
                }
            }
            
            // Simple velocity projection (basic divergence-free enforcement)
            @compute @workgroup_size(8, 8)
            fn project_velocity(@builtin(global_invocation_id) global_id: vec3<u32>) {
                if (global_id.x >= params.width || global_id.y >= params.height) {
                    return;
                }
                
                let coord = vec2<u32>(global_id.x, global_id.y);
                let x = i32(coord.x);
                let y = i32(coord.y);
                
                // Skip boundaries
                if (x <= 0 || x >= i32(params.width - 1) || y <= 0 || y >= i32(params.height - 1)) {
                    return;
                }
                
                let h = 1.0 / f32(params.width);
                
                // Calculate divergence (like CPU)
                let vel_left = sample_velocity(vec2<u32>(u32(x - 1), u32(y)));
                let vel_right = sample_velocity(vec2<u32>(u32(x + 1), u32(y)));
                let vel_up = sample_velocity(vec2<u32>(u32(x), u32(y - 1)));
                let vel_down = sample_velocity(vec2<u32>(u32(x), u32(y + 1)));
                
                let divergence = -0.5 * h * (vel_right.x - vel_left.x + vel_down.y - vel_up.y);
                
                // Simple pressure correction (single iteration for now)
                let pressure_correction = divergence * 0.25;
                
                // Apply pressure gradient correction
                let current_vel = sample_velocity(coord);
                let new_vel_x = current_vel.x - 0.5 * pressure_correction / h;
                let new_vel_y = current_vel.y - 0.5 * pressure_correction / h;
                
                set_velocity(coord, vec2<f32>(new_vel_x, new_vel_y));
            }
            
            // Copy velocity to velocity_prev (like CPU's copy_from_slice)
            @compute @workgroup_size(8, 8)
            fn copy_velocity_to_prev(@builtin(global_invocation_id) global_id: vec3<u32>) {
                if (global_id.x >= params.width || global_id.y >= params.height) {
                    return;
                }

                let coord = vec2<u32>(global_id.x, global_id.y);
                let velocity = sample_velocity(coord);
                textureStore(velocity_prev_texture, coord, vec4<f32>(velocity.x, velocity.y, 0.0, 1.0));
            }

            // Copy dye to dye_prev (for double buffering)
            @compute @workgroup_size(8, 8)
            fn copy_dye_to_prev(@builtin(global_invocation_id) global_id: vec3<u32>) {
                if (global_id.x >= params.width || global_id.y >= params.height) {
                    return;
                }

                let coord = vec2<u32>(global_id.x, global_id.y);
                let dye = sample_dye(coord);
                textureStore(dye_prev_texture, coord, vec4<f32>(dye.x, dye.y, dye.z, 1.0));
            }
        ";

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Functional Fluid Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Fluid Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(
                            NonZeroU64::new(std::mem::size_of::<SimulationParams>() as u64)
                                .unwrap(),
                        ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadWrite,
                        format: wgpu::TextureFormat::Rgba32Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadWrite,
                        format: wgpu::TextureFormat::Rgba32Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadWrite,
                        format: wgpu::TextureFormat::Rgba32Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadWrite,
                        format: wgpu::TextureFormat::Rgba32Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Fluid Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&velocity_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&velocity_prev_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&dye_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&dye_prev_view),
                },
            ],
        });

        // Create compute pipelines
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Fluid Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let diffuse_velocity_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Diffuse Velocity Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: "diffuse_velocity",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        });

        let diffuse_dye_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Diffuse Dye Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: "diffuse_dye",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        });

        let advect_velocity_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Advect Velocity Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: "advect_velocity",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        });

        let advect_dye_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Advect Dye Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: "advect_dye",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        });

        let set_velocity_boundaries_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Set Velocity Boundaries Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: "set_velocity_boundaries",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        });

        let set_dye_boundaries_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Set Dye Boundaries Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: "set_dye_boundaries",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        });

        let project_velocity_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Project Velocity Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: "project_velocity",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        });

        let copy_velocity_to_prev_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Copy Velocity to Prev Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: "copy_velocity_to_prev",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        });

        let copy_dye_to_prev_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Copy Dye to Prev Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: "copy_dye_to_prev",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        });

        Ok(Self {
            device,
            queue,
            width,
            height,
            params_buffer,
            velocity_texture,
            velocity_view,
            velocity_prev_texture,
            velocity_prev_view,
            dye_texture,
            dye_view,
            dye_prev_texture,
            dye_prev_view,
            diffuse_velocity_pipeline,
            diffuse_dye_pipeline,
            advect_velocity_pipeline,
            advect_dye_pipeline,
            set_velocity_boundaries_pipeline,
            set_dye_boundaries_pipeline,
            project_velocity_pipeline,
            copy_velocity_to_prev_pipeline,
            copy_dye_to_prev_pipeline,
            bind_group,
        })
    }

    pub fn step(&mut self) {
        // Test: ONLY copy, no advection at all
        // This will test if copy_dye_to_prev actually works
        self.run_compute_pass(&self.copy_dye_to_prev_pipeline);
        self.device.poll(wgpu::Maintain::Wait);

        // Don't call advection - just leave dye as-is
        // Dye should persist because we're not modifying it
    }

    fn run_compute_pass(&self, pipeline: &ComputePipeline) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Fluid Compute Encoder"),
            });

        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Fluid Compute Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(pipeline);
        compute_pass.set_bind_group(0, &self.bind_group, &[]);

        let workgroup_size = 8;
        let workgroup_count_x = (self.width + workgroup_size - 1) / workgroup_size;
        let workgroup_count_y = (self.height + workgroup_size - 1) / workgroup_size;

        compute_pass.dispatch_workgroups(workgroup_count_x, workgroup_count_y, 1);

        drop(compute_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn gpu_add_dye(&mut self, x: u32, y: u32, color: (f32, f32, f32)) {
        // Write directly to the texture using queue.write_texture instead of buffer copy
        let dye_data = vec![color.0, color.1, color.2, 1.0];

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.dye_texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&dye_data),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * std::mem::size_of::<f32>() as u32),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        // Ensure GPU operations complete
        self.device.poll(wgpu::Maintain::Wait);
    }

    pub fn gpu_add_force(&mut self, x: u32, y: u32, force: Vec2) {
        // Write directly to the texture using queue.write_texture
        let force_data = vec![force.x, force.y, 0.0, 1.0];

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.velocity_texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&force_data),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * std::mem::size_of::<f32>() as u32),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        // Ensure GPU operations complete
        self.device.poll(wgpu::Maintain::Wait);
    }

    pub fn gpu_width(&self) -> u32 {
        self.width
    }
    pub fn gpu_height(&self) -> u32 {
        self.height
    }

    pub fn get_dye_texture_view(&self) -> &TextureView {
        &self.dye_view
    }

    pub async fn read_dye_data(&self) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        let bytes_per_pixel = 4 * std::mem::size_of::<f32>();
        let bytes_per_row_unpadded = self.width as u64 * bytes_per_pixel as u64;
        
        // Align bytes per row to 256 bytes (WGSL requirement)
        let align = 256;
        let bytes_per_row = ((bytes_per_row_unpadded + align - 1) / align) * align;
        
        let buffer_size = bytes_per_row * self.height as u64;

        let read_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Dye Read Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Read Dye Encoder"),
            });

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &self.dye_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &read_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row as u32),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        let buffer_slice = read_buffer.slice(..);
        let (sender, receiver) = oneshot::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });

        self.device.poll(wgpu::Maintain::Wait);

        receiver.await??;

        let data = buffer_slice.get_mapped_range();
        let all_data: &[f32] = bytemuck::cast_slice(&data);
        
        // Extract actual data skipping padding
        let mut dye_data = Vec::with_capacity((self.width * self.height * 4) as usize);
        let pixels_per_row = self.width as usize;
        let floats_per_pixel = 4;
        let floats_per_row_unpadded = pixels_per_row * floats_per_pixel;
        let floats_per_row_padded = (bytes_per_row as usize) / std::mem::size_of::<f32>();
        
        for row in 0..self.height as usize {
            let row_start = row * floats_per_row_padded;
            let row_end = row_start + floats_per_row_unpadded;
            
            if row_end <= all_data.len() {
                dye_data.extend_from_slice(&all_data[row_start..row_end]);
            }
        }

        Ok(dye_data)
    }
}

impl crate::FluidSimulation for FunctionalGPUFluid {
    fn step(&mut self) {
        self.step()
    }

    fn add_force(&mut self, x: usize, y: usize, force: glam::Vec2) {
        self.gpu_add_force(x as u32, y as u32, force)
    }

    fn add_dye(&mut self, x: usize, y: usize, color: (f32, f32, f32)) {
        self.gpu_add_dye(x as u32, y as u32, color)
    }

    fn width(&self) -> usize {
        self.gpu_width() as usize
    }
    fn height(&self) -> usize {
        self.gpu_height() as usize
    }
}
