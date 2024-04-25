use std::collections::HashMap;
use crate::utils::handle::Handle;
use crate::managers::resource_handle::ResourceHandle;
use crate::managers::resource_manager::{ResourceManager, ResourceType};
use crate::utils::shader_reflect::Binding;

pub struct Material{
    // Textures
    textures: HashMap<String, ResourceHandle>,
    texture_bind_group_layouts: Vec<Handle<wgpu::BindGroupLayout>>,
    texture_bind_groups: Vec<Handle<wgpu::BindGroup>>,

    // Uniforms
    uniforms: HashMap<String, ResourceHandle>,
    uniform_bind_group_layouts: Vec<Handle<wgpu::BindGroupLayout>>,
    uniform_bind_groups: Vec<Handle<wgpu::BindGroup>>,

    // Shader
    shader_handle: Option<ResourceHandle>,   // Handle to the shader used by this material
    shader_bindings: Option<HashMap<String, Binding>>,

    // Acceptable pipelines
    pipelines: Vec<ResourceHandle>
}

impl Material{
    pub fn new() -> Self{
        Self{
            textures: HashMap::new(),
            texture_bind_group_layouts: Vec::new(),
            texture_bind_groups: Vec::new(),
            
            uniforms: HashMap::new(),
            uniform_bind_group_layouts: Vec::new(),
            uniform_bind_groups: Vec::new(),
            
            shader_handle: None, // Just a dummy handle for now
            shader_bindings: None, // we assign when we assign the shader
            pipelines: Vec::new()
        }
    }

    pub fn add_texture(&mut self, name: &str, texture: ResourceHandle){
        self.textures.insert(name.to_string(), texture);
    }

    pub fn get_texture(&self, name: &str) -> Option<&ResourceHandle>{
        self.textures.get(name)
    }

    pub fn set_shader(&mut self, shader: ResourceHandle, bindings: HashMap<String, Binding>){
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
