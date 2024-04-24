use std::collections::HashMap;
use crate::managers::resource_handle::ResourceHandle;
use crate::managers::resource_manager::ResourceType;

pub struct ShaderManager{
    shaders: HashMap<ResourceHandle, wgpu::ShaderModule>
}

impl ShaderManager{
    pub fn new() -> Self{
        Self{
            shaders: HashMap::new()
        }
    }

    pub fn create_shader(&mut self, device: &wgpu::Device, source: &str) -> ResourceHandle{
        let handle = ResourceHandle::new(
            ResourceType::Shader
        );
        
        let shader = device.create_shader_module(
            wgpu::ShaderModuleDescriptor{
                    label: Some(&handle.get_uuid().to_string()),
                    source: wgpu::ShaderSource::Wgsl(source.into())
            }
        );
        
        self.shaders.insert(handle.clone(), shader);
        handle
    }

    pub fn get_shader(&self, handle: &ResourceHandle) -> Option<&wgpu::ShaderModule>{
        self.shaders.get(handle)
    }
}

impl ShaderManager {
    pub(crate) fn get_all_shader_handles(&self) -> Vec<ResourceHandle> {
        self.shaders.keys().cloned().collect()
    }
}