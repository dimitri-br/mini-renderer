use log::info;
use crate::utils::handle::Handle;
use crate::instance_handle::InstanceHandle;

pub struct DeviceHandle{
    device: Handle<wgpu::Device>,
    queue: Handle<wgpu::Queue>
}

impl DeviceHandle{
    pub fn new(instance: &InstanceHandle) -> Self{
        let adapter = instance.get_adapter();
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor{
                label: Some("Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default()
            },
            None
        )).unwrap();
        info!("Device and Queue created");

        Self{
            device: Handle::new(device),
            queue: Handle::new(queue)
        }
    }

    pub fn get_device(&self) -> Handle<wgpu::Device>{
        self.device.clone()
    }

    pub fn get_queue(&self) -> Handle<wgpu::Queue>{
        self.queue.clone()
    }
}