use crate::managers::resource_handle::ResourceHandle;

pub struct Model{
    mesh: ResourceHandle,
    material: ResourceHandle
}

impl Model{
    pub fn new(mesh: ResourceHandle, material: ResourceHandle) -> Self{
        Self{
            mesh,
            material
        }
    }

    pub fn get_mesh(&self) -> &ResourceHandle{
        &self.mesh
    }

    pub fn get_material(&self) -> &ResourceHandle{
        &self.material
    }
}