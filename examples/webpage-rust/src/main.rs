use std::sync::Arc;
use std::time::Instant;

use akar_core::AkarCore;
use akar_layout::{
    length, Dimension, Display, FlexDirection, JustifyContent, Layout, Size, Style,
};
use akar_winit::process_window_event;
use wgpu::{
    CompositeAlphaMode, CurrentSurfaceTexture, InstanceDescriptor, PresentMode, TextureUsages,
};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes},
};

const THEME_BG: u32 = 0xfafafaff;
const THEME_TEXT: u32 = 0x09090bff;
const THEME_TEXT_SECONDARY: u32 = 0x52525bff;
const THEME_BORDER: u32 = 0xe4e4e7ff;
const THEME_HERO_BG: u32 = 0xf4f4f5ff;
const THEME_CARD_BG: u32 = 0xffffffff;
const THEME_PATTERN: u32 = 0xd4d4d8ff;

fn hex_to_f4(c: u32) -> [f32; 4] {
    [
        ((c >> 24) & 0xFF) as f32 / 255.0,
        ((c >> 16) & 0xFF) as f32 / 255.0,
        ((c >> 8) & 0xFF) as f32 / 255.0,
        (c & 0xFF) as f32 / 255.0,
    ]
}

struct AppState {
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    core: AkarCore,
    layout: Layout,
    root: akar_layout::NodeId,
    header_node: akar_layout::NodeId,
    hero_node: akar_layout::NodeId,
    card_nodes: [akar_layout::NodeId; 3],
    build_section: akar_layout::NodeId,
    paper_section: akar_layout::NodeId,
}

enum Site {
    Mimo,
}

fn main() {
    let mut site_name: Option<String> = None;
    let mut screenshot_path = None;
    let mut exit_after = false;
    let mut delay_secs = 5.0;

    let mut args = std::env::args().peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--site" => {
                site_name = args.next();
            }
            "--screenshot" => {
                screenshot_path = args.next();
            }
            "--exit" => {
                exit_after = true;
            }
            "--delay" => {
                if let Some(secs) = args.next() {
                    if let Ok(parsed) = secs.parse::<f64>() {
                        delay_secs = parsed;
                    }
                }
            }
            _ => {}
        }
    }

    let site = match site_name.as_deref() {
        Some("mimo") => Site::Mimo,
        Some(other) => {
            eprintln!("Unknown site '{other}'. Valid sites: mimo");
            std::process::exit(1);
        }
        None => {
            eprintln!("Usage: webpage-rust --site <NAME>");
            eprintln!("Available sites: mimo");
            std::process::exit(1);
        }
    };

    let event_loop = EventLoop::new().unwrap();
    event_loop
        .run_app(&mut App {
            state: None,
            site,
            screenshot_path,
            exit_after,
            delay_secs,
            start_time: None,
            screenshot_taken: false,
        })
        .unwrap();
}

#[allow(dead_code)]
struct App {
    state: Option<AppState>,
    site: Site,
    screenshot_path: Option<String>,
    exit_after: bool,
    delay_secs: f64,
    start_time: Option<Instant>,
    screenshot_taken: bool,
}

