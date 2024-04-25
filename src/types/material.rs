use std::collections::HashMap;
use log::error;
use crate::utils::handle::Handle;
use crate::managers::resource_handle::ResourceHandle;
use crate::managers::resource_manager::{ResourceManager, ResourceType};
use crate::types::texture::Texture;
use crate::utils::shader_reflect::{Binding, BindingType};

pub struct Material{
    // Textures
    textures: HashMap<String, ResourceHandle>,
    // Uniforms
    uniforms: HashMap<String, ResourceHandle>,

    // Bind group layouts and bind groups
    // Users of the bind groups will get a handle
    // to the group they want to use, so we always
    // maintain a mapping of the bind group to the
    // handle here

    // Entries for the bind group layouts - the ID is the group ID,
    // then a list of entries for that group and the name of the
    // bind group layout entry
    bind_group_layouts_entries: HashMap<u32, Vec<(String, wgpu::BindGroupLayoutEntry)>>,
    // The bind group layouts themselves - the ID is the group ID
    // and the handle is the handle to the bind group layout
    bind_group_layouts: HashMap<u32, Handle<wgpu::BindGroupLayout>>,

    // Bind groups - the ID is the group ID, and the handle is the
    // handle to the bind group

    // Entries are separate, and are generated from the bind group layouts
    // closer to the time of rendering
    bind_groups: HashMap<u32, Handle<wgpu::BindGroup>>,



    // Shader
    shader_handle: Option<ResourceHandle>,   // Handle to the shader used by this material
    shader_bindings: Option<HashMap<String, Binding>>,

    // Acceptable pipelines
    pipelines: Vec<ResourceHandle>,

    // A reference to the device
    _device: Handle<wgpu::Device>
}

impl Material{
    pub fn new(device: Handle<wgpu::Device>) -> Self{
        Self{
            textures: HashMap::new(),
            uniforms: HashMap::new(),


            bind_group_layouts_entries: HashMap::new(),
            bind_group_layouts: HashMap::new(),
            bind_groups: HashMap::new(),
            
            shader_handle: None, // Just a dummy handle for now
            shader_bindings: None, // we assign when we assign the shader
            pipelines: Vec::new(),

            _device: device
        }
    }

    pub fn add_texture(&mut self, name: &str, texture_handle: ResourceHandle, texture: Handle<Texture>){
        self.textures.insert(name.to_string(), texture_handle);

        // Generate the bind group layout entry

    }

    pub fn get_texture(&self, name: &str) -> Option<&ResourceHandle>{
        self.textures.get(name)
    }

    pub fn set_shader(&mut self, shader: ResourceHandle, bindings: HashMap<String, Binding>){
        /* Now generate all the bind group layouts */
        let mut group_layouts = HashMap::new();

        for (name, binding) in bindings.iter(){
            let group = binding.get_group();

            let entry = wgpu::BindGroupLayoutEntry{
                binding: binding.get_binding(),
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT | wgpu::ShaderStages::COMPUTE,
                ty: match binding.get_binding_type(){
                    BindingType::Texture => wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false
                    },
                    BindingType::TextureSampler => wgpu::BindingType::Sampler(
                        wgpu::SamplerBindingType::Filtering
                    ),
                    BindingType::Uniform => wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    BindingType::Storage => wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None
                    }
                },
                count: None
            };

            let entries = group_layouts.entry(group).or_insert_with(Vec::new);
            entries.push((name.clone(), entry));
        }

        // For each group, create a bind group layout
        for (group, entries) in group_layouts.iter(){
            let layout = self._device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor{
                    entries: entries.iter().map(|(_, entry)| *entry).collect::<Vec<_>>().as_slice(),
                    label: Some("Material Bind Group Layout")
                }
            );

            self.bind_group_layouts.insert(*group, Handle::new(layout));
            self.bind_group_layouts_entries.insert(*group, entries.clone());
        }

        self.shader_handle = Some(shader);
        self.shader_bindings = Some(bindings);
    }


    pub fn get_shader(&self) -> ResourceHandle{
        self.shader_handle.as_ref().unwrap().clone()
    }

    pub(crate) fn get_bind_group_layouts<'a>(&'a self, resource_manager: &'a ResourceManager)
        -> Vec<Handle<wgpu::BindGroupLayout>>{
        let mut layouts = Vec::new();
        for texture in self.textures.values(){
            let texture = resource_manager.get_texture(texture).unwrap();
            layouts.push(texture.get_bind_group_layout());
        }

        layouts
    }

    pub(crate) fn get_bind_groups<'a>(&'a self, resource_manager: &'a ResourceManager)
        -> Vec<Handle<wgpu::BindGroup>>{
        let mut bind_groups = Vec::new();
        for texture in self.textures.values(){
            let texture = resource_manager.get_texture(texture).unwrap();
            bind_groups.push(texture.get_bind_group());
        }

        bind_groups
    }

    pub fn bind_material<'a>(&'a self, resource_manager: &'a ResourceManager, render_pass: &mut wgpu::RenderPass<'a>){
        for (i, texture) in self.textures.values().enumerate(){
            let texture = resource_manager.get_texture(texture).unwrap();
            let texture_bind_group = texture.borrow_bind_group();
            render_pass.set_bind_group(i as u32, &texture_bind_group, &[]);
        }
    }

    pub fn add_pipeline(&mut self, pipeline: ResourceHandle){
        self.pipelines.push(pipeline);
    }
}
