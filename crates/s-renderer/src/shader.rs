use anyhow::{Result, anyhow};

/// Shader source wrapper.
///
/// Keeps shader creation separate from pipeline creation.
#[derive(Debug)]
pub struct Shader {
    module: wgpu::ShaderModule,
}

impl Shader {
    /// Creates shader from WGSL source.
    pub fn from_wgsl(device: &wgpu::Device, source: &str, label: &'static str) -> Result<Self> {
        if source.trim().is_empty() {
            return Err(anyhow!("shader source is empty"));
        }

        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),

            source: wgpu::ShaderSource::Wgsl(source.into()),
        });

        Ok(Self { module })
    }

    #[inline]
    pub fn module(&self) -> &wgpu::ShaderModule {
        &self.module
    }
}