fn build_mimo_layout(layout: &mut Layout) -> (
    akar_layout::NodeId,
    akar_layout::NodeId,
    akar_layout::NodeId,
    [akar_layout::NodeId; 3],
    akar_layout::NodeId,
    akar_layout::NodeId,
) {
    let root = layout.new_leaf(Style {
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        size: Size {
            width: Dimension::percent(1.0),
            height: Dimension::percent(1.0),
        },
        padding: taffy::geometry::Rect {
            left: length(48.0f32),
            right: length(48.0f32),
            top: length(0.0f32),
            bottom: length(0.0f32),
        },
        ..Default::default()
    });

    let header_node = layout.new_leaf(Style {
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        align_items: Some(akar_layout::AlignItems::CENTER),
        justify_content: Some(JustifyContent::SPACE_BETWEEN),
        flex_shrink: 0.0,
        size: Size {
            width: Dimension::percent(1.0),
            height: length(64.0f32),
        },
        padding: taffy::geometry::Rect {
            left: length(0.0f32),
            right: length(0.0f32),
            top: length(12.0f32),
            bottom: length(12.0f32),
        },
        ..Default::default()
    });

    let hero_node = layout.new_leaf(Style {
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        align_items: Some(akar_layout::AlignItems::CENTER),
        justify_content: Some(JustifyContent::CENTER),
        flex_shrink: 0.0,
        size: Size {
            width: Dimension::percent(1.0),
            height: length(480.0f32),
        },
        ..Default::default()
    });

    let card_style = || Style {
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        flex_grow: 1.0,
        size: Size {
            width: Dimension::auto(),
            height: length(220.0f32),
        },
        padding: taffy::geometry::Rect {
            left: length(20.0f32),
            right: length(20.0f32),
            top: length(20.0f32),
            bottom: length(20.0f32),
        },
        ..Default::default()
    };

    let card_nodes = [
        layout.new_leaf(card_style()),
        layout.new_leaf(card_style()),
        layout.new_leaf(card_style()),
    ];

    let cards_row = layout.new_with_children(
        Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            flex_shrink: 0.0,
            size: Size {
                width: Dimension::percent(1.0),
                height: length(220.0f32),
            },
            gap: taffy::geometry::Size {
                width: length(24.0f32),
                height: length(0.0f32),
            },
            margin: taffy::geometry::Rect {
                top: length(0.0f32),
                right: length(0.0f32),
                bottom: length(32.0f32),
                left: length(0.0f32),
            },
            ..Default::default()
        },
        &card_nodes,
    );

    let build_section = layout.new_leaf(Style {
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        flex_shrink: 0.0,
        size: Size {
            width: Dimension::percent(1.0),
            height: Dimension::auto(),
        },
        margin: taffy::geometry::Rect {
            top: length(0.0f32),
            right: length(0.0f32),
            bottom: length(24.0f32),
            left: length(0.0f32),
        },
        ..Default::default()
    });

    let paper_section = layout.new_leaf(Style {
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        flex_shrink: 0.0,
        size: Size {
            width: Dimension::percent(1.0),
            height: Dimension::auto(),
        },
        margin: taffy::geometry::Rect {
            top: length(16.0f32),
            right: length(0.0f32),
            bottom: length(0.0f32),
            left: length(0.0f32),
        },
        ..Default::default()
    });

    layout.set_children(root, &[header_node, hero_node, cards_row, build_section, paper_section]);

    (root, header_node, hero_node, card_nodes, build_section, paper_section)
}

fn render_mimo_header(state: &mut AppState, header_rect: [f32; 4]) {
    let bg = hex_to_f4(THEME_BG);
    state.core.draw_list.push_quad(akar_core::QuadCall {
        rect: header_rect,
        fill: bg,
        border_color: [0.0; 4],
        corner_radii: [0.0; 4],
        border_width: 0.0,
        z: 0.0,
        shadow_blur: 0.0,
        shadow_spread: 0.0,
        shadow_color: [0.0; 4],
        shadow_offset: [0.0; 2],
        _pad: [0.0; 2],
    });

    let logo_buf = state.core.text_pipeline.set_text(
        Some(100),
        "Xiaomi MiMo",
        glyphon::Metrics::new(20.0, 20.0 * 1.3),
        None,
        None,
    );
    state.core.draw_list.push_text(akar_core::TextCall {
        buffer_id: logo_buf,
        x: header_rect[0],
        y: header_rect[1] + 14.0,
        clip: header_rect,
        color: hex_to_f4(THEME_TEXT),
        z: 0.0,
    });

    let nav_items = ["Product", "Research", "Join Us"];
    let mut x_offset = header_rect[0] + header_rect[2] - 20.0;
    for (i, item) in nav_items.iter().rev().enumerate() {
        let buf = state.core.text_pipeline.set_text(
            Some(200 + i as u64),
            item,
            glyphon::Metrics::new(15.0, 15.0 * 1.4),
            None,
            None,
        );
        let measured = state.core.text_pipeline.measure(buf, None);
        let text_w = measured.x;
        x_offset -= text_w;
        state.core.draw_list.push_text(akar_core::TextCall {
            buffer_id: buf,
            x: x_offset,
            y: header_rect[1] + 18.0,
            clip: header_rect,
            color: hex_to_f4(THEME_TEXT),
            z: 0.0,
        });
        x_offset -= 32.0;
    }
}

