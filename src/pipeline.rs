use log::error;
use crate::managers::resource_handle::ResourceHandle;
use crate::types::renderable::Renderable;

pub struct Pipeline{
    uuid: u64,
    pipeline: wgpu::RenderPipeline,
    shader: ResourceHandle  // This tells us which shader is used by this pipeline
                            // so we can figure out which materials can use this pipeline
}

impl Pipeline {
    pub(crate) fn get_shader(&self) -> ResourceHandle {
        self.shader.clone()
    }
}

pub struct PipelineBuildSettings<'a>{
    uuid: u64,
    pub vertex_descriptors: Vec<wgpu::VertexBufferLayout<'static>>,
    pub bind_groups: Vec<&'a wgpu::BindGroupLayout>,
    pub shader: Option<&'a wgpu::ShaderModule>,
    pub use_depth: bool,
}


impl Pipeline{
    pub fn new(device: &wgpu::Device, settings: PipelineBuildSettings, shader_handle: ResourceHandle) -> Self{
        let uuid = settings.get_uuid();
        let layout = Self::create_layout(device, &settings.bind_groups);

        // If we don't have a shader, panic
        let shader = settings.shader.unwrap_or_else(||{
            error!("No shader provided for pipeline creation.");
            panic!("No shader provided for pipeline creation.");
        });

        let pipeline = Self::create_pipeline(device, layout, shader,
                                             settings.vertex_descriptors, settings.use_depth);

        Self{
            uuid,
            pipeline,
            shader: shader_handle
        }
    }
    
    fn create_layout(device: &wgpu::Device, bind_group_layouts: &Vec<&wgpu::BindGroupLayout>) -> wgpu::PipelineLayout{
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{
            label: Some("Pipeline Layout"),
            bind_group_layouts: &bind_group_layouts,
            push_constant_ranges: &[],
        })
    }

    fn create_pipeline(device: &wgpu::Device, layout: wgpu::PipelineLayout,
                       shader: &wgpu::ShaderModule,
                        vertex_buffer_layouts: Vec<wgpu::VertexBufferLayout>,
                        use_depth: bool) -> wgpu::RenderPipeline {

        let depth_stencil = if cfg!(target_arch = "wasm32") {
            None
        } else {
            if use_depth {
                Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                })
            }else{
                None
            }
        };

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vertex_main",
                buffers: &vertex_buffer_layouts,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fragment_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        })
    }

    pub fn get_uuid(&self) -> u64{
        self.uuid
    }
}

impl<'a> PipelineBuildSettings<'a>{
    pub fn new() -> Self{
        Self{
            uuid: 0,
            vertex_descriptors: Vec::new(),
            bind_groups: Vec::new(),
            shader: None,
            use_depth: false,
        }
    }

    pub fn add_vertex_descriptor(mut self, vertex_descriptor: wgpu::VertexBufferLayout<'static>) -> Self{
        self.vertex_descriptors.push(vertex_descriptor);
        self
    }

    pub fn add_bind_group(mut self, bind_group: &'a wgpu::BindGroupLayout) -> Self{
        self.bind_groups.push(bind_group);
        self
    }

    pub fn set_shader(mut self, shader: &'a wgpu::ShaderModule) -> Self{
        self.shader = Some(shader);
        self
    }

    pub fn use_depth(mut self, use_depth: bool) -> Self{
        self.use_depth = use_depth;
        self
    }

    pub fn calculate_hash(&mut self){
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        // Hash the vertex descriptors
        for descriptor in &self.vertex_descriptors{
            descriptor.hash(&mut hasher);
        }
        self.uuid = hasher.finish();
    }

    pub fn get_uuid(&self) -> u64{
        self.uuid
    }
}

impl<'a> Renderable<'a> for Pipeline{
    fn render<'b>(&'b self, render_pass: &'a mut wgpu::RenderPass<'b>) {
        render_pass.set_pipeline(&self.pipeline);
    }
}
