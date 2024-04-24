use std::collections::HashMap;
use crate::utils::handle::Handle;
use crate::managers::resource_handle::ResourceHandle;
use crate::managers::resource_manager::{ResourceManager, ResourceType};

pub struct Material{
    // Textures
    textures: HashMap<String, ResourceHandle>,

    // Shader
    shader_handle: Option<ResourceHandle>,   // Handle to the shader used by this material

    // Acceptable pipelines
    pipelines: Vec<ResourceHandle>
}

impl Material{
    pub fn new() -> Self{
        Self{
            textures: HashMap::new(),
            shader_handle: None, // Just a dummy handle for now
            pipelines: Vec::new()
        }
    }

    pub fn add_texture(&mut self, name: &str, texture: ResourceHandle){
        self.textures.insert(name.to_string(), texture);
    }

    pub fn get_texture(&self, name: &str) -> Option<&ResourceHandle>{
        self.textures.get(name)
    }

    pub fn set_shader(&mut self, shader: ResourceHandle){
        self.shader_handle = Some(shader);
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
            let texture_bind_group = texture.get_bind_group();
            //render_pass.set_bind_group(i as u32, &texture_bind_group, &[]);
        }
    }

    pub fn add_pipeline(&mut self, pipeline: ResourceHandle){
        self.pipelines.push(pipeline);
    }
}
