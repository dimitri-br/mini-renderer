use log::{info, log};
use crate::utils::handle::Handle;
use crate::surface_wrapper::SurfaceWrapper;

pub struct InstanceHandle{
    _instance: Handle<wgpu::Instance>,
    _adapter: Handle<wgpu::Adapter>
}

impl InstanceHandle{
    pub fn new() -> Self{
        let backends = wgpu::util::backend_bits_from_env().unwrap_or_default();
        let dx12_shader_compiler = wgpu::util::dx12_shader_compiler_from_env().unwrap_or_default();
        let gles_minor_version = wgpu::util::gles_minor_version_from_env().unwrap_or_default();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor{
            backends,
            flags: wgpu::InstanceFlags::from_build_config().with_env(),
            dx12_shader_compiler,
            gles_minor_version
        });

        info!("Instance created");

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions{
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false
        })).unwrap();

        info!("Adapter created");

        Self{
            _instance: Handle::new(instance),
            _adapter: Handle::new(adapter)
        }
    }

    pub fn get_instance(&self) -> Handle<wgpu::Instance>{
        self._instance.clone()
    }

    pub fn get_adapter(&self) -> Handle<wgpu::Adapter>{
        self._adapter.clone()
    }
}