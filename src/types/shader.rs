use std::collections::HashMap;
use crate::managers::resource_handle::ResourceHandle;
use crate::utils::handle::Handle;
use crate::utils::shader_reflect::{Binding, BindingType, ShaderReflect};

pub struct Shader{
    source: String,
    binds: ShaderReflect,
    bind_group_layouts: HashMap<String, Handle<wgpu::BindGroupLayout>>,

    _device: Handle<wgpu::Device>
}

impl Shader{
    pub fn new<T: Into<String>>(device: Handle<wgpu::Device>, source: T) -> Self{
        let source = source.into();
        Self{
            source: source.clone(),
            binds: ShaderReflect::new(source),
            bind_group_layouts: HashMap::new(),
            _device: device
        }
    }

    pub fn generate_bindings(&mut self){
        self.binds.reflect();
    }

    pub fn get_bindings(&self) -> HashMap<String, Binding>{
        self.binds.get_bindings()
    }
    

    pub fn compile(&self, device: &wgpu::Device) -> wgpu::ShaderModule{
        device.create_shader_module(
            wgpu::ShaderModuleDescriptor{
                label: Some("Shader Module"),
                source: wgpu::ShaderSource::Wgsl(self.source.clone().into())
            }
        )
    }
}