fn render_mimo_hero(state: &mut AppState, hero_rect: [f32; 4]) {
    let bg = hex_to_f4(THEME_HERO_BG);
    state.core.draw_list.push_quad(akar_core::QuadCall {
        rect: hero_rect,
        fill: bg,
        border_color: hex_to_f4(THEME_BORDER),
        corner_radii: [0.0; 4],
        border_width: 1.0,
        z: 0.0,
        shadow_blur: 0.0,
        shadow_spread: 0.0,
        shadow_color: [0.0; 4],
        shadow_offset: [0.0; 2],
        _pad: [0.0; 2],
    });

    let pattern_color = hex_to_f4(THEME_PATTERN);
    let pattern_text = "M I M O M I M O M I M O M I M O M I M O";
    let row_height = 52.0;
    let start_y = hero_rect[1] + 20.0;
    let mut row = 0u64;
    let mut y = start_y;
    while y < hero_rect[1] + hero_rect[3] - 20.0 {
        let x_offset = if row % 2 == 0 { 0.0 } else { -40.0 };
        let buf = state.core.text_pipeline.set_text(
            Some(3000 + row),
            pattern_text,
            glyphon::Metrics::new(28.0, 28.0 * 1.2),
            Some(hero_rect[2] + 80.0),
            None,
        );
        state.core.draw_list.push_text(akar_core::TextCall {
            buffer_id: buf,
            x: hero_rect[0] + x_offset,
            y,
            clip: hero_rect,
            color: pattern_color,
            z: 0.0,
        });
        y += row_height;
        row += 1;
    }

    let title_buf = state.core.text_pipeline.set_text(
        Some(4000),
        "HELLO, I'M MiMo",
        glyphon::Metrics::new(72.0, 72.0 * 1.1),
        None,
        None,
    );
    let measured = state.core.text_pipeline.measure(title_buf, None);
    let title_w = measured.x;
    let title_h = 80.0;
    let title_x = hero_rect[0] + (hero_rect[2] - title_w) / 2.0;
    let title_y = hero_rect[1] + (hero_rect[3] - title_h) / 2.0;
    state.core.draw_list.push_text(akar_core::TextCall {
        buffer_id: title_buf,
        x: title_x,
        y: title_y,
        clip: hero_rect,
        color: hex_to_f4(THEME_TEXT),
        z: 0.1,
    });
}

