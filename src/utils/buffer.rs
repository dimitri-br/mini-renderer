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
                    BufferType::Uniform => wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    BufferType::Storage => wgpu::BufferUsages::STORAGE,
                },
            }
        );

        info!("Buffer created");

        Self{
            buffer,
            size: data.len(),
            buffer_type,
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
}

impl Buffer{
    pub fn update(&self, queue: &wgpu::Queue, data: &[u8]){
        queue.write_buffer(&self.buffer, 0, data);
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