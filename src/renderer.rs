use std::collections::HashMap;
use log::{error, info};
use wgpu::StoreOp;
use winit::event::{Event, WindowEvent};
use crate::device_handle::DeviceHandle;
use crate::utils::{handle::Handle, mut_handle::MutHandle};
use crate::instance_handle::InstanceHandle;
use crate::surface_wrapper::SurfaceWrapper;

use winit::window::{Window, WindowBuilder};
use winit::event_loop::{ControlFlow, EventLoop};
use crate::managers::resource_handle::ResourceHandle;
use crate::managers::resource_manager::ResourceManager;
use crate::types::model::Model;
use crate::types::renderable::Renderable;


pub struct RenderFramework<T>{
    state: T, // Persistent state
    init: fn(&mut T, &mut Renderer) -> (),
    update: fn(&mut T, &mut Renderer) -> (),
    renderer: Renderer
}

impl<T> RenderFramework<T>{
    pub fn new(
        state: T,
        renderer: Renderer,
        init: fn(&mut T, &mut Renderer) -> (),
        update: fn(&mut T, &mut Renderer) -> (),
    ) -> Self{
        Self{
            state,
            init,
            update,
            renderer
        }
    }

    pub fn run(mut self){
        (self.init)(&mut self.state, &mut self.renderer);
        self.renderer.run(self.state, self.update);
    }
}



pub struct Renderer{
    instance_handler: InstanceHandle,
    device_handle: DeviceHandle,
    surface_wrapper: SurfaceWrapper,

    window: Handle<Window>,
    event_loop: Option<EventLoop<()>>,

    resource_manager: MutHandle<ResourceManager>,
}

impl Renderer{
    pub fn new() -> Self{
        env_logger::builder()
            .filter_level(log::LevelFilter::Info)
            // We keep wgpu at Error level, as it's very noisy.
            .filter_module("wgpu_core", log::LevelFilter::Info)
            .filter_module("wgpu_hal", log::LevelFilter::Error)
            .filter_module("naga", log::LevelFilter::Error)
            .parse_default_env()
            .init();

        let event_loop = EventLoop::new().unwrap_or_else(
            |e| {
                error!("Failed to create event loop: {}", e);
                panic!("Failed to create event loop: {}", e)
            }
        );

        let window = WindowBuilder::new()
            .with_title("Renderer")
            .with_inner_size(winit::dpi::PhysicalSize::new(1600, 1200))
            .build(&event_loop).unwrap_or_else(
                |e| {
                    error!("Failed to create window: {}", e);
                    panic!("Failed to create window: {}", e)
                }
            );

        let instance_handler = InstanceHandle::new();
        let device_handle = DeviceHandle::new(&instance_handler);

        // Set window to be borrowed for the lifetime of the surface
        let window = Handle::new(window);

        let surface = instance_handler.get_instance().create_surface(window.clone()).unwrap_or_else(
            |e| {
                error!("Failed to create surface: {}", e);
                panic!("Failed to create surface: {}", e)
            }
        );


        let surface_wrapper = SurfaceWrapper::new(surface, &instance_handler, &device_handle, &window);


        let resource_manager = MutHandle::new(ResourceManager::new(
            device_handle.get_device(),
            device_handle.get_queue(),
        ));

        Self{
            instance_handler,
            device_handle,
            surface_wrapper,

            window,
            event_loop: Some(event_loop),

            resource_manager
        }
    }

