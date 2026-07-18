use std::collections::HashMap;
use std::sync::Arc;

use akar_components::{
    akar_button, akar_container, akar_data_item, akar_label, akar_text_input,
    button::ButtonVariant, canvas_begin, canvas_data_item, canvas_end, canvas_portal_begin,
    canvas_portal_end, is_visible_world, BoxStyle, CanvasConfig, CanvasDataItemDescriptor,
    CanvasInput, CanvasState, DataItemStyle, AKAR_THEME_DARK,
};
use akar_core::AkarCore;
use akar_layout::{length, Layout, NodeId, PageConfig, Size, Style, WorldRect};
use akar_winit::process_window_event;
use glam::Vec2;
use wgpu::{
    CompositeAlphaMode, CurrentSurfaceTexture, InstanceDescriptor, PresentMode, TextureUsages,
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes},
};

const LOD_THRESHOLDS: [f32; 3] = [48.0, 120.0, 220.0];

struct DemoObject {
    bounds: WorldRect,
    fill: u32,
    name: &'static str,
}

struct ObjectState {
    button_clicked: bool,
    text_value: String,
    text_cursor: usize,
}

struct PortalLayout {
    layout: Layout,
    root: NodeId,
    item_node: NodeId,
    label_node: NodeId,
    button_node: NodeId,
    input_node: NodeId,
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
    portal_layouts: HashMap<usize, PortalLayout>,
    object_states: Vec<ObjectState>,
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
            .with_title("akar canvas portal demo")
            .with_inner_size(LogicalSize::new(800.0, 600.0));
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

        let page = layout.page(PageConfig {
            header_height: Some(48.0),
            footer_height: None,
            sidebar_left_width: None,
            sidebar_right_width: None,
        });

        let canvas_state = CanvasState::new();

