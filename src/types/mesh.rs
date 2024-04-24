use std::ops::Range;
use log::{error, info};
use wgpu::RenderPass;
use crate::types::{instance::Instance, vertex::Vertex};
use crate::types::renderable::Renderable;

#[derive(Debug, Clone)]
pub struct SubMesh{
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl SubMesh{
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self{
        Self{
            vertices,
            indices,
        }
    }

    pub fn get_vertices(&self) -> &Vec<Vertex> {
        &self.vertices
    }

    pub fn get_indices(&self) -> &Vec<u32> {
        &self.indices
    }

    pub fn get_indices_count(&self) -> usize {
        self.indices.len()
    }
}


#[derive(Debug, Clone)]
pub struct MeshLayout{
    pub vertex_buffer_layouts: Vec<wgpu::VertexBufferLayout<'static>>, // Vertex buffer layouts -
                                            // multiple vertex buffers can be used in a single mesh
    pub index_format: wgpu::IndexFormat,
}

impl MeshLayout{
    pub fn new(vertex_buffer_layouts: Vec<wgpu::VertexBufferLayout<'static>>, index_format: wgpu::IndexFormat) -> Self{
        Self{
            vertex_buffer_layouts,
            index_format,
        }
    }

    pub fn get_vertex_buffer_layouts(&self) -> &Vec<wgpu::VertexBufferLayout<'static>>{
        &self.vertex_buffer_layouts
    }
}

#[derive(Debug, Clone)]
pub struct Mesh{
    sub_meshes: Vec<SubMesh>,
    instances: Vec<Instance>, // Instances are per mesh

    // Mesh layout
    layout: MeshLayout,
}

impl Mesh{

    pub(crate) fn load_obj<T: AsRef<std::path::Path>>(path: T) -> Self{
        let load_options = tobj::LoadOptions {
            single_index: true,
            triangulate: true,
            ..Default::default()
        };

        let obj = tobj::load_obj(path.as_ref(), &load_options).unwrap_or_else(|e| {
            error!("Failed to load obj file: {}", e);
            panic!("Failed to load obj file: {}", e);
        });

        let (models, _) = obj;

        let mut sub_meshes = Vec::new();

        for model in models{
            let mesh = &model.mesh;

            let mut vertices = Vec::new();
            let mut indices = Vec::new();

            for i in 0..mesh.positions.len() / 3{
                let vertex = Vertex{
                    position: [
                        mesh.positions[i * 3],
                        mesh.positions[i * 3 + 1],
                        mesh.positions[i * 3 + 2],
                    ],
                    normal: [
                        mesh.normals[i * 3],
                        mesh.normals[i * 3 + 1],
                        mesh.normals[i * 3 + 2],
                    ],
                    tex_coords: [
                        mesh.texcoords[i * 2],
                        mesh.texcoords[i * 2 + 1],
                    ],
                };

                vertices.push(vertex);
            }

            for i in 0..mesh.indices.len(){
                indices.push(mesh.indices[i]);
            }

            sub_meshes.push(SubMesh::new(vertices, indices));
        }

        info!("Loaded mesh from file: {:?}", path.as_ref());

        // Output submesh information
        for (idx, sub_mesh) in sub_meshes.iter().enumerate(){
            info!("Submesh {} vertices: {:?}", idx, sub_mesh.get_vertices().len());
            info!("Submesh {} indices: {:?}", idx, sub_mesh.get_indices().len());
        }

        Self{
            sub_meshes,
            instances: Vec::new(),
            layout: MeshLayout::new(vec![Vertex::desc()], wgpu::IndexFormat::Uint32),
        }
    }

    pub fn get_sub_meshes(&self) -> &Vec<SubMesh>{
        &self.sub_meshes
    }

    pub fn get_instances(&self) -> &Vec<Instance>{
        &self.instances
    }

    pub fn get_layout(&self) -> &MeshLayout{
        &self.layout
    }
}

impl<'a> Renderable<'a> for Mesh{
    fn render<'b>(&'b self, render_pass: &'a mut RenderPass<'b>) {
        for sub_mesh in self.get_sub_meshes(){
            let indices_count = sub_mesh.get_indices_count();
            render_pass.draw_indexed(0..indices_count as u32, 0, 0..1);
        }
    }
}
