use std::collections::HashMap;
use log::info;
use crate::utils::handle::Handle;
use crate::managers::shader_manager::ShaderManager;
use crate::pipeline::Pipeline;
use crate::types::material::Material;
use crate::types::model::Model;
use crate::types::mesh::Mesh;
use crate::types::texture::Texture;
use crate::utils::buffer::*;

use super::pipeline_manager::PipelineManager;
use super::resource_handle::ResourceHandle;

/// # Resource Type
///
/// Represents the type of a resource
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ResourceType{
    Mesh,
    Texture,
    Material,
    Pipeline,
    Shader,
    Model // A model is a combination of a mesh and a material, used for rendering
}

/// # Resource Manager
///
/// Manages resources such as meshes, textures, materials, and models
pub struct ResourceManager{
    meshes: HashMap<ResourceHandle, Mesh>,
    mesh_vertex_buffers: HashMap<ResourceHandle, Vec<Buffer>>,
    mesh_index_buffers: HashMap<ResourceHandle, Vec<Buffer>>,

    textures: HashMap<ResourceHandle, Texture>,
    materials: HashMap<ResourceHandle, Material>,
    models: HashMap<ResourceHandle, Model>,

    shader_manager: ShaderManager,
    pipeline_manager: PipelineManager,

    _device: Handle<wgpu::Device>,
    _queue: Handle<wgpu::Queue>
}

impl ResourceManager{
    pub(crate) fn new(device: Handle<wgpu::Device>, queue: Handle<wgpu::Queue>) -> Self{
        Self{
            meshes: HashMap::new(),
            mesh_vertex_buffers: HashMap::new(),
            mesh_index_buffers: HashMap::new(),

            textures: HashMap::new(),
            materials: HashMap::new(),
            models: HashMap::new(),

            shader_manager: ShaderManager::new(),
            pipeline_manager: PipelineManager::new(),

            _device: device,
            _queue: queue
        }
    }

    /// # Load Mesh
    ///
    /// Loads a mesh from a file and returns a handle to it
    pub fn load_mesh(&mut self, path: &str) -> ResourceHandle{
        let mesh = Mesh::load_obj(path);
        let handle = ResourceHandle::new(ResourceType::Mesh);

        // We need to create a buffer for each submesh
        let mut vertex_buffers = Vec::new();
        let mut index_buffers = Vec::new();

        for sub_mesh in mesh.get_sub_meshes(){
            let vertices = sub_mesh.get_vertices();
            let indices = sub_mesh.get_indices();

            let vertex_buffer = Buffer::create_buffer_from_type(&self._device,
                                                                vertices, BufferType::Vertex);
            let index_buffer = Buffer::create_buffer_from_type(&self._device,
                                                               &indices.as_slice(), BufferType::Index);

            vertex_buffers.push(vertex_buffer);
            index_buffers.push(index_buffer);
        }

        self.meshes.insert(handle.clone(), mesh);

        self.mesh_vertex_buffers.insert(handle.clone(), vertex_buffers);
        self.mesh_index_buffers.insert(handle.clone(), index_buffers);

        handle
    }

    /// # Load Texture
    ///
    /// Loads a texture from a file and returns a handle to it
    pub fn load_texture(&mut self, path: &str) -> ResourceHandle{
        let texture = Texture::load_from_file(&self._device, &self._queue, path);
        let handle = ResourceHandle::new(ResourceType::Texture);

        self.textures.insert(handle.clone(), texture);

        handle
    }

    /// # Create Material
    ///
    /// Creates a new material and returns a handle to it
    pub fn create_material(&mut self) -> ResourceHandle{
        let material = Material::new();
        let handle = ResourceHandle::new(ResourceType::Material);

        self.materials.insert(handle.clone(), material);

        handle
    }

    /// # Assign Texture to Material
    ///
    /// Assigns a texture to a material
    pub fn assign_texture_to_material(&mut self, material_handle: &ResourceHandle, texture_handle: &ResourceHandle, name: &str){
        let material = self.materials.get_mut(material_handle).unwrap();

        material.add_texture(name, texture_handle.clone());
    }

    /// # Assign Shader to Material
    ///
    /// Assigns a shader to a material
    pub fn assign_shader_to_material(&mut self, material_handle: &ResourceHandle, shader_handle: &ResourceHandle){
        let material = self.materials.get_mut(material_handle).unwrap();

        material.set_shader(shader_handle.clone());
    }

    /// # Load Shader
    ///
    /// Loads a shader from a file and returns a handle to it
    pub fn load_shader(&mut self, path: &str) -> ResourceHandle{
        self.shader_manager.create_shader(&self._device, path)
    }

