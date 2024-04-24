use std::ops::Deref;
use log::{error, info};
use winit::raw_window_handle::{HasDisplayHandle, HasRawWindowHandle};
use crate::device_handle::DeviceHandle;
use crate::utils::{handle::Handle, mut_handle::MutHandle};
use crate::instance_handle::InstanceHandle;

pub struct SurfaceWrapper{
    // wgpu
    _surface: Handle<wgpu::Surface<'static>>,
    _surface_configuration: MutHandle<wgpu::SurfaceConfiguration>
}

impl SurfaceWrapper{
    pub fn new(surface: wgpu::Surface<'static>, instance: &InstanceHandle, device: &DeviceHandle, window: &winit::window::Window) -> Self{
        let adapter = instance.get_adapter();

        info!("Surface created");

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or_else(|| {
                error!("No sRGB format found. Falling back to first format.");
                surface_caps.formats[0]
            });

        let surface_configuration = MutHandle::new(wgpu::SurfaceConfiguration{
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            desired_maximum_frame_latency: 3,
            view_formats: vec![]
        });

        surface.configure(&device.get_device(), &surface_configuration.get());

        info!("Surface configured");

        let surface = Handle::new(surface);
        Self{
            _surface: surface,
            _surface_configuration: surface_configuration
        }
    }

    pub fn get_surface(&self) -> Handle<wgpu::Surface>{
        self._surface.clone()
    }

    pub fn get_configuration(&self) -> MutHandle<wgpu::SurfaceConfiguration>{
        self._surface_configuration.clone()
    }

    pub fn get_surface_extent(&self) -> wgpu::Extent3d{
        wgpu::Extent3d{
            width: self._surface_configuration.get().width,
            height: self._surface_configuration.get().height,
            depth_or_array_layers: 1
        }
    }

    pub fn resize_surface(&mut self, device: &wgpu::Device, size: winit::dpi::PhysicalSize<u32>){
        self._surface_configuration.get().width = size.width;
        self._surface_configuration.get().height = size.height;

        self._surface.configure(device, &self._surface_configuration.get());
    }
}