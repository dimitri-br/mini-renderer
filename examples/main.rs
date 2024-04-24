use log::info;
use minirenderer::Renderer;


fn main() {
    let renderer = Renderer::new();

    // Load a mesh
    let resource_manager_handle = renderer.get_resource_manager();
    let mut resource_manager = resource_manager_handle.get();

    let mesh_handle = resource_manager.load_mesh("assets/meshes/cube.obj");

    // Load a texture
    let texture_handle = resource_manager.load_texture("assets/textures/cube.jpeg");

    // Create a material
    let material_handle = resource_manager.create_material();
    //resource_manager.assign_texture_to_material(&material_handle, &texture_handle, "diffuse_texture");

    // Create a model
    let model_handle = resource_manager.create_model(&mesh_handle, &material_handle);

    info!("Model handle: {:?}", model_handle);
    // Load a shader
    let shader_handle = resource_manager.load_shader(include_str!("../assets/shaders/shader.wgsl"));

    info!("Shader handle: {:?}", shader_handle);

    // Assign the shader to the material
    resource_manager.assign_shader_to_material(&material_handle, &shader_handle);

    // Create a pipeline
    let pipeline_handle = resource_manager.create_pipeline(&mesh_handle, &material_handle);

    info!("Pipeline handle: {:?}", pipeline_handle);
    
    // Drop the resource manager, so that the renderer can take ownership of it
    drop(resource_manager);


    renderer.run();
}   