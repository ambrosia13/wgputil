use std::sync::Arc;

use winit::window::Window;

#[derive(Clone)]
pub struct GpuHandle {
    pub instance: Arc<wgpu::Instance>,
    pub adapter: Arc<wgpu::Adapter>,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
}

pub struct FrameRecord {
    pub encoder: wgpu::CommandEncoder,
    pub surface_texture: wgpu::SurfaceTexture,
}

pub struct SurfaceState {
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,

    pub viewport_size: winit::dpi::PhysicalSize<u32>,
    pub window: Arc<Window>,

    pub gpu_handle: GpuHandle,
}

impl SurfaceState {
    pub async fn new(window: Arc<Window>, features: wgpu::Features, limits: wgpu::Limits) -> Self {
        let viewport_size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: features,
                    required_limits: limits,
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = if surface_caps
            .formats
            .contains(&wgpu::TextureFormat::Rgba8Unorm)
        {
            wgpu::TextureFormat::Rgba8Unorm
        } else {
            log::info!(
                "Couldn't use Rgba8Unorm for the surface format, using {:?} instead",
                &surface_caps.formats[0]
            );

            surface_caps.formats[0]
        };

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: viewport_size.width,
            height: viewport_size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let instance = Arc::new(instance);
        let adapter = Arc::new(adapter);
        let device = Arc::new(device);
        let queue = Arc::new(queue);

        Self {
            surface,
            config,
            viewport_size,
            window,
            gpu_handle: GpuHandle {
                instance,
                adapter,
                device,
                queue,
            },
        }
    }

    pub fn reconfigure_surface(&self) {
        self.surface
            .configure(&self.gpu_handle.device, &self.config);
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.viewport_size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.reconfigure_surface();
        }
    }

    pub fn begin_frame(&self) -> Result<FrameRecord, wgpu::SurfaceError> {
        let encoder =
            self.gpu_handle
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Frame Encoder"),
                });

        let surface_texture = self.surface.get_current_texture()?;

        Ok(FrameRecord {
            encoder,
            surface_texture,
        })
    }

    pub fn finish_frame(&self, frame: FrameRecord) {
        self.gpu_handle
            .queue
            .submit(std::iter::once(frame.encoder.finish()));

        frame.surface_texture.present();
    }
}
