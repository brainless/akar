use std::sync::Arc;
use std::time::Instant;

use akar_core::AkarCore;
use akar_layout::{Layout, Size};
use akar_winit::process_window_event;
use wgpu::{
    CompositeAlphaMode, CurrentSurfaceTexture, InstanceDescriptor, PresentMode, TextureUsages,
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes},
};

use crate::site::Site;
use crate::sites;

struct AppState {
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    core: AkarCore,
    layout: Layout,
    site: Box<dyn Site>,
}

pub struct App {
    state: Option<AppState>,
    site_name: String,
    screenshot_path: Option<String>,
    exit_after: bool,
    delay_secs: f64,
    width: u32,
    height: u32,
    start_time: Option<Instant>,
    screenshot_taken: bool,
}

impl App {
    pub fn new(
        site_name: String,
        screenshot_path: Option<String>,
        exit_after: bool,
        delay_secs: f64,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            state: None,
            site_name,
            screenshot_path,
            exit_after,
            delay_secs,
            width,
            height,
            start_time: None,
            screenshot_taken: false,
        }
    }
}

fn prepare_layout(state: &mut AppState, size: PhysicalSize<u32>, scale: f32) {
    state.layout.compute(
        state.site.root(),
        (
            Some(size.width as f32 / scale),
            Some(size.height as f32 / scale),
        ),
        |_, _, _, _, _| Size::ZERO,
    );
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        let window_attrs = WindowAttributes::default()
            .with_title("akar webpage demo")
            .with_inner_size(PhysicalSize::new(self.width, self.height));
        let window = Arc::new(event_loop.create_window(window_attrs).unwrap());

        let instance = wgpu::Instance::new(InstanceDescriptor::new_with_display_handle(Box::new(
            event_loop.owned_display_handle(),
        )));
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .unwrap();
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).unwrap();

        let size = window.inner_size();
        let mut surface_config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        surface_config.usage = TextureUsages::RENDER_ATTACHMENT;
        surface_config.present_mode = PresentMode::Fifo;
        surface_config.alpha_mode = CompositeAlphaMode::Opaque;
        let surface_format = surface_config.format;
        surface.configure(&device, &surface_config);

        let core = AkarCore::new(&device, &queue, surface_format);
        let mut layout = Layout::new();

        let mut site = sites::create_site(&self.site_name);
        site.build_layout(&mut layout);

        if self.screenshot_path.is_some() {
            self.start_time = Some(Instant::now());
        }

        self.state = Some(AppState {
            window,
            device,
            queue,
            surface,
            surface_config,
            core,
            layout,
            site,
        });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = &mut self.state else {
            return;
        };

        match event {
            WindowEvent::Resized(new_size) => {
                state.surface_config.width = new_size.width;
                state.surface_config.height = new_size.height;
                state
                    .surface
                    .configure(&state.device, &state.surface_config);
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let size = state.window.inner_size();
                let scale = state.window.scale_factor() as f32;

                state.core.begin_frame(size.width, size.height, scale);

                let viewport_rect = [
                    0.0,
                    0.0,
                    size.width as f32 / scale,
                    size.height as f32 / scale,
                ];

                prepare_layout(state, size, scale);

                state
                    .site
                    .render(&mut state.core, &mut state.layout, viewport_rect);

                let normal_capture = !self.screenshot_taken
                    && self.screenshot_path.is_some()
                    && self.start_time.is_some_and(|t| {
                        t.elapsed() >= std::time::Duration::from_secs_f64(self.delay_secs)
                    });

                if normal_capture {
                    state.core.request_screenshot();
                }

                let output = match state.surface.get_current_texture() {
                    CurrentSurfaceTexture::Success(t) | CurrentSurfaceTexture::Suboptimal(t) => t,
                    CurrentSurfaceTexture::Outdated | CurrentSurfaceTexture::Lost => {
                        state
                            .surface
                            .configure(&state.device, &state.surface_config);
                        state.window.request_redraw();
                        return;
                    }
                    CurrentSurfaceTexture::Timeout
                    | CurrentSurfaceTexture::Occluded
                    | CurrentSurfaceTexture::Validation => {
                        state.window.request_redraw();
                        return;
                    }
                };
                let mut encoder = state
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

                {
                    let surface_view = output
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    let render_view = if normal_capture {
                        state
                            .core
                            .capture_target_view(&state.device, size.width, size.height)
                            .unwrap()
                    } else {
                        surface_view
                    };

                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("main pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &render_view,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });
                    let _ = state.core.end_frame(&state.device, &state.queue, &mut pass);
                }

                if normal_capture {
                    let captured =
                        state
                            .core
                            .take_screenshot(&state.device, &state.queue, encoder, &output);
                    match captured {
                        Ok(frame) => {
                            let path = self.screenshot_path.as_ref().unwrap();
                            match std::fs::File::create(path) {
                                Ok(file) => {
                                    let mut png_encoder =
                                        png::Encoder::new(file, frame.width, frame.height);
                                    png_encoder.set_color(png::ColorType::Rgba);
                                    png_encoder.set_depth(png::BitDepth::Eight);
                                    match png_encoder.write_header() {
                                        Ok(mut writer) => {
                                            if let Err(e) = writer.write_image_data(&frame.rgba) {
                                                eprintln!("Failed to write PNG data: {e}");
                                            } else {
                                                eprintln!("Screenshot saved to {path}");
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to write PNG header: {e}");
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to create file '{path}': {e}");
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Screenshot failed: {e}");
                        }
                    }
                    self.screenshot_taken = true;
                    if self.exit_after {
                        event_loop.exit();
                    }
                } else {
                    state.queue.submit(std::iter::once(encoder.finish()));
                }
                output.present();
                state.window.request_redraw();
            }
            _ => {}
        }

        process_window_event(&mut state.core.input, &event);

        if !matches!(event, WindowEvent::RedrawRequested) {
            state.window.request_redraw();
        }
    }
}