fn render_mimo_card(core: &mut AkarCore, card_rect: [f32; 4], index: usize) {
    let bg = hex_to_f4(THEME_CARD_BG);
    core.draw_list.push_quad(akar_core::QuadCall {
        rect: card_rect,
        fill: bg,
        border_color: hex_to_f4(THEME_BORDER),
        corner_radii: [0.0; 4],
        border_width: 1.0,
        z: 0.0,
        shadow_blur: 0.0,
        shadow_spread: 0.0,
        shadow_color: [0.0; 4],
        shadow_offset: [0.0; 2],
        _pad: [0.0; 2],
    });

    let (title, subtitle) = match index {
        0 => ("Xiaomi MiMo-V2.5-Pro", "A leap in agentic and long horizon coherence."),
        1 => ("Xiaomi MiMo-V2.5", "A leap in agency and multimodality."),
        2 => (
            "Xiaomi MiMo-V2.5-TTS Series",
            "Give your agent a voice. Give it a soul.",
        ),
        _ => ("", ""),
    };

    let title_buf = core.text_pipeline.set_text(
        Some(5000 + index as u64),
        title,
        glyphon::Metrics::new(20.0, 20.0 * 1.3),
        Some(card_rect[2] - 40.0),
        None,
    );
    core.draw_list.push_text(akar_core::TextCall {
        buffer_id: title_buf,
        x: card_rect[0] + 20.0,
        y: card_rect[1] + 24.0,
        clip: card_rect,
        color: hex_to_f4(THEME_TEXT),
        z: 0.0,
    });

    let sub_buf = core.text_pipeline.set_text(
        Some(5100 + index as u64),
        subtitle,
        glyphon::Metrics::new(14.0, 14.0 * 1.4),
        Some(card_rect[2] - 40.0),
        None,
    );
    core.draw_list.push_text(akar_core::TextCall {
        buffer_id: sub_buf,
        x: card_rect[0] + 20.0,
        y: card_rect[1] + 56.0,
        clip: card_rect,
        color: hex_to_f4(THEME_TEXT_SECONDARY),
        z: 0.0,
    });

    let wave_color = hex_to_f4(THEME_PATTERN);
    let wave_y_base = card_rect[1] + card_rect[3] - 60.0;
    for w in 0..3 {
        let wave_rect = [
            card_rect[0] + 20.0 + w as f32 * 50.0,
            wave_y_base + (w as f32 * 8.0).sin() * 10.0,
            60.0,
            40.0,
        ];
        core.draw_list.push_quad(akar_core::QuadCall {
            rect: wave_rect,
            fill: [wave_color[0], wave_color[1], wave_color[2], 0.3],
            border_color: [0.0; 4],
            corner_radii: [20.0; 4],
            border_width: 0.0,
            z: 0.0,
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            shadow_color: [0.0; 4],
            shadow_offset: [0.0; 2],
            _pad: [0.0; 2],
        });
    }

    if index == 1 {
        let dot_rect = [
            card_rect[0] + card_rect[2] / 2.0 - 15.0,
            card_rect[1] + card_rect[3] - 50.0,
            30.0,
            30.0,
        ];
        core.draw_list.push_quad(akar_core::QuadCall {
            rect: dot_rect,
            fill: [wave_color[0], wave_color[1], wave_color[2], 0.4],
            border_color: [0.0; 4],
            corner_radii: [15.0; 4],
            border_width: 0.0,
            z: 0.0,
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            shadow_color: [0.0; 4],
            shadow_offset: [0.0; 2],
            _pad: [0.0; 2],
        });
    }
}

fn render_mimo_build_section(core: &mut AkarCore, section_rect: [f32; 4]) {
    let border_color = hex_to_f4(THEME_BORDER);

    core.draw_list.push_quad(akar_core::QuadCall {
        rect: section_rect,
        fill: hex_to_f4(THEME_BG),
        border_color,
        corner_radii: [0.0; 4],
        border_width: 1.0,
        z: 0.0,
        shadow_blur: 0.0,
        shadow_spread: 0.0,
        shadow_color: [0.0; 4],
        shadow_offset: [0.0; 2],
        _pad: [0.0; 2],
    });

    let heading_buf = core.text_pipeline.set_text(
        Some(6000),
        "Build with MiMo",
        glyphon::Metrics::new(42.0, 42.0 * 1.2),
        None,
        None,
    );
    core.draw_list.push_text(akar_core::TextCall {
        buffer_id: heading_buf,
        x: section_rect[0] + 48.0,
        y: section_rect[1] + 48.0,
        clip: section_rect,
        color: hex_to_f4(THEME_TEXT),
        z: 0.0,
    });

    let desc_buf = core.text_pipeline.set_text(
        Some(6001),
        "Experience the powerful capabilities of Xiaomi MiMo's large-scale model\nnow and explore the infinite possibilities of AI.",
        glyphon::Metrics::new(16.0, 16.0 * 1.5),
        Some(400.0),
        None,
    );
    core.draw_list.push_text(akar_core::TextCall {
        buffer_id: desc_buf,
        x: section_rect[0] + section_rect[2] * 0.45,
        y: section_rect[1] + 56.0,
        clip: section_rect,
        color: hex_to_f4(THEME_TEXT_SECONDARY),
        z: 0.0,
    });

    let items = [
        ("01", "Web Demo", "Interact with MiMo directly through the web"),
        (
            "02",
            "API Access",
            "Developer portal for quick integration of MiMo capabilities",
        ),
    ];

    let row_height = 80.0;
    let content_top = section_rect[1] + 120.0;

    for (i, (num, title, desc)) in items.iter().enumerate() {
        let row_y = content_top + i as f32 * row_height;

        core.draw_list.push_quad(akar_core::QuadCall {
            rect: [section_rect[0], row_y, section_rect[2], row_height],
            fill: hex_to_f4(THEME_BG),
            border_color,
            corner_radii: [0.0; 4],
            border_width: 1.0,
            z: 0.0,
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            shadow_color: [0.0; 4],
            shadow_offset: [0.0; 2],
            _pad: [0.0; 2],
        });

        let num_buf = core.text_pipeline.set_text(
            Some(6100 + i as u64),
            num,
            glyphon::Metrics::new(18.0, 18.0 * 1.3),
            Some(60.0),
            None,
        );
        core.draw_list.push_text(akar_core::TextCall {
            buffer_id: num_buf,
            x: section_rect[0] + 48.0,
            y: row_y + 20.0,
            clip: section_rect,
            color: hex_to_f4(THEME_TEXT_SECONDARY),
            z: 0.0,
        });

        let title_buf = core.text_pipeline.set_text(
            Some(6200 + i as u64),
            title,
            glyphon::Metrics::new(20.0, 20.0 * 1.3),
            Some(section_rect[2] * 0.5),
            None,
        );
        core.draw_list.push_text(akar_core::TextCall {
            buffer_id: title_buf,
            x: section_rect[0] + 120.0,
            y: row_y + 16.0,
            clip: section_rect,
            color: hex_to_f4(THEME_TEXT),
            z: 0.0,
        });

        let desc_buf = core.text_pipeline.set_text(
            Some(6300 + i as u64),
            desc,
            glyphon::Metrics::new(14.0, 14.0 * 1.4),
            Some(section_rect[2] * 0.5),
            None,
        );
        core.draw_list.push_text(akar_core::TextCall {
            buffer_id: desc_buf,
            x: section_rect[0] + 120.0,
            y: row_y + 44.0,
            clip: section_rect,
            color: hex_to_f4(THEME_TEXT_SECONDARY),
            z: 0.0,
        });

        let arrow_buf = core.text_pipeline.set_text(
            Some(6400 + i as u64),
            "\u{2192}",
            glyphon::Metrics::new(24.0, 24.0 * 1.3),
            None,
            None,
        );
        core.draw_list.push_text(akar_core::TextCall {
            buffer_id: arrow_buf,
            x: section_rect[0] + section_rect[2] - 60.0,
            y: row_y + 24.0,
            clip: section_rect,
            color: hex_to_f4(THEME_TEXT),
            z: 0.0,
        });
    }
}