        let object_states = (0..5)
            .map(|_| ObjectState {
                button_clicked: false,
                text_value: String::new(),
                text_cursor: 0,
            })
            .collect();

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
            portal_layouts: HashMap::new(),
            object_states,
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
                    (
                        Some(size.width as f32 / scale),
                        Some(size.height as f32 / scale),
                    ),
                    |_, _, _, _, _| Size::ZERO,
                );

                akar_container(
                    &mut state.core,
                    &state.layout,
                    state.page.header.unwrap(),
                    &BoxStyle::panel(&AKAR_THEME_DARK),
                );

                let objects = [
                    DemoObject {
                        bounds: WorldRect::from_xywh(-180.0, -80.0, 120.0, 60.0),
                        fill: 0x3B82F6FF,
                        name: "Server Alpha",
                    },
                    DemoObject {
                        bounds: WorldRect::from_xywh(80.0, -80.0, 120.0, 60.0),
                        fill: 0x10B981FF,
                        name: "Database Beta",
                    },
                    DemoObject {
                        bounds: WorldRect::from_xywh(-60.0, 40.0, 120.0, 60.0),
                        fill: 0xF59E0BFF,
                        name: "Cache Gamma",
                    },
                    DemoObject {
                        bounds: WorldRect::from_xywh(-280.0, 60.0, 80.0, 80.0),
                        fill: 0xEF4444FF,
                        name: "Gateway Delta",
                    },
                    DemoObject {
                        bounds: WorldRect::from_xywh(200.0, 20.0, 100.0, 100.0),
                        fill: 0x8B5CF6FF,
                        name: "Worker Epsilon",
                    },
                ];

                let config = CanvasConfig::default();
                let (response, mut painter) = canvas_begin(
                    &mut state.core,
                    &state.layout,
                    state.page.main,
                    &mut state.canvas_state,
                    &config,
                );

                let canvas_input = CanvasInput::new(&state.core.input, &response.screen_to_world);

                let theme = &AKAR_THEME_DARK;
                let item_style = DataItemStyle::from_theme(theme);

                for obj in &objects {
                    if !is_visible_world(response.visible_world_rect, obj.bounds) {
                        continue;
                    }

                    let lod = response.lod_index(obj.bounds, &LOD_THRESHOLDS);

                    match lod {
                        0 => {
                            let hovered = canvas_input.is_hovering(obj.bounds);
                            let center = Vec2::new(
                                (obj.bounds.min.x + obj.bounds.max.x) * 0.5,
                                (obj.bounds.min.y + obj.bounds.max.y) * 0.5,
                            );
                            let dot_size = 4.0;
                            let dot = WorldRect::from_xywh(
                                center.x - dot_size * 0.5,
                                center.y - dot_size * 0.5,
                                dot_size,
                                dot_size,
                            );
                            let fill = if hovered { 0xFFFFFFFF } else { obj.fill };
                            painter.push_quad(dot, fill, 0x00000000, 0.0, [2.0; 4], 0.0);
                        }
                        1 => {
                            canvas_data_item(
                                &mut painter,
                                &canvas_input,
                                obj.bounds,
                                &CanvasDataItemDescriptor {
                                    title: Some(obj.name),
                                    supporting_text: None,
                                    metadata: None,
                                    style: &item_style,
                                },
                            );
                        }
                        2 => {
                            canvas_data_item(
                                &mut painter,
                                &canvas_input,
                                obj.bounds,
                                &CanvasDataItemDescriptor {
                                    title: Some(obj.name),
                                    supporting_text: Some("Preview"),
                                    metadata: None,
                                    style: &item_style,
                                },
                            );
                        }
                        _ => {
                            painter.push_quad(obj.bounds, obj.fill, 0x00000000, 0.0, [8.0; 4], 0.0);
                        }
                    }
                }

                canvas_end(&mut state.core, painter);

                let objects_for_portal = [
                    DemoObject {
                        bounds: WorldRect::from_xywh(-180.0, -80.0, 120.0, 60.0),
                        fill: 0x3B82F6FF,
                        name: "Server Alpha",
                    },
                    DemoObject {
                        bounds: WorldRect::from_xywh(80.0, -80.0, 120.0, 60.0),
                        fill: 0x10B981FF,
                        name: "Database Beta",
                    },
                    DemoObject {
                        bounds: WorldRect::from_xywh(-60.0, 40.0, 120.0, 60.0),
                        fill: 0xF59E0BFF,
                        name: "Cache Gamma",
                    },
                    DemoObject {
                        bounds: WorldRect::from_xywh(-280.0, 60.0, 80.0, 80.0),
                        fill: 0xEF4444FF,
                        name: "Gateway Delta",
                    },
                    DemoObject {
                        bounds: WorldRect::from_xywh(200.0, 20.0, 100.0, 100.0),
                        fill: 0x8B5CF6FF,
                        name: "Worker Epsilon",
                    },
                ];

                let (core, portal_layouts, object_states) = (
                    &mut state.core,
                    &mut state.portal_layouts,
                    &mut state.object_states,
                );

                for (i, obj) in objects_for_portal.iter().enumerate() {
                    if !is_visible_world(response.visible_world_rect, obj.bounds) {
                        continue;
                    }

                    let lod = response.lod_index(obj.bounds, &LOD_THRESHOLDS);
                    if lod < 3 {
                        continue;
                    }

                    let projected = response.project(obj.bounds);
                    if !projected.visible {
                        continue;
                    }

                    let sr = projected.screen_rect;
                    let portal_w = sr[2];
                    let portal_h = sr[3];
                    if portal_w < 1.0 || portal_h < 1.0 {
                        continue;
                    }

                    let portal = portal_layouts.entry(i).or_insert_with(|| {
                        let mut pl = Layout::new();
                        pl.set_namespace_id(i as u64 + 1);

                        let root = pl.new_with_children(
                            Style {
                                display: akar_layout::Display::Flex,
                                flex_direction: akar_layout::FlexDirection::Column,
                                ..Default::default()
                            },
                            &[],
                        );

                        let item_node = pl.new_with_children(
                            Style {
                                display: akar_layout::Display::Flex,
                                flex_direction: akar_layout::FlexDirection::Column,
                                size: Size {
                                    width: akar_layout::Dimension::percent(1.0),
                                    height: akar_layout::Dimension::percent(1.0),
                                },
                                ..Default::default()
                            },
                            &[],
                        );

                        let label_node = pl.new_leaf(Style {
                            size: Size {
                                width: akar_layout::Dimension::percent(1.0),
                                height: length(24.0),
                            },
                            ..Default::default()
                        });

                        let button_node = pl.new_leaf(Style {
                            size: Size {
                                width: akar_layout::Dimension::percent(1.0),
                                height: length(32.0),
                            },
                            ..Default::default()
                        });

                        let input_node = pl.new_leaf(Style {
                            size: Size {
                                width: akar_layout::Dimension::percent(1.0),
                                height: length(32.0),
                            },
                            ..Default::default()
                        });

                        pl.set_children(item_node, &[label_node, button_node]);
                        pl.set_children(root, &[item_node, input_node]);

                        PortalLayout {
                            layout: pl,
                            root,
                            item_node,
                            label_node,
                            button_node,
                            input_node,
                        }
                    });

                    portal.layout.set_screen_origin([sr[0], sr[1]]);
                    portal.layout.set_style(
                        portal.root,
                        Style {
                            display: akar_layout::Display::Flex,
                            flex_direction: akar_layout::FlexDirection::Column,
                            size: Size {
                                width: length(portal_w),
                                height: length(portal_h),
                            },
                            padding: akar_layout::Rect {
                                top: length(8.0),
                                right: length(8.0),
                                bottom: length(8.0),
                                left: length(8.0),
                            },
                            ..Default::default()
                        },
                    );

                    portal.layout.compute(
                        portal.root,
                        (Some(portal_w), Some(portal_h)),
                        |_, _, _, _, _| Size::ZERO,
                    );

                    let guard = canvas_portal_begin(core, &response, &portal.layout, portal.root);

                    akar_container(core, &portal.layout, portal.root, &BoxStyle::panel(theme));

                    let item_response = akar_data_item(
                        core,
                        &portal.layout,
                        portal.item_node,
                        i as u64,
                        &DataItemStyle::from_theme(theme),
                    );
                    if item_response.clicked {
                        object_states[i].button_clicked = !object_states[i].button_clicked;
                    }

                    akar_label(
                        core,
                        &portal.layout,
                        portal.label_node,
                        obj.name,
                        theme.base_content,
                        theme,
                    );

                    let _button_id = portal
                        .layout
                        .widget_id_keyed(portal.button_node, i as u64 + 1);
                    let btn = akar_button(
                        core,
                        &portal.layout,
                        portal.button_node,
                        if object_states[i].button_clicked {
                            "Clicked!"
                        } else {
                            "Click me"
                        },
                        ButtonVariant::Solid,
                        theme,
                    );
                    if btn.clicked {
                        object_states[i].button_clicked = !object_states[i].button_clicked;
                    }

                    let obj_state = &mut object_states[i];
                    let _ = akar_text_input(
                        core,
                        &portal.layout,
                        portal.input_node,
                        &mut obj_state.text_value,
                        &mut obj_state.text_cursor,
                        "Type here...",
                        true,
                        theme,
                    );

                    canvas_portal_end(core, guard);
                }

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
