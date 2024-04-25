use std::collections::HashMap;
use crate::managers::resource_handle::ResourceHandle;
use crate::managers::resource_manager::ResourceType;
use crate::types::shader::Shader;
use crate::utils::handle::Handle;
use crate::utils::shader_reflect::Binding;

pub struct ShaderManager{
    shaders: HashMap<ResourceHandle, Shader>,

    _device: Handle<wgpu::Device>
}

impl ShaderManager{
    pub fn new(device: Handle<wgpu::Device>) -> Self{
        Self{
            shaders: HashMap::new(),
            _device: device
        }
    }

    pub fn create_shader(&mut self, source: &str) -> ResourceHandle{
        let handle = ResourceHandle::new(
            ResourceType::Shader
        );
        
        let mut shader = Shader::new(source);

        shader.generate_bindings();
        
        self.shaders.insert(handle.clone(), shader);
        handle
    }

    pub fn get_shader(&self, handle: &ResourceHandle) -> Option<wgpu::ShaderModule>{
        Some(self.shaders.get(handle).unwrap().compile(handle.clone(), &self._device))
    }

    pub fn get_shader_bindings(&self, handle: &ResourceHandle) -> Option<HashMap<String, Binding>>{
        Some(self.shaders.get(handle).unwrap().get_bindings())
    }
}

impl ShaderManager {
    pub(crate) fn get_all_shader_handles(&self) -> Vec<ResourceHandle> {
        self.shaders.keys().cloned().collect()
    }
}