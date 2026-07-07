use std::sync::Arc;

use akar_components::{
    akar_alert, akar_avatar, akar_badge, akar_button, akar_container, akar_label, akar_navbar,
    akar_skeleton, akar_stat, akar_steps, progress_at, AlertVariant, BadgeVariant, BoxStyle,
    ButtonVariant, NavbarSlots, ProgressStyle, SkeletonVariant, AKAR_THEME_DARK,
};
use akar_components::{scroll_area_begin, scroll_area_end};
use akar_core::list_clip;
use akar_core::AkarCore;
use akar_layout::{length, Dimension, Display, FlexDirection, Layout, PageConfig, Size, Style};
use akar_winit::process_window_event;
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

struct AppState {
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    core: AkarCore,
    layout: Layout,
    page: akar_layout::PageLayout,
    two_col: akar_layout::TwoColumnLayout,
    scroll_y: f32,
    scroll_container: akar_layout::NodeId,
    navbar_slots: Option<NavbarSlots>,
    navbar_title_node: akar_layout::NodeId,
    navbar_badge_node: akar_layout::NodeId,
    navbar_btn_node: akar_layout::NodeId,
    alert_node: akar_layout::NodeId,
    alert_dismissed: bool,
    stat_nodes: [akar_layout::NodeId; 3],
    steps_node: akar_layout::NodeId,
    avatar_nodes: [akar_layout::NodeId; 3],
    skeleton_toggle_node: akar_layout::NodeId,
    show_skeleton: bool,
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
            .with_title("akar demo")
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
            sidebar_left_width: Some(200.0),
            sidebar_right_width: None,
        });

        let two_col = layout.two_column(page.main, 0.5, 1.0);

        layout.set_style(
            two_col.right,
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                gap: taffy::geometry::Size {
                    width: length(0.0),
                    height: length(8.0),
                },
                ..Default::default()
            },
        );

        let alert_node = layout.new_leaf(Style {
            flex_shrink: 0.0,
            size: Size {
                width: Dimension::percent(1.0),
                height: length(48.0),
            },
            ..Default::default()
        });

        let stat_row = layout.new_leaf(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            flex_shrink: 0.0,
            size: Size {
                width: Dimension::percent(1.0),
                height: length(100.0),
            },
            gap: taffy::geometry::Size {
                width: length(8.0),
                height: length(0.0),
            },
            padding: taffy::geometry::Rect {
                left: length(8.0),
                right: length(8.0),
                top: length(4.0),
                bottom: length(4.0),
            },
            ..Default::default()
        });

        let stat_1 = layout.new_leaf(Style {
            flex_grow: 1.0,
            size: Size {
                width: Dimension::auto(),
                height: Dimension::percent(1.0),
            },
            ..Default::default()
        });
        let stat_2 = layout.new_leaf(Style {
            flex_grow: 1.0,
            size: Size {
                width: Dimension::auto(),
                height: Dimension::percent(1.0),
            },
            ..Default::default()
        });
        let stat_3 = layout.new_leaf(Style {
            flex_grow: 1.0,
            size: Size {
                width: Dimension::auto(),
                height: Dimension::percent(1.0),
            },
            ..Default::default()
        });
        let stat_nodes = [stat_1, stat_2, stat_3];
        for &n in &stat_nodes {
            layout.add_child(stat_row, n);
        }

        let steps_node = layout.new_leaf(Style {
            flex_shrink: 0.0,
            size: Size {
                width: Dimension::percent(1.0),
                height: length(56.0),
            },
            ..Default::default()
        });

        let avatar_row = layout.new_leaf(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            flex_shrink: 0.0,
            align_items: Some(taffy::prelude::AlignItems::CENTER),
            size: Size {
                width: Dimension::percent(1.0),
                height: length(56.0),
            },
            gap: taffy::geometry::Size {
                width: length(8.0),
                height: length(0.0),
            },
            padding: taffy::geometry::Rect {
                left: length(8.0),
                right: length(0.0),
                top: length(0.0),
                bottom: length(0.0),
            },
            ..Default::default()
        });

        let avatar_1 = layout.new_leaf(Style {
            flex_shrink: 0.0,
            size: Size {
                width: length(40.0),
                height: length(40.0),
            },
            ..Default::default()
        });
        let avatar_2 = layout.new_leaf(Style {
            flex_shrink: 0.0,
            size: Size {
                width: length(40.0),
                height: length(40.0),
            },
            ..Default::default()
        });
        let avatar_3 = layout.new_leaf(Style {
            flex_shrink: 0.0,
            size: Size {
                width: length(40.0),
                height: length(40.0),
            },
            ..Default::default()
        });
        let avatar_nodes = [avatar_1, avatar_2, avatar_3];
        for &n in &avatar_nodes {
            layout.add_child(avatar_row, n);
        }

        let skeleton_toggle_node = layout.new_leaf(Style {
            flex_shrink: 0.0,
            size: Size {
                width: length(140.0),
                height: length(32.0),
            },
            ..Default::default()
        });
        layout.add_child(avatar_row, skeleton_toggle_node);

        let scroll_container = layout.new_leaf(Style {
            flex_grow: 1.0,
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            overflow: taffy::geometry::Point {
                x: taffy::style::Overflow::Clip,
                y: taffy::style::Overflow::Clip,
            },
            size: Size {
                width: Dimension::percent(1.0),
                height: Dimension::auto(),
            },
            ..Default::default()
        });

        layout.add_child(two_col.right, alert_node);
        layout.add_child(two_col.right, stat_row);
        layout.add_child(two_col.right, steps_node);
        layout.add_child(two_col.right, avatar_row);
        layout.add_child(two_col.right, scroll_container);

        let navbar_title_node = layout.new_leaf(Style {
            flex_shrink: 0.0,
            size: Size {
                width: length(80.0),
                height: length(48.0),
            },
            ..Default::default()
        });
        let navbar_badge_node = layout.new_leaf(Style {
            flex_shrink: 0.0,
            size: Size {
                width: length(32.0),
                height: length(24.0),
            },
            ..Default::default()
        });
        let navbar_btn_node = layout.new_leaf(Style {
            flex_shrink: 0.0,
            size: Size {
                width: length(120.0),
                height: length(32.0),
            },
            ..Default::default()
        });

        self.state = Some(AppState {
            window,
            device,
            queue,
            surface,
            surface_config,
            core,
            layout,
            page,
            two_col,
            scroll_y: 0.0,
            scroll_container,
            navbar_slots: None,
            navbar_title_node,
            navbar_badge_node,
            navbar_btn_node,
            alert_node,
            alert_dismissed: false,
            stat_nodes,
            steps_node,
            avatar_nodes,
            skeleton_toggle_node,
            show_skeleton: false,
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

                let navbar_id = state.page.header.unwrap();

                if state.navbar_slots.is_none() {
                    let slots = akar_navbar(
                        &mut state.core,
                        &mut state.layout,
                        navbar_id,
                        &AKAR_THEME_DARK,
                    );
                    state.layout.add_child(slots.start, state.navbar_title_node);
                    state.layout.add_child(slots.end, state.navbar_badge_node);
                    state.layout.add_child(slots.end, state.navbar_btn_node);
                    state.navbar_slots = Some(slots);
                }

                if state.alert_dismissed {
                    state.layout.set_style(
                        state.alert_node,
                        Style {
                            display: Display::None,
                            ..Default::default()
                        },
                    );
                }

                state.layout.compute(
                    state.page.root,
                    (
                        Some(size.width as f32 / scale),
                        Some(size.height as f32 / scale),
                    ),
                    |_, _, _, _, _| Size::ZERO,
                );

                akar_label(
                    &mut state.core,
                    &state.layout,
                    state.navbar_title_node,
                    "akar",
                    AKAR_THEME_DARK.base_content,
                    &AKAR_THEME_DARK,
                );
                akar_badge(
                    &mut state.core,
                    &state.layout,
                    state.navbar_badge_node,
                    "3",
                    BadgeVariant::Primary,
                    &AKAR_THEME_DARK,
                );
                akar_button(
                    &mut state.core,
                    &state.layout,
                    state.navbar_btn_node,
                    "Notifications",
                    ButtonVariant::Ghost,
                    &AKAR_THEME_DARK,
                );

                akar_container(
                    &mut state.core,
                    &state.layout,
                    state.page.sidebar_left.unwrap(),
                    &BoxStyle::panel(&AKAR_THEME_DARK),
                );
                akar_container(
                    &mut state.core,
                    &state.layout,
                    state.page.main,
                    &BoxStyle::surface(&AKAR_THEME_DARK),
                );
                akar_container(
                    &mut state.core,
                    &state.layout,
                    state.two_col.left,
                    &BoxStyle::flat(0x172554ff),
                );
                akar_container(
                    &mut state.core,
                    &state.layout,
                    state.two_col.right,
                    &BoxStyle::flat(0x27272aff),
                );

                if !state.alert_dismissed {
                    let result = akar_alert(
                        &mut state.core,
                        &state.layout,
                        state.alert_node,
                        "Welcome to akar demo!",
                        AlertVariant::Info,
                        true,
                        &AKAR_THEME_DARK,
                    );
                    state.alert_dismissed = result.dismissed;
                }

                let stat_data = [
                    ("Revenue", "$12,345", Some("+12% vs last month")),
                    ("Users", "1,234", Some("+8% vs last month")),
                    ("Active", "89%", Some("+3% vs last month")),
                ];
                for (i, &(title, value, desc)) in stat_data.iter().enumerate() {
                    akar_stat(
                        &mut state.core,
                        &state.layout,
                        state.stat_nodes[i],
                        title,
                        value,
                        desc,
                        &AKAR_THEME_DARK,
                    );
                }

                akar_steps(
                    &mut state.core,
                    &state.layout,
                    state.steps_node,
                    &["Plan", "Build", "Test", "Launch"],
                    1,
                    &AKAR_THEME_DARK,
                );

                let avatar_initials = ["JD", "AK", "MR"];
                for (i, initials) in avatar_initials.iter().enumerate() {
                    if state.show_skeleton {
                        akar_skeleton(
                            &mut state.core,
                            &state.layout,
                            state.avatar_nodes[i],
                            SkeletonVariant::Circle,
                            &AKAR_THEME_DARK,
                        );
                    } else {
                        akar_avatar(
                            &mut state.core,
                            &state.layout,
                            state.avatar_nodes[i],
                            initials,
                            None,
                            &AKAR_THEME_DARK,
                        );
                    }
                }
                let toggle_label = if state.show_skeleton {
                    "Show Avatars"
                } else {
                    "Show Skeleton"
                };
                let toggle_result = akar_button(
                    &mut state.core,
                    &state.layout,
                    state.skeleton_toggle_node,
                    toggle_label,
                    ButtonVariant::Solid,
                    &AKAR_THEME_DARK,
                );
                if toggle_result.clicked {
                    state.show_skeleton = !state.show_skeleton;
                }

                let scroll_rect = state.layout.rect(state.scroll_container);
                let total_items = 50_usize;
                let item_height = 48.0_f32;
                let content_height = total_items as f32 * item_height;

                let resp = scroll_area_begin(
                    &mut state.core,
                    scroll_rect,
                    &mut state.scroll_y,
                    content_height,
                );
                let visible = list_clip(total_items, item_height, state.scroll_y, scroll_rect[3]);

                for i in visible {
                    let y = resp.content_y + i as f32 * item_height;
                    let item_rect = [scroll_rect[0], y, scroll_rect[2], item_height];
                    let inner_pad = 4.0_f32;
                    let inner_rect = [
                        item_rect[0] + inner_pad,
                        item_rect[1] + inner_pad,
                        item_rect[2] - 2.0 * inner_pad,
                        item_rect[3] - 2.0 * inner_pad,
                    ];

                    let item_bg = 0x1e293bffu32;
                    let item_border = 0x334155ffu32;

                    state.core.draw_list.push_quad(akar_core::QuadCall {
                        rect: item_rect,
                        fill: [
                            ((item_bg >> 24) & 0xFF) as f32 / 255.0,
                            ((item_bg >> 16) & 0xFF) as f32 / 255.0,
                            ((item_bg >> 8) & 0xFF) as f32 / 255.0,
                            (item_bg & 0xFF) as f32 / 255.0,
                        ],
                        border_color: [
                            ((item_border >> 24) & 0xFF) as f32 / 255.0,
                            ((item_border >> 16) & 0xFF) as f32 / 255.0,
                            ((item_border >> 8) & 0xFF) as f32 / 255.0,
                            (item_border & 0xFF) as f32 / 255.0,
                        ],
                        corner_radii: [6.0; 4],
                        border_width: 1.0,
                        z: 0.0,
                        shadow_blur: 0.0,
                        shadow_spread: 0.0,
                        shadow_color: [0.0; 4],
                        shadow_offset: [0.0; 2],
                        _pad: [0.0; 2],
                    });

                    let label_text = format!("Item {}", i + 1);
                    let buffer_id = state.core.text_pipeline.set_text(
                        Some(i as u64),
                        &label_text,
                        glyphon::Metrics::new(14.0, 14.0 * 1.2),
                        Some(inner_rect[2] * 0.6),
                        None,
                    );
                    state.core.draw_list.push_text(akar_core::TextCall {
                        buffer_id,
                        x: inner_rect[0],
                        y: inner_rect[1],
                        clip: inner_rect,
                        color: [0.98, 0.98, 0.98, 1.0],
                        z: 0.0,
                    });

                    let progress_value = (i + 1) as f32 / total_items as f32;
                    let progress_x = inner_rect[0] + inner_rect[2] * 0.65;
                    let progress_w = inner_rect[2] * 0.35;
                    let progress_h = 8.0;
                    let progress_y = inner_rect[1] + (inner_rect[3] - progress_h) / 2.0;
                    let progress_rect = [progress_x, progress_y, progress_w, progress_h];
                    let progress_style = ProgressStyle {
                        track_color: 0x27272aff,
                        fill_color: 0x3b82f6ff,
                        corner_radius: 4.0,
                    };
                    progress_at(
                        &mut state.core,
                        progress_rect,
                        progress_value,
                        &progress_style,
                    );
                }
                scroll_area_end(&mut state.core);

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

        if !matches!(event, WindowEvent::RedrawRequested) {
            state.window.request_redraw();
        }
    }
}
