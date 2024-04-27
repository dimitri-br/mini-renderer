use std::collections::HashMap;
use std::ops::Deref;
use log::{error, info};
use crate::utils::handle::Handle;
use crate::managers::shader_manager::ShaderManager;
use crate::pipeline::Pipeline;
use crate::Transform;
use crate::types::material::Material;
use crate::types::model::Model;
use crate::types::mesh::Mesh;
use crate::types::shader::Shader;
use crate::types::texture::Texture;
use crate::types::transform::TransformUniform;
use crate::uniform::uniform_buffer::UniformBuffer;
use crate::utils::buffer::*;
use crate::utils::mut_handle::MutHandle;

use super::pipeline_manager::PipelineManager;
use super::resource_handle::ResourceHandle;

/// # Resource Type
///
/// Represents the type of a resource
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ResourceType{
    None,
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
    mesh_instance_buffers: HashMap<ResourceHandle, Option<Vec<Buffer>>>, // Optional instance buffers

    textures: HashMap<ResourceHandle, Handle<Texture>>,
    materials: HashMap<ResourceHandle, Handle<Material>>,
    models: HashMap<ResourceHandle, Handle<Model>>,
    uniforms: HashMap<ResourceHandle, Handle<UniformBuffer>>,

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
            mesh_instance_buffers: HashMap::new(),

            textures: HashMap::new(),
            materials: HashMap::new(),
            models: HashMap::new(),
            uniforms: HashMap::new(),

            shader_manager: ShaderManager::new(device.clone()),
            pipeline_manager: PipelineManager::new(),
            
            _device: device,
            _queue: queue
        }
    }

    pub(crate) fn update_model_transforms(&mut self){
        let mut to_update = Vec::new();
        for model in self.models.values().cloned(){
            let transform = model.get_transform();
            let transform_uniform_handle = model.get_transform_uniform_handle();
            let transform_uniform = TransformUniform::new(&transform.clone());
            
            to_update.push((transform_uniform_handle, transform_uniform));
        }
        
        for (handle, data) in to_update{
            self.update_uniform_buffer(&handle, data);
        }
    }
    
    pub(crate) fn update_materials(&mut self){
        for mut material in self.materials.values().cloned(){
            material.update(self);
        }
    }

    /// # Load Mesh
    ///
    /// Loads a mesh from a file and returns a handle to it
    pub fn load_mesh(&mut self, path: &str) -> ResourceHandle{
        // Check if the path is an obj or fbx
        let mesh = if path.ends_with(".obj"){
            Mesh::load_obj(path)
        }else if path.ends_with(".gltf") || path.ends_with(".glb"){
            Mesh::load_gltf(path)
        }else{
            error!("Unsupported mesh format");
            panic!("Unsupported mesh format")
        };

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

        self.textures.insert(handle.clone(), Handle::new(texture));

        handle
    }

    /// # Create Material
    ///
    /// Creates a new material and returns a handle to it
    pub fn create_material(&mut self) -> ResourceHandle{
        let material = Material::new(self._device.clone(), self._queue.clone());
        let handle = ResourceHandle::new(ResourceType::Material);

        self.materials.insert(handle.clone(), Handle::new(material));

        handle
    }

    /// # Assign Texture to Material
    ///
    /// Assigns a texture to a material
    ///
    /// The name is important. This is the name of the texture in the shader
    /// The sampler is assumed to be called <strong>`texture_name`</strong>_sampler,
    /// where `texture_name` is the name of the texture
    pub fn assign_texture_to_material(&mut self, material_handle: &ResourceHandle, texture_handle: &ResourceHandle, name: &str){
        let material = self.materials.get_mut(material_handle).unwrap();

        material.add_texture(name, texture_handle.clone());
    }

    /// # Assign Shader to Material
    ///
    /// Assigns a shader to a material
    pub fn assign_shader_to_material(&mut self, material_handle: &ResourceHandle, shader_handle: &ResourceHandle){
        let material = self.materials.get_mut(material_handle).unwrap();

        if let Some(bindings) = self.shader_manager.get_shader_bindings(shader_handle){
            material.set_shader(shader_handle.clone(), bindings);
        }else{
            error!("Shader bindings not found");
        }
    }


    /// # Assign Uniform to Material
    ///
    /// Assigns a uniform buffer to a material
    pub fn assign_uniform_to_material(&mut self, material_handle: &ResourceHandle, uniform_handle: &ResourceHandle, name: &str){
        let material = self.materials.get_mut(material_handle).unwrap();

        material.add_uniform(name, uniform_handle.clone());
    }

    /// # Load Shader
    ///
    /// Loads a shader from a file and returns a handle to it
    pub fn load_shader(&mut self, path: &str) -> ResourceHandle{
        self.shader_manager.create_shader(path)
    }

    /// # Create Model
    ///
    /// Creates a new model and returns a handle to it
    pub fn create_model(&mut self, mesh_handle: &ResourceHandle, material_handle: &ResourceHandle, transform: Transform) -> ResourceHandle{
        let handle = ResourceHandle::new(ResourceType::Model);

        // Create the uniform buffer for the model transform
        let transform_handle = self.create_uniform_buffer::<TransformUniform>(transform.clone().into());

        let model = Model::new(mesh_handle.clone(), material_handle.clone(), transform.clone(), transform_handle.clone());

        self.models.insert(handle.clone(), Handle::new(model));

        handle
    }

    /// # Create Pipeline
    ///
    /// Creates a new pipeline and returns a handle to it
    pub fn create_pipeline(&mut self, mesh_handle: &ResourceHandle, material_handle: &ResourceHandle) -> ResourceHandle{
        let mesh = self.meshes.get(mesh_handle).unwrap();
        let material = self.materials.get(material_handle).unwrap();
        let shader = self.shader_manager.get_shader(&material.get_shader()).unwrap_or_else(
            || panic!("Shader not found")
        );

        let bind_group_layouts = shader.get_bind_group_layouts();

        let pipeline_handle = self.pipeline_manager.create_or_get_pipeline(
            &self._device,
            mesh.get_layout(),
            bind_group_layouts,
            shader,
            material.get_shader().clone()
        );

        pipeline_handle
    }


    /// # Create Uniform Buffer
    ///
    /// Creates a new uniform buffer and returns a handle to it
    pub fn create_uniform_buffer<T: AsBytes + 'static>(&mut self, data: T) -> ResourceHandle{
        let handle = ResourceHandle::new(ResourceType::Material);

        let buffer = UniformBuffer::new(self._device.clone(), data, "Uniform Buffer");

        self.uniforms.insert(handle.clone(), Handle::new(buffer));

        handle
    }

    /// # Update Uniform Buffer
    ///
    /// Updates the data in a uniform buffer
    pub fn update_uniform_buffer<T: AsBytes + 'static>(&mut self, handle: &ResourceHandle, data: T){
        let buffer = self.uniforms.get_mut(handle).unwrap();

        buffer.set_data(data);

        buffer.update(&self._queue);
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

    pub(crate) fn get_texture(&self, handle: &ResourceHandle) -> Option<Handle<Texture>>{
        self.textures.get(handle).cloned()
    }

    pub(crate) fn borrow_texture(&self, handle: &ResourceHandle) -> &Texture{
        &self.textures.get(handle).unwrap()
    }

    pub(crate) fn get_material(&self, handle: &ResourceHandle) -> Option<Handle<Material>>{
        self.materials.get(handle).cloned()
    }

    pub(crate) fn borrow_material(&self, handle: &ResourceHandle) -> &Material{
        &self.materials.get(handle).unwrap()
    }

    pub(crate) fn get_shader(&self, handle: &ResourceHandle) -> Option<&Shader>{
        self.shader_manager.get_shader(handle)
    }

    pub(crate) fn get_pipeline(&self, handle: &ResourceHandle) -> Option<&Pipeline>{
        self.pipeline_manager.get_pipeline(handle)
    }

    pub(crate) fn get_uniform_buffer(&self, handle: &ResourceHandle) -> Option<Handle<UniformBuffer>>{
        self.uniforms.get(handle).cloned()
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

    pub(crate) fn get_all_textures(&self) -> Vec<Handle<Texture>>{
        self.textures.values().cloned().collect()
    }

    pub(crate) fn get_all_materials(&self) -> Vec<Handle<Material>>{
        self.materials.values().cloned().collect()
    }

    pub(crate) fn get_all_models(&self) -> Vec<Handle<Model>>{
        self.models.values().cloned().collect()
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

/* Model functions */
impl ResourceManager{
    pub(crate) fn get_model(&self, handle: &ResourceHandle) -> Option<Handle<Model>>{
        self.models.get(handle).cloned()
    }

    pub(crate) fn borrow_model(&self, handle: &ResourceHandle) -> &Model{
        &self.models.get(handle).unwrap()
    }

    pub fn get_model_transform(&self, handle: &ResourceHandle) -> Handle<Transform>{
        self.models.get(handle).unwrap().get_transform()
    }

    pub fn get_model_transform_uniform_handle(&self, handle: &ResourceHandle) -> ResourceHandle{
        self.models.get(handle).unwrap().get_transform_uniform_handle()
    }
}

