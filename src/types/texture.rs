use std::collections::HashMap;
use log::{error, info};
use crate::managers::resource_handle::ResourceHandle;
use crate::utils::{handle::Handle, mut_handle::MutHandle};

pub struct Texture {
    texture: wgpu::Texture,
    view: Handle<wgpu::TextureView>,
    sampler: Handle<wgpu::Sampler>,

    size: wgpu::Extent3d,

    // All bind groups for this texture for all shaders
    //
    // As this shader could be used by multiple shaders, we need to keep track of all bind groups
    // for this texture
    bind_groups: HashMap<ResourceHandle, wgpu::BindGroupEntry<'static>>
}

impl Texture {
    pub fn get_texture_view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn get_texture_sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }

    pub fn get_texture_size(&self) -> wgpu::Extent3d {
        self.size
    }

    pub fn load_from_file<T: AsRef<std::path::Path>>(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: T,
    ) -> Self {
        info!("Loading texture from file: {:?}", path.as_ref());
        let img = image::open(path).unwrap().to_rgba8();
        let dimensions = img.dimensions();
        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            label: Some("Texture"),
            view_formats: &[],
        });

        queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // The actual pixel data
            img.as_raw(),
            // The layout of the texture
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            label: Some("Texture Sampler"),
            ..Default::default()
        });
        

        Self {
            texture,
            view: Handle::new(view),
            sampler: Handle::new(sampler),

            size,
            
            bind_groups: HashMap::new()
        }
    }

    pub fn create_depth_texture(device: &wgpu::Device, sc_desc: MutHandle<wgpu::SurfaceConfiguration>) -> Self {
        let sc_desc = sc_desc.get();

        let size = wgpu::Extent3d {
            width: sc_desc.width,
            height: sc_desc.height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            label: Some("Depth Texture"),
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
            label: Some("Depth Texture Sampler"),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Depth,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("Depth Texture Bind Group Layout")
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Depth Texture Bind Group"),
        });

        Self{
            texture,
            view: Handle::new(view),
            sampler: Handle::new(sampler),

            size,
            
            bind_groups: HashMap::new()
        }
    }

    // Resize a screen texture
    //
    // Can be used for depth/full screen effects
    //
    // It is assumed screen textures do not store any data
    // and are instead written to.
    pub fn resize_screen_texture(&mut self, device: &wgpu::Device, sc_desc: MutHandle<wgpu::SurfaceConfiguration>) {
        error!("Resizing screen texture");
        let sc_desc = sc_desc.get();

        self.size = wgpu::Extent3d {
            width: sc_desc.width,
            height: sc_desc.height,
            depth_or_array_layers: 1,
        };

        self.texture = device.create_texture(&wgpu::TextureDescriptor {
            size: self.size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            label: Some("Depth Texture"),
            view_formats: &[],
        });

        self.view = Handle::new(self.texture.create_view(&wgpu::TextureViewDescriptor::default()));
    }
}
