use anyhow::{Result, anyhow};

/// GPU global context.
///
/// Owns:
/// - Instance
/// - Adapter
/// - Device
/// - Queue
///
/// Does not own:
/// - Window
/// - Surface
/// - Render pipeline
#[derive(Debug)]
pub struct RenderContext {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl RenderContext {
    pub async fn new() -> Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::METAL,
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .map_err(|_| anyhow!("no compatible GPU adapter found"))?;

        let required_limits = wgpu::Limits::default();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("SOS Renderer Device"),
                required_features: wgpu::Features::empty(),
                required_limits,
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await?;

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
        })
    }

    #[inline]
    pub fn instance(&self) -> &wgpu::Instance {
        &self.instance
    }

    #[inline]
    pub fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }

    #[inline]
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    #[inline]
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}
