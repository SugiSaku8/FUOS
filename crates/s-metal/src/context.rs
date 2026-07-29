use metal::{CommandQueue, CompileOptions, Device, Library};

pub struct MetalContext {
    device: Device,

    queue: CommandQueue,
}

impl MetalContext {
    pub fn new() -> Self {
        let device = Device::system_default().expect("Metal device not found");

        let queue = device.new_command_queue();

        Self { device, queue }
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn command_queue(&self) -> &CommandQueue {
        &self.queue
    }
    pub fn queue(&self) -> &CommandQueue {
        &self.queue
    }

    /// Compile Metal shader source
    pub fn load_library(&self, source: &str) -> Library {
        self.device
            .new_library_with_source(source, &CompileOptions::new())
            .expect("Failed to compile Metal shader")
    }
}