fn render_mimo_paper_section(core: &mut AkarCore, section_rect: [f32; 4]) {
    let border_color = hex_to_f4(THEME_BORDER);

    core.draw_list.push_quad(akar_core::QuadCall {
        rect: section_rect,
        fill: hex_to_f4(THEME_BG),
        border_color,
        corner_radii: [0.0; 4],
        border_width: 1.0,
        z: 0.0,
        shadow_blur: 0.0,
        shadow_spread: 0.0,
        shadow_color: [0.0; 4],
        shadow_offset: [0.0; 2],
        _pad: [0.0; 2],
    });

    let heading_buf = core.text_pipeline.set_text(
        Some(7000),
        "Paper",
        glyphon::Metrics::new(42.0, 42.0 * 1.2),
        None,
        None,
    );
    core.draw_list.push_text(akar_core::TextCall {
        buffer_id: heading_buf,
        x: section_rect[0] + 48.0,
        y: section_rect[1] + 48.0,
        clip: section_rect,
        color: hex_to_f4(THEME_TEXT),
        z: 0.0,
    });

    let items = [(
        "01",
        "MOPD: Multi-Teacher On-Policy Distillation for Capability Integration in LLM Post-Training",
        "June 29, 2026",
    )];

    let row_height = 100.0;
    let content_top = section_rect[1] + 110.0;

    for (i, (num, title, date)) in items.iter().enumerate() {
        let row_y = content_top + i as f32 * row_height;

        core.draw_list.push_quad(akar_core::QuadCall {
            rect: [section_rect[0], row_y, section_rect[2], row_height],
            fill: hex_to_f4(THEME_BG),
            border_color,
            corner_radii: [0.0; 4],
            border_width: 1.0,
            z: 0.0,
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            shadow_color: [0.0; 4],
            shadow_offset: [0.0; 2],
            _pad: [0.0; 2],
        });

        let num_buf = core.text_pipeline.set_text(
            Some(7100 + i as u64),
            num,
            glyphon::Metrics::new(18.0, 18.0 * 1.3),
            Some(60.0),
            None,
        );
        core.draw_list.push_text(akar_core::TextCall {
            buffer_id: num_buf,
            x: section_rect[0] + 48.0,
            y: row_y + 28.0,
            clip: section_rect,
            color: hex_to_f4(THEME_TEXT_SECONDARY),
            z: 0.0,
        });

        let title_buf = core.text_pipeline.set_text(
            Some(7200 + i as u64),
            title,
            glyphon::Metrics::new(20.0, 20.0 * 1.3),
            Some(section_rect[2] * 0.6),
            None,
        );
        core.draw_list.push_text(akar_core::TextCall {
            buffer_id: title_buf,
            x: section_rect[0] + 120.0,
            y: row_y + 18.0,
            clip: section_rect,
            color: hex_to_f4(THEME_TEXT),
            z: 0.0,
        });

        let date_buf = core.text_pipeline.set_text(
            Some(7300 + i as u64),
            date,
            glyphon::Metrics::new(13.0, 13.0 * 1.4),
            Some(section_rect[2] * 0.6),
            None,
        );
        core.draw_list.push_text(akar_core::TextCall {
            buffer_id: date_buf,
            x: section_rect[0] + 120.0,
            y: row_y + 48.0,
            clip: section_rect,
            color: hex_to_f4(THEME_TEXT_SECONDARY),
            z: 0.0,
        });

        let arrow_buf = core.text_pipeline.set_text(
            Some(7400 + i as u64),
            "\u{2192}",
            glyphon::Metrics::new(24.0, 24.0 * 1.3),
            None,
            None,
        );
        core.draw_list.push_text(akar_core::TextCall {
            buffer_id: arrow_buf,
            x: section_rect[0] + section_rect[2] - 60.0,
            y: row_y + 32.0,
            clip: section_rect,
            color: hex_to_f4(THEME_TEXT),
            z: 0.0,
        });
    }
}

