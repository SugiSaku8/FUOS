use anyhow::{Context, Result};
use std::sync::Arc;
use winit::{dpi::PhysicalSize, window::Window};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex { position: [f32; 2], color: [f32; 3] }

pub struct TimeFinder {
    pub window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_capacity: usize,
}

impl TimeFinder {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::METAL | wgpu::Backends::VULKAN,
            ..Default::default()
        });
        let surface = instance.create_surface(window.clone()).context("時間ファインダーの描画面を作成できません")?;
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.context("時間ファインダー用GPUを取得できません")?;
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("SOS Time Finder"), required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_defaults(), memory_hints: wgpu::MemoryHints::MemoryUsage,
            trace: wgpu::Trace::Off,
        }).await?;
        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats.iter().copied().find(|format| format.is_srgb()).unwrap_or(caps.formats[0]);
        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT, format,
            width: size.width.max(1), height: size.height.max(1),
            present_mode: caps.present_modes.first().copied().unwrap_or(wgpu::PresentMode::Fifo),
            desired_maximum_frame_latency: 2,
            alpha_mode: caps.alpha_modes.first().copied().unwrap_or(wgpu::CompositeAlphaMode::Auto),
            view_formats: vec![],
        };
        surface.configure(&device, &config);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("SOS Time Finder Shader"), source: wgpu::ShaderSource::Wgsl(r#"
struct In { @location(0) position: vec2<f32>, @location(1) color: vec3<f32> };
struct Out { @builtin(position) position: vec4<f32>, @location(0) color: vec3<f32> };
@vertex fn vs_main(v: In) -> Out { var o: Out; o.position=vec4<f32>(v.position,0.0,1.0); o.color=v.color; return o; }
@fragment fn fs_main(v: Out) -> @location(0) vec4<f32> { return vec4<f32>(v.color,1.0); }
"#.into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("SOS Time Finder Layout"), bind_group_layouts: &[], push_constant_ranges: &[] });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("SOS Time Finder Pipeline"), layout: Some(&layout),
            vertex: wgpu::VertexState { module: &shader, entry_point: Some("vs_main"), compilation_options: Default::default(), buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as u64, step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x2 }, wgpu::VertexAttribute { offset: 8, shader_location: 1, format: wgpu::VertexFormat::Float32x3 }],
            }] },
            fragment: Some(wgpu::FragmentState { module: &shader, entry_point: Some("fs_main"), compilation_options: Default::default(), targets: &[Some(wgpu::ColorTargetState { format, blend: Some(wgpu::BlendState::REPLACE), write_mask: wgpu::ColorWrites::ALL })] }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, ..Default::default() },
            depth_stencil: None, multisample: Default::default(), multiview: None, cache: None,
        });
        let vertex_capacity = 2048;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("SOS Time Finder Vertices"), size: (vertex_capacity * std::mem::size_of::<Vertex>()) as u64, usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        Ok(Self { window, surface, device, queue, config, pipeline, vertex_buffer, vertex_capacity })
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 { self.config.width=size.width; self.config.height=size.height; self.surface.configure(&self.device,&self.config); }
    }

    pub fn render(
        &mut self,
        progress: f32,
        history_len: usize,
        current_time: f64,
        max_time: f64,
        options_open: bool,
        show_temperature: bool,
        flow_fraction: f32,
    ) {
        let temp_label = if show_temperature { "温度可視化 ON" } else { "温度可視化 OFF" };
        self.window.set_title(&format!(
            "SOS 時間ファインダー | t={current_time:.2}/{max_time:.2}s | 履歴 {}点 | OPTIONS: 左チェック=温度色 {} / スライダー=流線 {:.0}%",
            history_len, temp_label, flow_fraction.clamp(0.05, 1.0) * 100.0
        ));
        let p=progress.clamp(0.0,1.0); let mut v=Vec::with_capacity(512);
        let mut rect=|x0:f32,y0:f32,x1:f32,y1:f32,c:[f32;3]| {
            v.extend_from_slice(&[
                Vertex{position:[x0,y0],color:c},Vertex{position:[x1,y0],color:c},Vertex{position:[x1,y1],color:c},
                Vertex{position:[x0,y0],color:c},Vertex{position:[x1,y1],color:c},Vertex{position:[x0,y1],color:c},
            ]);
        };
        // Timeline occupies the lower band. OPTIONS is always visible at upper-right.
        rect(-0.92,-0.72,0.92,-0.48,[0.16,0.22,0.29]);
        rect(-0.92,-0.72,-0.92+1.84*p,-0.48,[0.05,0.78,0.92]);
        for i in 0..=10 { let x=-0.92+1.84*i as f32/10.0; rect(x-0.004,-0.78,x+0.004,-0.42,[0.72,0.82,0.88]); }
        // OPTIONS button; a brighter fill indicates the panel is open.
        rect(0.52,0.58,0.92,0.88, if options_open { [0.05,0.78,0.92] } else { [0.20,0.34,0.44] });
        // Render the OPTIONS label with a tiny 3x5 bitmap font so it remains visible
        // without depending on a platform-specific text renderer.
        let glyphs: [[u8; 5]; 6] = [
            [0b111,0b101,0b101,0b101,0b111], // O
            [0b110,0b101,0b110,0b100,0b100], // P
            [0b111,0b010,0b010,0b010,0b010], // T
            [0b111,0b010,0b010,0b010,0b111], // I
            [0b101,0b111,0b111,0b111,0b101], // N
            [0b111,0b100,0b111,0b001,0b111], // S
        ];
        let label_chars = [0_usize,1,2,3,0,4,5];
        let cell_w = 0.010_f32;
        let cell_h = 0.026_f32;
        let start_x = 0.565_f32;
        let start_y = 0.815_f32;
        for (letter, glyph_index) in label_chars.iter().copied().enumerate() {
            for row in 0..5 {
                for col in 0..3 {
                    if glyphs[glyph_index][row] & (1 << (2-col)) != 0 {
                        let x = start_x + letter as f32 * 0.044 + col as f32 * cell_w;
                        let y = start_y - row as f32 * cell_h;
                        rect(x,y-cell_h*0.78,x+cell_w*0.78,y,[0.92,0.98,1.0]);
                    }
                }
            }
        }
        if options_open {
            // Temperature visualization toggle (checked = cyan/green accent).
            rect(-0.90,0.30,-0.84,0.42,[0.72,0.82,0.88]);
            if show_temperature { rect(-0.89,0.31,-0.85,0.41,[0.05,0.90,0.62]); }
            // Flow amount slider: 5% to 100% of available seed points.
            rect(-0.60,0.32,0.82,0.40,[0.16,0.22,0.29]);
            let slider_x = -0.60 + 1.42 * ((flow_fraction.clamp(0.05, 1.0) - 0.05) / 0.95);
            rect(-0.60,0.32,slider_x,0.40,[0.05,0.78,0.92]);
            rect(slider_x-0.018,0.27,slider_x+0.018,0.45,[0.88,0.96,1.0]);
            // Panel separator.
            rect(-0.94,0.20,0.94,0.215,[0.20,0.34,0.44]);
        }
        drop(rect);
        let required=v.len(); if required>self.vertex_capacity { return; }
        self.queue.write_buffer(&self.vertex_buffer,0,bytemuck::cast_slice(&v));
        let frame=match self.surface.get_current_texture(){Ok(f)=>f,Err(_)=>{self.surface.configure(&self.device,&self.config);return;}};
        let view=frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder=self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label:Some("SOS Time Finder Encoder")});
        { let mut pass=encoder.begin_render_pass(&wgpu::RenderPassDescriptor{label:Some("SOS Time Finder Pass"),color_attachments:&[Some(wgpu::RenderPassColorAttachment{view:&view,resolve_target:None,ops:wgpu::Operations{load:wgpu::LoadOp::Clear(wgpu::Color{r:0.025,g:0.045,b:0.075,a:1.0}),store:wgpu::StoreOp::Store},depth_slice:None})],depth_stencil_attachment:None,timestamp_writes:None,occlusion_query_set:None}); pass.set_pipeline(&self.pipeline); pass.set_vertex_buffer(0,self.vertex_buffer.slice(..)); pass.draw(0..v.len() as u32,0..1); }
        self.queue.submit(Some(encoder.finish())); frame.present();
    }
}
