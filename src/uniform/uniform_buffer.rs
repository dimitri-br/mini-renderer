use wgpu::util::DeviceExt;
use crate::uniform::observable_data::ObservableData;
use crate::utils::buffer::AsBytes;
use crate::utils::handle::Handle;

pub struct UniformBuffer<T> {
    buffer: wgpu::Buffer,
    data: ObservableData<T>,
    device: Handle<wgpu::Device>,
}

impl<T: AsBytes> UniformBuffer<T> {
    pub fn new(device: Handle<wgpu::Device>, initial_data: T, label: &str) -> Self {
        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: initial_data.as_bytes(),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            },
        );

        Self {
            buffer,
            data: ObservableData::new(initial_data),
            device: device.clone(),
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue) {
        if self.data.is_dirty() {
            queue.write_buffer(&self.buffer, 0, self.data.get().as_bytes());
            self.data.clear_dirty();
        }
    }

    pub fn set_data(&mut self, new_data: T) {
        self.data.set(new_data);
    }

    pub fn get_buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}
