use wgpu::util::DeviceExt;
use crate::uniform::observable_data::ObservableData;
use crate::utils::buffer::AsBytes;
use crate::utils::handle::Handle;

pub struct UniformBuffer {
    buffer: wgpu::Buffer,
    data: ObservableData<Box<dyn AsBytes>>,
    device: Handle<wgpu::Device>,
}

impl UniformBuffer {
    pub(crate) fn new<T: AsBytes + 'static>(device: Handle<wgpu::Device>, initial_data: T, label: &str) -> Self {
        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: initial_data.as_bytes(),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            },
        );

        Self {
            buffer,
            data: ObservableData::new(Box::new(initial_data)),
            device: device.clone(),
        }
    }

    pub(crate) fn update(&mut self, queue: &wgpu::Queue) {
        if self.data.is_dirty() {
            queue.write_buffer(&self.buffer, 0, self.data.get().as_bytes());
            self.data.clear_dirty();
        }
    }

    pub fn set_data<T: AsBytes + 'static>(&mut self, new_data: T) {
        self.data.set(Box::new(new_data));
    }

    pub(crate) fn get_data(&self) -> &Box<dyn AsBytes> {
        &self.data.get()
    }

    pub(crate) fn get_buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub(crate) fn is_dirty(&self) -> bool {
        self.data.is_dirty()
    }
}
