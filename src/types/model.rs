use crate::managers::resource_handle::ResourceHandle;
use crate::Transform;
use crate::utils::handle::Handle;

pub struct Model{
    mesh: ResourceHandle,
    material: ResourceHandle,

    transform: Handle<Transform>,
    transform_uniform_handle: ResourceHandle
}

impl Model{
    pub fn new(mesh: ResourceHandle, material: ResourceHandle, transform: Transform, transform_uniform_handle: ResourceHandle) -> Self{
        Self{
            mesh,
            material,
            transform: Handle::new(transform),
            transform_uniform_handle
        }
    }

    pub fn get_mesh(&self) -> &ResourceHandle{
        &self.mesh
    }

    pub fn get_material(&self) -> &ResourceHandle{
        &self.material
    }

    pub fn get_transform(&self) -> Handle<Transform>{
        self.transform.clone()
    }

    pub fn get_transform_uniform_handle(&self) -> ResourceHandle{
        self.transform_uniform_handle.clone()
    }
}

