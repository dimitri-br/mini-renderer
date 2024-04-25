use std::collections::HashMap;
use crate::managers::resource_handle::ResourceHandle;
use crate::utils::shader_reflect::{Binding, ShaderReflect};

pub struct Shader{
    source: String,
    binds: ShaderReflect,
    compiled: Option<wgpu::ShaderModule>,
}

impl Shader{
    pub fn new<T: Into<String>>(source: T) -> Self{
        let source = source.into();
        Self{
            source: source.clone(),
            binds: ShaderReflect::new(source),
            compiled: None
        }
    }

    pub fn generate_bindings(&mut self){
        self.binds.reflect();
    }

    pub fn get_bindings(&self) -> HashMap<String, Binding>{
        self.binds.get_bindings()
    }

    pub fn compile(&self, handle: ResourceHandle, device: &wgpu::Device) -> wgpu::ShaderModule{
        device.create_shader_module(
            wgpu::ShaderModuleDescriptor{
                label: Some(&handle.get_uuid().to_string()),
                source: wgpu::ShaderSource::Wgsl(self.source.clone().into())
            }
        )
    }
}