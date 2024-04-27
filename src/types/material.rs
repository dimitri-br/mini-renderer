use std::collections::HashMap;
use std::ops::Deref;
use log::{error, info};
use crate::utils::handle::Handle;
use crate::managers::resource_handle::ResourceHandle;
use crate::managers::resource_manager::{ResourceManager, ResourceType};
use crate::types::texture::Texture;
use crate::utils::buffer::{Buffer, BufferType};
use crate::utils::shader_reflect::{Binding, BindingType};

pub struct Material{
    // Textures
    textures: HashMap<String, ResourceHandle>,
    // Uniforms
    uniforms: HashMap<String, ResourceHandle>,

    // Entries are separate, and are generated from the bind group layouts
    // closer to the time of rendering
    bind_groups: HashMap<u32, Handle<wgpu::BindGroup>>,
    // The buffer to the binding (for Uniforms only)
    bind_group_buffers: HashMap<String, Handle<Buffer>>,

    // Flag to check if the bind groups need to be regenerated
    needs_regen: bool,


    // Shader
    shader_handle: Option<ResourceHandle>,   // Handle to the shader used by this material
    shader_bindings: Option<HashMap<String, Binding>>,

    // Acceptable pipelines
    pipelines: Vec<ResourceHandle>,

    // A reference to the device
    _device: Handle<wgpu::Device>,
    _queue: Handle<wgpu::Queue>
}

impl Material{
    pub fn new(device: Handle<wgpu::Device>, queue: Handle<wgpu::Queue>) -> Self{
        Self{
            textures: HashMap::new(),
            uniforms: HashMap::new(),

            bind_groups: HashMap::new(),
            bind_group_buffers: HashMap::new(),
            needs_regen: true,
            
            shader_handle: None, // Just a dummy handle for now
            shader_bindings: None, // we assign when we assign the shader
            pipelines: Vec::new(),

            _device: device,
            _queue: queue
        }
    }

    pub(crate) fn update(&mut self, resource_manager: &ResourceManager){
        // Update the buffers

        // Go through each uniform and each buffer, and check the string name are the same

        // If they are, update the buffer with the new data
        for (buffer_name, buffer_handle) in self.bind_group_buffers.iter(){
            for (uniform_name, uniform_handle) in self.uniforms.iter(){
                if buffer_name == uniform_name{
                    let uniform = resource_manager.get_uniform_buffer(uniform_handle).unwrap();
                    buffer_handle.copy_buffer(&self._device, &self._queue, uniform.get_buffer());
                }
            }
        }
    }

    pub fn add_texture(&mut self, name: &str, texture_handle: ResourceHandle){
        self.textures.insert(name.to_string(), texture_handle);

        // We need to regenerate the bind groups whenever the material is updated
        self.needs_regen = true;
    }

    pub fn add_uniform(&mut self, name: &str, uniform_handle: ResourceHandle){
        self.uniforms.insert(name.to_string(), uniform_handle);

        // We need to regenerate the bind groups whenever the material is updated
        self.needs_regen = true;
    }

    pub fn get_texture(&self, name: &str) -> Option<&ResourceHandle>{
        self.textures.get(name)
    }

    pub fn get_uniform(&self, name: &str) -> Option<&ResourceHandle>{
        self.uniforms.get(name)
    }

    pub fn set_shader(&mut self, shader: ResourceHandle, bindings: HashMap<String, Binding>){
        self.shader_handle = Some(shader);
        self.shader_bindings = Some(bindings);
    }


    pub fn get_shader(&self) -> ResourceHandle{
        self.shader_handle.as_ref().unwrap().clone()
    }
    

