use std::collections::HashMap;
use log::info;
use crate::utils::handle::Handle;
use crate::utils::shader_reflect::{Binding, BindingType, ShaderReflect};

pub struct Shader{
    source: String,
    binds: ShaderReflect,
    // group name, bind group layout
    bind_group_layouts: HashMap<u32, Handle<wgpu::BindGroupLayout>>,

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

        // Once we've reflected the shader, we can generate the bind group layouts

        // Firstly, we need to get the groups. Once we have the groups,
        // we can generate the entries for each group, to generate the layout
        let mut group_entries: HashMap<u32, Vec<wgpu::BindGroupLayoutEntry>> = HashMap::new();
        for (_, binding) in self.binds.get_bindings(){
            let group = binding.get_group();
            let bind = binding.get_binding();

            let entry = match binding.get_binding_type(){
                BindingType::Texture => {
                    wgpu::BindGroupLayoutEntry{
                        binding: bind,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false
                        },
                        count: None
                    }
                },
                BindingType::TextureSampler => {
                    wgpu::BindGroupLayoutEntry{
                        binding: bind,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(
                            wgpu::SamplerBindingType::Filtering
                        ),
                        count: None
                    }
                },
                BindingType::Uniform => {
                    wgpu::BindGroupLayoutEntry{
                        binding: bind,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None
                        },
                        count: None
                    }
                },
                BindingType::Storage => {
                    wgpu::BindGroupLayoutEntry{
                        binding: bind,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None
                        },
                        count: None
                    }
                }
            };

            let entries = group_entries.entry(group).or_insert_with(Vec::new);
            entries.push(entry);
        }

        // Sort the entries by binding
        for (_, entries) in group_entries.iter_mut(){
            entries.sort_by(|a, b| a.binding.cmp(&b.binding));
        }

        // Now, based on the entries per group, create the bind group layouts
        for (group, entries) in group_entries{
            let layout = self._device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor{
                    entries: &entries,
                    label: Some("Bind Group Layout")
                }
            );

            println!("Created bind group layout for group {}", group);
            println!("{:?}", entries);

            self.bind_group_layouts.insert(group, Handle::new(layout));
        }
    }

    pub fn get_bindings(&self) -> HashMap<String, Binding>{
        self.binds.get_bindings()
    }
    
    pub fn get_binding_by_name(&self, name: &str) -> Option<u32>{
        // Search each binding for the name. If we find it, return the group and binding
        for (_, binding) in self.binds.get_bindings(){
            if binding.get_name() == name{
                return Some(binding.get_binding());
            }
        }
        
        None
    }

    pub fn get_bind_group_layout(&self, group: u32) -> Option<&Handle<wgpu::BindGroupLayout>>{
        self.bind_group_layouts.get(&group)
    }

    pub fn get_bind_group_layouts(&self) -> Vec<Handle<wgpu::BindGroupLayout>>{
        let mut layouts: Vec<(u32, Handle<wgpu::BindGroupLayout>)> = Vec::new();

        for (id, layout) in self.bind_group_layouts.iter(){
            layouts.push((*id, layout.clone()));
        }

        // Sort the layouts by group id, and store just the layouts
        // This means we can guarantee the order of the layouts
        // from the group id with the lowest value to the highest
        layouts.sort_by(|a, b| a.0.cmp(&b.0));

        layouts.iter().map(|(_, layout)| layout.clone()).collect()
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