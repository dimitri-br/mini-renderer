use log::info;
use minirenderer::{AsBytes, Renderer, RenderFramework, ResourceHandle, Transform};


struct Camera {
    position: glam::Vec3,
    rotation: glam::Quat,
    fov: f32,
    aspect: f32,
    near: f32,
    far: f32,
}

impl Camera {
    fn new() -> Self {
        Self {
            position: glam::Vec3::new(0.0, 0.0, 0.0),
            rotation: glam::Quat::from_euler(glam::EulerRot::YXZ, 0.0, 0.0, 0.0),
            fov: 45.0,
            aspect: 1.0,
            near: 0.1,
            far: 100.0,
        }
    }

    fn get_view_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_translation(self.position) * glam::Mat4::from_quat(self.rotation)
    }

    fn get_projection_matrix(&self) -> glam::Mat4 {
        glam::Mat4::perspective_rh_gl(self.fov.to_radians(), self.aspect, self.near, self.far)
    }
}

struct CameraUniform {
    view: glam::Mat4,
    projection: glam::Mat4,
}

impl CameraUniform {
    fn new(camera: &Camera) -> Self {
        Self {
            view: camera.get_view_matrix(),
            projection: camera.get_projection_matrix(),
        }
    }
}

impl AsBytes for CameraUniform {
    fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                (self as *const Self) as *const u8,
                std::mem::size_of::<Self>(),
            )
        }
    }
}

pub struct RenderState {
    // Persistent Variables

    // Uniforms
    camera_handle: ResourceHandle,

    // Meshes
    mesh_handle: ResourceHandle,

    // Textures
    texture_handle: ResourceHandle,

    // Materials
    material_handle: ResourceHandle,

    // Models
    model_handle: ResourceHandle,
}

impl RenderState {
    pub fn new() -> Self {
        Self {
            camera_handle: ResourceHandle::default(),
            mesh_handle: ResourceHandle::default(),
            texture_handle: ResourceHandle::default(),
            material_handle: ResourceHandle::default(),
            model_handle: ResourceHandle::default(),
        }
    }
}

fn main() {
    let renderer = Renderer::new();

    let state = RenderState::new();

    let render_framework = RenderFramework::new(state, renderer, initialize_renderer, update_renderer);

    render_framework.run();
}


// Function to load a mesh, texture, material, and model - and return the handles
fn load_mesh_texture_material_model(renderer: &Renderer) -> (ResourceHandle, ResourceHandle, ResourceHandle, ResourceHandle) {
    let resource_manager_handle = renderer.get_resource_manager();
    let mut resource_manager = resource_manager_handle.get();

    let mesh_handle = resource_manager.load_mesh("assets/meshes/cube.glb");
    let texture_handle = resource_manager.load_texture("assets/textures/cube.jpeg");
    let material_handle = resource_manager.create_material();
    resource_manager.assign_texture_to_material(&material_handle, &texture_handle, "diffuse");

    let mut transform = Transform::new();
    let position = glam::Vec3::new(0.0, 0.0, -15.0);
    // Rotate 45 degrees around the y-axis and 45 degrees around the x-axis
    let rotation = glam::Quat::from_euler(glam::EulerRot::YXZ, 45.0, 45.0, 0.0);
    let scale = glam::Vec3::new(1.0, 1.0, 1.0);
    transform.set_position(position);
    transform.set_rotation(rotation);
    transform.set_scale(scale);

    let model_handle = resource_manager.create_model(&mesh_handle, &material_handle, transform);
    let transform_uniform = resource_manager.get_model_transform_uniform_handle(&model_handle);
    resource_manager.assign_uniform_to_material(&material_handle, &transform_uniform, "transform");

    (mesh_handle, texture_handle, material_handle, model_handle)
}

fn initialize_renderer(state: &mut RenderState, renderer: &mut Renderer) {
    // Load a mesh
    let resource_manager_handle = renderer.get_resource_manager();

    let (mesh_handle, texture_handle, material_handle, model_handle)
        = load_mesh_texture_material_model(&renderer);

    // Create a camera
    let camera = Camera::new();
    let camera_uniform = CameraUniform::new(&camera);


    let mut resource_manager = resource_manager_handle.get();


    // Load another mesh, but use the same material
    let mesh_handle = resource_manager.load_mesh("assets/meshes/cube.glb");
    let mut second_transform = Transform::new();
    second_transform.set_position(glam::Vec3::new(-2.0, 0.0, -15.0));
    let model_handle = resource_manager.create_model(&mesh_handle, &material_handle, second_transform);

    // Load a uniform
    let camera_handle = resource_manager.create_uniform_buffer(camera_uniform);
    // Assign the uniform to the material
    resource_manager.assign_uniform_to_material(&material_handle, &camera_handle, "camera");


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

    // Store the handles in the state
    state.camera_handle = camera_handle;
    state.mesh_handle = mesh_handle;
    state.texture_handle = texture_handle;
    state.material_handle = material_handle;
    state.model_handle = model_handle;
}

fn update_renderer(state: &mut RenderState, renderer: &mut Renderer) {
    // Rotate the model
    let resource_manager_handle = renderer.get_resource_manager();
    let resource_manager = resource_manager_handle.get();

    let mut transform = resource_manager.get_model_transform(&state.model_handle);

    info!("Transform Position: {:?}", transform.get_position());
    info!("Transform Rotation: {:?}", transform.get_rotation());
    info!("Transform Scale: {:?}", transform.get_scale());

    let rotation = transform.get_rotation();
    let rotation = glam::Quat::from_euler(glam::EulerRot::YXZ, 0.0, 0.01, 0.01) * rotation;
    transform.set_rotation(rotation);
}