fn prepare_layout(state: &mut AppState, size: PhysicalSize<u32>, scale: f32) {
    state.layout.compute(
        state.root,
        (
            Some(size.width as f32 / scale),
            Some(size.height as f32 / scale),
        ),
        |_, _, _, _, _| Size::ZERO,
    );
}

fn render_all(state: &mut AppState, viewport_rect: [f32; 4]) {
    let bg = hex_to_f4(THEME_BG);
    state.core.draw_list.push_quad(akar_core::QuadCall {
        rect: [0.0, 0.0, viewport_rect[2], viewport_rect[3]],
        fill: bg,
        border_color: [0.0; 4],
        corner_radii: [0.0; 4],
        border_width: 0.0,
        z: -1.0,
        shadow_blur: 0.0,
        shadow_spread: 0.0,
        shadow_color: [0.0; 4],
        shadow_offset: [0.0; 2],
        _pad: [0.0; 2],
    });

    let header_rect = state.layout.rect(state.header_node);
    render_mimo_header(state, header_rect);

    let hero_rect = state.layout.rect(state.hero_node);
    render_mimo_hero(state, hero_rect);

    let card_rects: Vec<[f32; 4]> = state.card_nodes.iter().map(|&n| state.layout.rect(n)).collect();
    for (i, card_rect) in card_rects.into_iter().enumerate() {
        render_mimo_card(&mut state.core, card_rect, i);
    }

    let build_rect = state.layout.rect(state.build_section);
    render_mimo_build_section(&mut state.core, build_rect);

    let paper_rect = state.layout.rect(state.paper_section);
    render_mimo_paper_section(&mut state.core, paper_rect);
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        let window_attrs = WindowAttributes::default()
            .with_title("akar webpage demo")
            .with_inner_size(LogicalSize::new(1280.0, 900.0));
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

        let (root, header_node, hero_node, card_nodes, build_section, paper_section) =
            build_mimo_layout(&mut layout);

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
            root,
            header_node,
            hero_node,
            card_nodes,
            build_section,
            paper_section,
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

                render_all(state, viewport_rect);

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
            }
            _ => {}
        }

        process_window_event(&mut state.core.input, &event);

        if !matches!(event, WindowEvent::RedrawRequested) {
            state.window.request_redraw();
        }
    }
}
