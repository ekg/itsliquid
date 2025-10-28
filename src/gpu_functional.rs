//! Functional GPU fluid simulation with actual computation

use wgpu::{Device, Queue, Texture, TextureView, BindGroup, BindGroupLayout, ComputePipeline, Buffer};
use wgpu::util::DeviceExt;
use glam::Vec2;
use bytemuck::{Pod, Zeroable};
use std::num::NonZeroU64;
use tokio::sync::oneshot;

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
    dye_texture: Texture,
    dye_view: TextureView,
    
    // Compute pipelines
    advect_pipeline: ComputePipeline,
    diffuse_pipeline: ComputePipeline,
    
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
            dt: 0.1,
            viscosity: 0.001,
            diffusion: 0.0001,
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
        
        let dye_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Dye Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        
        let dye_view = dye_texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        // Create shader with actual fluid simulation
        let shader_source = r"
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
            
            fn sample_velocity(coord: vec2<u32>) -> vec2<f32> {
                let texel = textureLoad(velocity_texture, coord);
                return vec2<f32>(texel.x, texel.y);
            }
            
            fn sample_dye(coord: vec2<u32>) -> vec3<f32> {
                let texel = textureLoad(dye_texture, coord);
                return vec3<f32>(texel.x, texel.y, texel.z);
            }
            
            fn set_velocity(coord: vec2<u32>, velocity: vec2<f32>) {
                textureStore(velocity_texture, coord, vec4<f32>(velocity.x, velocity.y, 0.0, 1.0));
            }
            
            fn set_dye(coord: vec2<u32>, dye: vec3<f32>) {
                textureStore(dye_texture, coord, vec4<f32>(dye.x, dye.y, dye.z, 1.0));
            }
            
            // Helper functions for interpolation
            fn mix_vec2(a: vec2<f32>, b: vec2<f32>, t: f32) -> vec2<f32> {
                return a + (b - a) * t;
            }
            
            fn mix_vec3(a: vec3<f32>, b: vec3<f32>, t: f32) -> vec3<f32> {
                return a + (b - a) * t;
            }
            
            fn floor(x: f32) -> f32 {
                return f32(i32(x));
            }
            
            // Advanced diffusion shader with proper boundary handling
            @compute @workgroup_size(8, 8)
            fn diffuse(@builtin(global_invocation_id) global_id: vec3<u32>) {
                if (global_id.x >= params.width || global_id.y >= params.height) {
                    return;
                }
                
                let coord = vec2<u32>(global_id.x, global_id.y);
                let x = i32(coord.x);
                let y = i32(coord.y);
                
                // Boundary conditions
                if (x <= 0 || x >= i32(params.width - 1) || y <= 0 || y >= i32(params.height - 1)) {
                    // Set boundary velocity to zero
                    set_velocity(coord, vec2<f32>(0.0));
                    // Let dye diffuse naturally at boundaries
                    return;
                }
                
                // Sample current state
                let current_velocity = sample_velocity(coord);
                let current_dye = sample_dye(coord);
                
                // Sample neighbors with boundary checks
                let left = sample_velocity(vec2<u32>(u32(x - 1), u32(y)));
                let right = sample_velocity(vec2<u32>(u32(x + 1), u32(y)));
                let up = sample_velocity(vec2<u32>(u32(x), u32(y - 1)));
                let down = sample_velocity(vec2<u32>(u32(x), u32(y + 1)));
                
                // Velocity diffusion with proper scaling
                let a = params.dt * params.viscosity * f32(params.width * params.height);
                let diffused_velocity = (current_velocity + a * (left + right + up + down)) / (1.0 + 4.0 * a);
                
                // Dye diffusion
                let dye_left = sample_dye(vec2<u32>(u32(x - 1), u32(y)));
                let dye_right = sample_dye(vec2<u32>(u32(x + 1), u32(y)));
                let dye_up = sample_dye(vec2<u32>(u32(x), u32(y - 1)));
                let dye_down = sample_dye(vec2<u32>(u32(x), u32(y + 1)));
                
                let b = params.dt * params.diffusion * f32(params.width * params.height);
                let diffused_dye = (current_dye + b * (dye_left + dye_right + dye_up + dye_down)) / (1.0 + 4.0 * b);
                
                set_velocity(coord, diffused_velocity);
                set_dye(coord, diffused_dye);
            }
            
            // Advanced advection shader with bilinear interpolation
            @compute @workgroup_size(8, 8)
            fn advect(@builtin(global_invocation_id) global_id: vec3<u32>) {
                if (global_id.x >= params.width || global_id.y >= params.height) {
                    return;
                }
                
                let coord = vec2<u32>(global_id.x, global_id.y);
                let x = i32(coord.x);
                let y = i32(coord.y);
                
                // Skip boundaries for advection
                if (x <= 0 || x >= i32(params.width - 1) || y <= 0 || y >= i32(params.height - 1)) {
                    return;
                }
                
                let velocity = sample_velocity(coord);
                
                // Backtrace position with proper scaling
                let src_x = f32(x) - velocity.x * params.dt * f32(params.width);
                let src_y = f32(y) - velocity.y * params.dt * f32(params.height);
                
                // Clamp to valid range with border
                let clamped_x = max(0.5, min(src_x, f32(params.width - 1) - 0.5));
                let clamped_y = max(0.5, min(src_y, f32(params.height - 1) - 0.5));
                
                // Bilinear interpolation for better quality
                let x0 = u32(floor(clamped_x));
                let x1 = u32(min(f32(params.width - 1), f32(x0) + 1.0));
                let y0 = u32(floor(clamped_y));
                let y1 = u32(min(f32(params.height - 1), f32(y0) + 1.0));
                
                let tx = clamped_x - f32(x0);
                let ty = clamped_y - f32(y0);
                
                // Sample four surrounding texels
                let v00 = sample_velocity(vec2<u32>(x0, y0));
                let v10 = sample_velocity(vec2<u32>(x1, y0));
                let v01 = sample_velocity(vec2<u32>(x0, y1));
                let v11 = sample_velocity(vec2<u32>(x1, y1));
                
                let d00 = sample_dye(vec2<u32>(x0, y0));
                let d10 = sample_dye(vec2<u32>(x1, y0));
                let d01 = sample_dye(vec2<u32>(x0, y1));
                let d11 = sample_dye(vec2<u32>(x1, y1));
                
                // Bilinear interpolation
                let advected_velocity = mix_vec2(mix_vec2(v00, v10, tx), mix_vec2(v01, v11, tx), ty);
                let advected_dye = mix_vec3(mix_vec3(d00, d10, tx), mix_vec3(d01, d11, tx), ty);
                
                set_velocity(coord, advected_velocity);
                set_dye(coord, advected_dye);
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
                        min_binding_size: Some(NonZeroU64::new(std::mem::size_of::<SimulationParams>() as u64).unwrap()),
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
                    resource: wgpu::BindingResource::TextureView(&dye_view),
                },
            ],
        });
        
        // Create compute pipelines
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Fluid Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let diffuse_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Diffusion Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: "diffuse",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        });
        
        let advect_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Advection Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: "advect",
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
            dye_texture,
            dye_view,
            advect_pipeline,
            diffuse_pipeline,
            bind_group,
        })
    }
    
    pub fn step(&mut self) {
        // Run diffusion
        self.run_compute_pass(&self.diffuse_pipeline);
        
        // Run advection
        self.run_compute_pass(&self.advect_pipeline);
    }
    
    fn run_compute_pass(&self, pipeline: &ComputePipeline) {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
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
        // Create a staging buffer with the dye data
        let dye_data = vec![color.0, color.1, color.2, 1.0];
        
        let staging_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Dye Staging Buffer"),
            contents: bytemuck::cast_slice(&dye_data),
            usage: wgpu::BufferUsages::COPY_SRC,
        });
        
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Add Dye Encoder"),
        });
        
        // Align bytes per row to 256 bytes
        let bytes_per_pixel = 4 * std::mem::size_of::<f32>() as u32;
        let aligned_bytes_per_row = ((bytes_per_pixel + 255) / 256) * 256;
        
        encoder.copy_buffer_to_texture(
            wgpu::ImageCopyBuffer {
                buffer: &staging_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(aligned_bytes_per_row),
                    rows_per_image: Some(1),
                },
            },
            wgpu::ImageCopyTexture {
                texture: &self.dye_texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        
        self.queue.submit(std::iter::once(encoder.finish()));
    }
    
    pub fn gpu_add_force(&mut self, x: u32, y: u32, force: Vec2) {
        // Create a staging buffer with the force data
        let force_data = vec![force.x, force.y, 0.0, 1.0];
        
        let staging_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Force Staging Buffer"),
            contents: bytemuck::cast_slice(&force_data),
            usage: wgpu::BufferUsages::COPY_SRC,
        });
        
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Add Force Encoder"),
        });
        
        // Align bytes per row to 256 bytes
        let bytes_per_pixel = 4 * std::mem::size_of::<f32>() as u32;
        let aligned_bytes_per_row = ((bytes_per_pixel + 255) / 256) * 256;
        
        encoder.copy_buffer_to_texture(
            wgpu::ImageCopyBuffer {
                buffer: &staging_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(aligned_bytes_per_row),
                    rows_per_image: Some(1),
                },
            },
            wgpu::ImageCopyTexture {
                texture: &self.velocity_texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        
        self.queue.submit(std::iter::once(encoder.finish()));
    }
    
    pub fn gpu_width(&self) -> u32 { self.width }
    pub fn gpu_height(&self) -> u32 { self.height }
    
    pub fn get_dye_texture_view(&self) -> &TextureView {
        &self.dye_view
    }
    
    pub async fn read_dye_data(&self) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        let buffer_size = (self.width as u64 * self.height as u64 * 4 * std::mem::size_of::<f32>() as u64) as u64;
        
        let read_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Dye Read Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
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
                    bytes_per_row: Some(self.width * 4 * std::mem::size_of::<f32>() as u32),
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
        let dye_data: &[f32] = bytemuck::cast_slice(&data);
        
        Ok(dye_data.to_vec())
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
    
    fn width(&self) -> usize { self.gpu_width() as usize }
    fn height(&self) -> usize { self.gpu_height() as usize }
}