    pub fn add_pipeline(&mut self, pipeline: ResourceHandle){
        self.pipelines.push(pipeline);
    }

    
    pub fn generate_bind_groups(&mut self, resource_manager: &ResourceManager){
        // Check if we need to regenerate the bind groups
        if !self.needs_regen{
            return;
        }

        // We generate bind groups for each binding the shader has, using
        // the textures and uniforms we have. We check the string name of the
        // binding against the textures and uniforms we have (saved as keys), and generate the
        // appropriate bindings here. These will be cached
        // so we can reuse them, and only regenerate them if the textures or uniforms change
        let shader_bindings = self.shader_bindings.as_ref().unwrap();

        error!("Generating bind groups");

        // Initial pass to generate the buffers for the uniforms and storage
        for (name, binding) in shader_bindings.iter(){
            match binding.get_binding_type(){
                BindingType::Uniform => {
                    let uniform_handle = self.uniforms.get(name).unwrap_or_else(||{
                        error!("Failed to bind uniform: {}", name);
                        error!("Please ensure the shader and material are correctly configured");
                        panic!();
                    });

                    let uniform = resource_manager.get_uniform_buffer(uniform_handle).unwrap_or_else(||{
                        error!("Failed to bind uniform: {}", name);
                        error!("Please ensure the shader and material are correctly configured");
                        panic!();
                    });

                    // Create another buffer for the bind group
                    let buffer = Buffer::create_buffer_from_type(
                        &self._device,
                        &uniform.get_data().as_bytes(),
                        BufferType::Uniform
                    );

                    let buffer_handle = Handle::new(buffer);

                    info!("Created buffer for uniform: {}", name);

                    self.bind_group_buffers.insert(name.to_string(), buffer_handle.clone());
                },
                BindingType::Storage => {
                    todo!()
                },
                _ => {}
            }
        }

        // Now we have the bindings, figure out which textures and uniforms we need
        // Group -> Entry, so we can generate the bind groups correctly
        let mut entries: HashMap<u32, Vec<wgpu::BindGroupEntry>> = HashMap::new();

        for (name, binding) in shader_bindings.iter(){
            info!("Binding: {}", name);
            match binding.get_binding_type(){
                BindingType::Texture => {
                    info!("Type: Texture");

                    let texture_handle = self.textures.get(name).unwrap_or_else(||{
                        error!("Failed to bind texture: {}", name);
                        error!("Please ensure the shader and material are correctly configured");
                        panic!();
                    });
                    let texture = resource_manager.borrow_texture(texture_handle);
                    let texture_view = texture.get_texture_view();
                    let entry = wgpu::BindGroupEntry{
                        binding: binding.get_binding(),
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    };
                    let entries = entries.entry(binding.get_group()).or_insert_with(Vec::new);
                    entries.push(entry);
                },
                BindingType::TextureSampler => {
                    info!("Type: Texture Sampler");
                    // The name will be *texture_name*_sampler,
                    // so we need to strip the _sampler part
                    let sampler_texture_name = &name[..name.len() - 8];
                    let texture_handle = self.textures.get(sampler_texture_name).unwrap_or_else(||{
                        error!("Failed to bind texture sampler: {}", name);
                        error!("Please ensure the shader and material are correctly configured");
                        panic!();
                    });
                    let texture = resource_manager.borrow_texture(texture_handle);
                    let texture_sampler = texture.get_texture_sampler();
                    let entry = wgpu::BindGroupEntry{
                        binding: binding.get_binding(),
                        resource: wgpu::BindingResource::Sampler(&texture_sampler),
                    };
                    let entries = entries.entry(binding.get_group()).or_insert_with(Vec::new);
                    entries.push(entry);
                },
                BindingType::Uniform => {
                    info!("Type: Uniform");
                    // We already generated the buffer for this, so we just need to get it
                    let buffer_handle = self.bind_group_buffers.get(name).unwrap();

                    // Create the entry
                    let entry = wgpu::BindGroupEntry{
                        binding: binding.get_binding(),
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding{
                            buffer: &buffer_handle.get_buffer(),
                            offset: 0,
                            size: None
                        })
                    };

                    let entries = entries.entry(binding.get_group()).or_insert_with(Vec::new);
                    entries.push(entry);
                },
                BindingType::Storage => {
                    info!("Type: Storage");
                    todo!()
                }
            }
        }

        let shader = resource_manager.get_shader(&self.shader_handle.as_ref().unwrap()).unwrap();

        // For each group, generate the bind group layout
        for (group, entries) in entries.iter(){
            let layout = shader.get_bind_group_layout(*group);
            if let Some(layout) = layout {
                let bind_group = self._device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &layout,
                    entries,
                    label: None
                });
                self.bind_groups.insert(*group, Handle::new(bind_group));
            }
        }

        self.needs_regen = false;
    }

    pub fn bind_material<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>){
        for (group, bind_group) in self.bind_groups.iter(){
            render_pass.set_bind_group(*group, bind_group, &[]);
        }
    }
}
