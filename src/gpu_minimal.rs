//! Minimal GPU fluid simulation proof-of-concept

use wgpu::{Device, Queue, Texture, TextureView};

pub struct MinimalGPUFluid {
    device: Device,
    queue: Queue,
    width: u32,
    height: u32,
    dye_texture: Texture,
    dye_view: TextureView,
}

impl MinimalGPUFluid {
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
                    label: Some("Minimal Fluid GPU"),
                    required_features: wgpu::Features::CLEAR_TEXTURE,
                    required_limits: wgpu::Limits::downlevel_defaults(),
                },
                None,
            )
            .await?;
        
        let dye_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Dye Texture"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        
        let dye_view = dye_texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        Ok(Self {
            device,
            queue,
            width,
            height,
            dye_texture,
            dye_view,
        })
    }
    
    pub fn step(&mut self) {
        // Simple step - just clear the texture for now
        let clear_color = wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
        
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Clear Encoder"),
        });
        
        encoder.clear_texture(
            &self.dye_texture,
            &wgpu::ImageSubresourceRange {
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
            },
        );
        
        self.queue.submit(std::iter::once(encoder.finish()));
    }
    
    pub fn gpu_add_dye(&mut self, x: u32, y: u32, color: (f32, f32, f32)) {
        println!("GPU: Adding dye at ({}, {}) with color {:?}", x, y, color);
    }
    
    pub fn get_dye_texture_view(&self) -> &TextureView {
        &self.dye_view
    }
    
    pub fn gpu_width(&self) -> u32 { self.width }
    pub fn gpu_height(&self) -> u32 { self.height }
}

impl crate::FluidSimulation for MinimalGPUFluid {
    fn step(&mut self) { self.step() }
    
    fn add_force(&mut self, _x: usize, _y: usize, _force: glam::Vec2) {
        // Not implemented yet
    }
    
    fn add_dye(&mut self, x: usize, y: usize, color: (f32, f32, f32)) {
        self.gpu_add_dye(x as u32, y as u32, color)
    }
    
    fn width(&self) -> usize { self.gpu_width() as usize }
    fn height(&self) -> usize { self.gpu_height() as usize }
}