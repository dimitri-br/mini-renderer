use log::info;
use minirenderer::{AsBytes, Renderer, ResourceHandle};


pub struct ColorUniform{
    pub color: [f32; 4]
}

impl AsBytes for ColorUniform{
    fn as_bytes(&self) -> &[u8] {
        unsafe{
            std::slice::from_raw_parts(
                (self as *const Self) as *const u8,
                std::mem::size_of::<Self>()
            )

        }
    }
}


fn main() {
    let renderer = Renderer::new();

    // Load a mesh
    let resource_manager_handle = renderer.get_resource_manager();

    let (mesh_handle, texture_handle, material_handle, model_handle)
        = load_mesh_texture_material_model(&renderer);

    // Create a color uniform
    let color_uniform = ColorUniform {
        color: [0.0, 0.0, 0.0, 1.0]
    };

    {
        let mut resource_manager = resource_manager_handle.get();

        // Load a uniform
        let uniform_handle = resource_manager.create_uniform_buffer(color_uniform);
        // Assign the uniform to the material
        resource_manager.assign_uniform_to_material(&material_handle, &uniform_handle, "color");


        // Load a shader
        let shader_handle = resource_manager.load_shader(
            include_str!("../assets/shaders/shader.wgsl")
        );

        // Assign the shader to the material. This is required for rendering,
        // otherwise the material will not be rendered
        resource_manager.assign_shader_to_material(&material_handle, &shader_handle);

        // Create a pipeline. This requires a mesh and a material - the mesh can be any mesh that
        // will be used with the pipeline (based on the shader), and the material must use a shader
        // for the pipeline to use
        //
        // Once the pipeline is created, you can use any material with the same shader as the pipeline
        // so it's not necessary to create a new pipeline for each material. For meshes,
        // the only time you need to create a new pipeline is when the mesh uses a different vertex
        // layout than the mesh used to create the pipeline (e.g, if a mesh uses different vertex attributes
        // or instancing for example.)
        let pipeline_handle = resource_manager.create_pipeline(&mesh_handle, &material_handle);
    }


    // The renderer will now use the registered resources to render the scene
    renderer.run();
}


// Function to load a mesh, texture, material, and model - and return the handles
fn load_mesh_texture_material_model(renderer: &Renderer) -> (ResourceHandle, ResourceHandle, ResourceHandle, ResourceHandle){
    let resource_manager_handle = renderer.get_resource_manager();
    let mut resource_manager = resource_manager_handle.get();

    let mesh_handle = resource_manager.load_mesh("assets/meshes/cube.glb");
    let texture_handle = resource_manager.load_texture("assets/textures/cube.jpeg");
    let material_handle = resource_manager.create_material();
    resource_manager.assign_texture_to_material(&material_handle, &texture_handle, "diffuse");
    let model_handle = resource_manager.create_model(&mesh_handle, &material_handle);
    
    (mesh_handle, texture_handle, material_handle, model_handle)
}