    /// # Create Model
    ///
    /// Creates a new model and returns a handle to it
    pub fn create_model(&mut self, mesh_handle: &ResourceHandle, material_handle: &ResourceHandle) -> ResourceHandle{
        let handle = ResourceHandle::new(ResourceType::Model);

        let model = Model::new(mesh_handle.clone(), material_handle.clone());

        self.models.insert(handle.clone(), model);

        handle
    }

    /// # Create Pipeline
    ///
    /// Creates a new pipeline and returns a handle to it
    pub fn create_pipeline(&mut self, mesh_handle: &ResourceHandle, material_handle: &ResourceHandle) -> ResourceHandle{
        let mesh = self.meshes.get(mesh_handle).unwrap();
        let material = self.materials.get(material_handle).unwrap();
        let bind_group_layouts = material.get_bind_group_layouts(self);
        let shader = self.shader_manager.get_shader(&material.get_shader()).unwrap_or_else(
            || panic!("Shader not found")
        );

        let pipeline_handle = self.pipeline_manager.create_or_get_pipeline(
            &self._device,
            mesh.get_layout(),
            bind_group_layouts,
            shader,
            material.get_shader().clone()
        );

        pipeline_handle
    }
}

impl ResourceManager{
    // Getters are pub(crate) because we don't want the user to access the internal resources directly

    pub(crate) fn get_mesh(&self, handle: &ResourceHandle) -> Option<&Mesh>{
        self.meshes.get(handle)
    }

    pub(crate) fn get_mesh_vertex_buffers(&self, handle: &ResourceHandle) -> Option<&Vec<Buffer>>{
        self.mesh_vertex_buffers.get(handle)
    }

    pub(crate) fn get_mesh_index_buffers(&self, handle: &ResourceHandle) -> Option<&Vec<Buffer>>{
        self.mesh_index_buffers.get(handle)
    }

    pub(crate) fn get_texture(&self, handle: &ResourceHandle) -> Option<&Texture>{
        self.textures.get(handle)
    }

    pub(crate) fn get_material(&self, handle: &ResourceHandle) -> Option<&Material>{
        self.materials.get(handle)
    }

    pub(crate) fn get_model(&self, handle: &ResourceHandle) -> Option<&Model>{
        self.models.get(handle)
    }

    pub(crate) fn get_shader(&self, handle: &ResourceHandle) -> Option<&wgpu::ShaderModule>{
        self.shader_manager.get_shader(handle)
    }

    pub(crate) fn get_pipeline(&self, handle: &ResourceHandle) -> Option<&Pipeline>{
        self.pipeline_manager.get_pipeline(handle)
    }

    pub(crate) fn get_all_meshes(&self) -> Vec<&Mesh>{
        self.meshes.values().collect()
    }

    pub(crate) fn get_all_mesh_vertex_buffers(&self) -> Vec<&Vec<Buffer>>{
        self.mesh_vertex_buffers.values().collect()
    }

    pub(crate) fn get_all_mesh_index_buffers(&self) -> Vec<&Vec<Buffer>>{
        self.mesh_index_buffers.values().collect()
    }

    pub(crate) fn get_all_textures(&self) -> Vec<&Texture>{
        self.textures.values().collect()
    }

    pub(crate) fn get_all_materials(&self) -> Vec<&Material>{
        self.materials.values().collect()
    }

    pub(crate) fn get_all_models(&self) -> Vec<&Model>{
        self.models.values().collect()
    }

    pub(crate) fn get_all_pipelines(&self) -> Vec<&Pipeline>{
        self.pipeline_manager.get_all_pipelines()
    }
}

// Getters for all handles
impl ResourceManager{
    pub(crate) fn get_all_mesh_handles(&self) -> Vec<ResourceHandle>{
        self.meshes.keys().cloned().collect()
    }
    
    pub(crate) fn get_all_texture_handles(&self) -> Vec<ResourceHandle>{
        self.textures.keys().cloned().collect()
    }
    
    pub(crate) fn get_all_material_handles(&self) -> Vec<ResourceHandle>{
        self.materials.keys().cloned().collect()
    }
    
    pub(crate) fn get_all_model_handles(&self) -> Vec<ResourceHandle>{
        self.models.keys().cloned().collect()
    }
    
    pub(crate) fn get_all_shader_handles(&self) -> Vec<ResourceHandle>{
        self.shader_manager.get_all_shader_handles()
    }
    
    pub(crate) fn get_all_pipeline_handles(&self) -> Vec<ResourceHandle>{
        self.pipeline_manager.get_all_pipeline_handles()
    }
}