    pub(crate) fn render(&mut self){

        let rm = self.resource_manager.get();

        let models = rm.get_all_models();

        // Prepare the render. We want to create a collection per pipeline, made up
        // of all the materials that use that pipeline. We then want to render all the
        // meshes that use that material.
        //
        // We can check which meshes use which materials by checking the models
        //
        // The materials can be checked by checking the material's shader against the pipeline's shader

        // Pipeline - List of materials that use the pipeline
        let mut pipeline_materials: HashMap<ResourceHandle, Vec<ResourceHandle>> = HashMap::new();
        // Material, and the meshes that want to use that material
        let mut material_meshes: HashMap<ResourceHandle, Vec<Handle<Model>>> = HashMap::new();

        let pipeline_handles = rm.get_all_pipeline_handles();
        let material_handles = rm.get_all_material_handles();

        // Generate bind groups for all the materials
        for material_handle in material_handles.iter(){
            let mut material = rm.get_material(material_handle).unwrap();
            material.generate_bind_groups(&rm);
        }


        // Populate the pipeline_materials hashmap, and link the materials to the pipelines
        for pipeline_handle in pipeline_handles.iter(){
            let pipeline = rm.get_pipeline(pipeline_handle).unwrap();
            let shader = pipeline.get_shader();
            for material_handle in material_handles.iter(){
                let material = rm.get_material(material_handle).unwrap();

                if material.get_shader() == shader{
                    let materials = pipeline_materials.entry(pipeline_handle.clone()).or_insert_with(Vec::new);
                    materials.push(material_handle.clone());
                }
            }
        }

        // Now we've linked the materials to the pipelines, we can link the meshes to the materials
        // We don't care about the pipeline at this point, as we can get it from the material
        for model in models.iter(){
            let materials = material_meshes.entry(model.get_material().clone()).or_insert_with(Vec::new);
            materials.push(model.clone());
        }

        // Now we have a set of materials linked to pipelines, and a set of materials linked to meshes
        // This means we can link a pipeline, find all the materials that use that pipeline, and then find
        // all the meshes that use those materials
        //
        // This gives us great flexibility in rendering, as we can render all the meshes that use a certain
        // pipeline, and then render all the meshes that use a different pipeline, without having to worry about
        // the order of the meshes in the render loop



        // Get the current frame from the surface
        let frame = self.surface_wrapper.get_surface().get_current_texture()
            .unwrap_or_else(|e| {
                error!("Failed to get current frame: {}", e);
                panic!("Failed to get current frame: {}", e)
            }
        );

        let output = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device_handle.get_device().create_command_encoder(
            &wgpu::CommandEncoderDescriptor{
                label: Some("Render Encoder")
            }
        );

        {
            let mut render_pass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor{
                    label: Some("Render Pass"),
                    color_attachments: &[
                        Some(wgpu::RenderPassColorAttachment{
                            view: &output,
                            resolve_target: None,
                            ops: wgpu::Operations{
                                load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                                store: StoreOp::Store
                            }
                        })
                    ],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                }
            );

            // Using the resource_manager, let's get to rendering
            for (pipeline_handle, materials) in pipeline_materials.iter(){
                let pipeline = rm.get_pipeline(pipeline_handle).unwrap();
                pipeline.render(&mut render_pass);

                for material_handle in materials.iter(){
                    let material = rm.borrow_material(material_handle);

                    for model in material_meshes.get(material_handle).unwrap_or(&Vec::new()).iter(){
                        let mesh = rm.get_mesh(model.get_mesh()).unwrap();

                        let vertex_buffers = rm.get_mesh_vertex_buffers(model.get_mesh()).unwrap();
                        let index_buffers = rm.get_mesh_index_buffers(model.get_mesh()).unwrap();

                        let mut temp_update_material = rm.get_material(material_handle).unwrap();
                        temp_update_material.set_uniform("transform", model.get_transform_uniform_handle(), &rm);

                        info!("Setting transform!");
                        let transform = model.get_transform();
                        info!("Transform: {:?}", transform.get_position());

                        material.bind_material(&mut render_pass);


                        for (idx, submesh) in mesh.get_sub_meshes().iter().enumerate(){
                            vertex_buffers[idx].bind_vertex_buffer(0, &mut render_pass);
                            index_buffers[idx].bind_index_buffer(&mut render_pass);
                            submesh.render(&mut render_pass);
                        }
                    }
                }
            }
        }

        self.device_handle.get_queue().submit(std::iter::once(encoder.finish()));

        frame.present();
    }

    pub fn run<T>(mut self, mut render_state: T, render_func: fn(&mut T, &mut Renderer) -> ()){
        let event_loop = self.event_loop.take().unwrap();

        // Run the event loop, without blocking the current thread
        event_loop.run(move |event, target| {
            target.set_control_flow(ControlFlow::Poll);

            match event{
                Event::AboutToWait{..} => {
                    self.window.request_redraw();
                }
                Event::WindowEvent{
                    event,
                    window_id
                } => {
                    if window_id == self.window.id(){
                        match event{
                            WindowEvent::CloseRequested => {
                                target.exit();
                            }
                            WindowEvent::Resized(new_size) => {
                                self.surface_wrapper.resize_surface(
                                    &self.device_handle.get_device(),
                                    new_size
                                );
                                self.window.request_redraw();
                            }
                            WindowEvent::RedrawRequested => {
                                // Run the render closure
                                render_func(&mut render_state, &mut self);

                                // Update resources here, as they may have changed
                                // We need a closure so we drop the mutable borrow of the resource manager
                                {
                                    let mut rm = self.resource_manager.get();
                                    rm.update_model_transforms();
                                    rm.update_materials();
                                }


                                self.render();
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }).expect("TODO: panic message");
    }

    pub fn get_resource_manager(&self) -> MutHandle<ResourceManager>{
        self.resource_manager.clone()
    }
}

impl Default for Renderer{
    fn default() -> Self{
        Self::new()
    }
}
