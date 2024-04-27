use log::info;
// Helpful buffer utilities
use wgpu::util::DeviceExt;

#[derive(Debug, Clone, Copy)]
pub enum BufferType{
    Vertex,
    Index,
    Instance,
    Uniform,
    Storage,
}

pub struct Buffer{
    pub buffer: wgpu::Buffer,
    pub size: usize,
    pub buffer_type: BufferType,
    
    /* 
     * Bind group layout and bind group
     * Used for uniform buffers
     */
    bind_group_layout: Option<wgpu::BindGroupLayout>,
    bind_group: Option<wgpu::BindGroup>,
}

impl Buffer{
    pub fn create_buffer_from_bytes(device: &wgpu::Device, data: &[u8], buffer_type: BufferType) -> Self{
        info!("Creating buffer from bytes: {:?}", buffer_type);
        info!("Data size: {:?}", data.len());

        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor{
                label: Some("Buffer"),
                contents: data,
                usage: match buffer_type{
                    BufferType::Vertex => wgpu::BufferUsages::VERTEX,
                    BufferType::Index => wgpu::BufferUsages::INDEX,
                    BufferType::Instance => wgpu::BufferUsages::VERTEX,
                    BufferType::Uniform => wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
                    BufferType::Storage => wgpu::BufferUsages::STORAGE,
                },
            }
        );

        info!("Buffer created");
        
        let bind_group_layout = match buffer_type{
            BufferType::Uniform => Some(device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor{
                    label: Some("Uniform Buffer Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry{
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        }
                    ],
                }
            )),
            _ => None,
        };
        
        let bind_group = match buffer_type{
            BufferType::Uniform => Some(device.create_bind_group(
                &wgpu::BindGroupDescriptor{
                    label: Some("Uniform Buffer Bind Group"),
                    layout: bind_group_layout.as_ref().unwrap(),
                    entries: &[
                        wgpu::BindGroupEntry{
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding{
                                buffer: &buffer,
                                offset: 0,
                                size: None,
                            }),
                        }
                    ],
                }
            )),
            _ => None,
        };

        Self{
            buffer,
            size: data.len(),
            buffer_type,
            
            bind_group_layout,
            bind_group,
        }
    }

    pub fn create_buffer_from_type<T: AsBytes>(device: &wgpu::Device, data: &T, buffer_type: BufferType) -> Self{
        Self::create_buffer_from_bytes(device, data.as_bytes(), buffer_type)
    }
}

impl Buffer{
    pub fn bind_vertex_buffer<'a>(&'a self, index: u32, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_vertex_buffer(index, self.buffer.slice(..));
    }

    pub fn bind_index_buffer<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_index_buffer(self.buffer.slice(..), wgpu::IndexFormat::Uint32);
    }
    
    pub fn bind_uniform_buffer<'a>(&'a self, index: u32, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_bind_group(index, self.bind_group.as_ref().unwrap(), &[]);
    }
}

impl Buffer{
    pub fn update(&self, queue: &wgpu::Queue, data: &[u8]){
        queue.write_buffer(&self.buffer, 0, data);
    }

    pub fn copy_buffer(&self, device: &wgpu::Device, queue: &wgpu::Queue, buffer: &wgpu::Buffer){
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor{
            label: Some("Buffer Copy Encoder"),
        });

        encoder.copy_buffer_to_buffer(&buffer, 0, &self.buffer, 0, self.size as wgpu::BufferAddress);

        queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn update_from_type<T: AsBytes>(&self, queue: &wgpu::Queue, data: &T){
        self.update(queue, data.as_bytes());
    }

    pub fn update_at(&self, queue: &wgpu::Queue, offset: usize, data: &[u8]){
        queue.write_buffer(&self.buffer, offset as wgpu::BufferAddress, data);
    }

    pub fn update_at_from_type<T: AsBytes>(&self, queue: &wgpu::Queue, offset: usize, data: &T){
        self.update_at(queue, offset, data.as_bytes());
    }

    pub fn get_size(&self) -> usize{
        self.size
    }
    
    pub fn get_buffer(&self) -> &wgpu::Buffer{
        &self.buffer
    }
}

impl Drop for Buffer{
    fn drop(&mut self){
        info!("Dropping buffer");
    }
}


/// Trait for converting a type to a byte slice.
///
/// Must be implemented for types that are used in buffers.
pub trait AsBytes {
    fn as_bytes(&self) -> &[u8];
}

impl AsBytes for &[u8]{
    fn as_bytes(&self) -> &[u8]{
        *self
    }
}

impl AsBytes for &[u32] {
    fn as_bytes(&self) -> &[u8] {
        // Cast to a byte slice
        unsafe {
            std::slice::from_raw_parts(
                self.as_ptr() as *const u8,
                self.len() * std::mem::size_of::<u32>(),
            )
        }
    }
}