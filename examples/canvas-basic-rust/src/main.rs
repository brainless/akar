use std::sync::Arc;

use akar_components::{akar_container, canvas_begin, canvas_end, is_visible_world, CanvasConfig, CanvasState, AKAR_THEME_DARK};
use akar_core::AkarCore;
use akar_layout::{Layout, PageConfig, Rect, Size};
use akar_winit::process_window_event;
use wgpu::{CompositeAlphaMode, CurrentSurfaceTexture, InstanceDescriptor, PresentMode, TextureUsages};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes},
};

struct DemoObject {
    bounds: Rect,
    fill: u32,
}

struct AppState {
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    core: AkarCore,
    layout: Layout,
    page: akar_layout::PageLayout,
    canvas_state: CanvasState,
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.run_app(&mut App { state: None }).unwrap();
}

struct App {
    state: Option<AppState>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        let window_attrs = WindowAttributes::default()
            .with_title("akar canvas demo")
            .with_inner_size(LogicalSize::new(800.0, 600.0));
        let window = Arc::new(event_loop.create_window(window_attrs).unwrap());

        let instance = wgpu::Instance::new(InstanceDescriptor::new_with_display_handle(Box::new(
            event_loop.owned_display_handle(),
        )));
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            }))
            .unwrap();
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .unwrap();

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

        let page = layout.page(PageConfig {
            header_height: Some(48.0),
            footer_height: None,
            sidebar_left_width: None,
            sidebar_right_width: None,
        });

        let canvas_state = CanvasState::new();

        self.state = Some(AppState {
            window,
            device,
            queue,
            surface,
            surface_config,
            core,
            layout,
            page,
            canvas_state,
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

                state.layout.compute(
                    state.page.root,
                    (Some(size.width as f32 / scale), Some(size.height as f32 / scale)),
                    |_, _, _, _, _| Size::ZERO,
                );

                akar_container(&mut state.core, &state.layout, state.page.header.unwrap(), 0x1e3a8aff, &AKAR_THEME_DARK);

                let objects = [
                    DemoObject { bounds: Rect::from_xywh(-180.0, -80.0, 120.0, 60.0), fill: 0x3B82F6FF },
                    DemoObject { bounds: Rect::from_xywh(80.0,  -80.0, 120.0, 60.0),  fill: 0x10B981FF },
                    DemoObject { bounds: Rect::from_xywh(-60.0,  40.0, 120.0, 60.0),  fill: 0xF59E0BFF },
                    DemoObject { bounds: Rect::from_xywh(-280.0, 60.0,  80.0, 80.0),  fill: 0xEF4444FF },
                    DemoObject { bounds: Rect::from_xywh(200.0,  20.0, 100.0, 100.0), fill: 0x8B5CF6FF },
                ];

                let config = CanvasConfig::default();
                let (response, mut painter) = canvas_begin(&mut state.core, &state.layout, state.page.main, &mut state.canvas_state, &config);

                for obj in &objects {
                    if is_visible_world(response.visible_world_rect, obj.bounds) {
                        painter.push_quad(obj.bounds, obj.fill, 0x00000000, 0.0, [8.0; 4], 0.0);
                    }
                }

                canvas_end(&mut state.core, painter);

                let output = match state.surface.get_current_texture() {
                    CurrentSurfaceTexture::Success(t) | CurrentSurfaceTexture::Suboptimal(t) => t,
                    _ => return,
                };
                let view = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = state
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
                {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("main pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
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
                state.queue.submit(std::iter::once(encoder.finish()));
                output.present();
            }
            _ => {}
        }

        process_window_event(&mut state.core.input, &event);

        if let WindowEvent::RedrawRequested = event {
        } else {
            state.window.request_redraw();
        }
    }
}
