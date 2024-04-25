use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use crate::utils::handle::Handle;
use crate::pipeline::{Pipeline, PipelineBuildSettings};
use crate::types::material::Material;
use crate::types::mesh::MeshLayout;
use crate::types::shader::Shader;
use super::resource_handle::ResourceHandle;
use super::resource_manager::{ResourceManager, ResourceType};

pub struct PipelineManager{
    pipelines: HashMap<ResourceHandle, Pipeline>
}

impl PipelineManager{
    pub fn new() -> Self{
        Self{
            pipelines: HashMap::new()
        }
    }

    pub fn create_or_get_pipeline(&mut self, device: &wgpu::Device, mesh_layout: &MeshLayout,
                                  material_bind_groups: Vec<Handle<wgpu::BindGroupLayout>>,
                                  shader: &Shader,
                                  shader_handle: ResourceHandle) -> ResourceHandle {
        let mut config = PipelineBuildSettings::new()
            .use_depth(false);

        // For each vertex buffer layout in the mesh layout, add it to the pipeline config
        for vertex_buffer_layout in mesh_layout.get_vertex_buffer_layouts().iter(){
            config = config.add_vertex_descriptor(vertex_buffer_layout.clone());
        }

        // For each bind group layout in the material, add it to the pipeline config
        for bind_group in material_bind_groups.iter(){
            config = config.add_bind_group(bind_group);
        }

        // Add the shader to the pipeline config
        config = config.set_shader(shader);

        config.calculate_hash();

        let config_hash = config.get_uuid();

        for (handle, pipeline) in self.pipelines.iter() {
            if pipeline.get_uuid() == config_hash {
                return handle.clone();
            }
        }

        let handle = self.create_pipeline(device, config, shader_handle.clone());
        handle
    }

    pub fn create_pipeline(&mut self, device: &wgpu::Device, config: PipelineBuildSettings,
                            shader_handle: ResourceHandle) -> ResourceHandle {
        let handle = ResourceHandle::new(ResourceType::Pipeline);
        let pipeline = Pipeline::new(device, config, shader_handle.clone());
        self.pipelines.insert(handle.clone(), pipeline);
        handle
    }

    pub fn get_pipeline(&self, handle: &ResourceHandle) -> Option<&Pipeline>{
        self.pipelines.get(handle)
    }

    pub fn get_all_pipelines(&self) -> Vec<&Pipeline>{
        self.pipelines.values().collect()
    }
}

impl PipelineManager {
    pub(crate) fn get_all_pipeline_handles(&self) -> Vec<ResourceHandle> {
        self.pipelines.keys().cloned().collect()
    }
}