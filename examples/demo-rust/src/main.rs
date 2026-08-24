use std::sync::Arc;
use std::time::Instant;

use akar_components::{
    akar_alert, akar_avatar, akar_badge, akar_button, akar_card, akar_card_layout, akar_checkbox,
    akar_container, akar_heading, akar_label, akar_link, akar_navbar_layout, akar_paragraph,
    akar_radio_group, akar_select, akar_skeleton, akar_slider, akar_stat, akar_steps, akar_switch,
    akar_tab_bar, akar_text_input, akar_textarea, akar_tooltip, drawer_begin, drawer_end,
    dropdown_begin, dropdown_end, modal_begin, modal_end, progress_at, toasts, AlertVariant,
    BadgeVariant, BoxStyle, ButtonVariant, CardLayout, CardSlots, CardStyle, DrawerEdge,
    FontFamily, HeadingLevel, LinkResult, NavbarSlots, ProgressStyle, SkeletonVariant, TabVariant,
    TextStyle, ToastItem, ToastVariant, TooltipSide, AKAR_THEME_DARK,
};
use akar_components::{scroll_area_begin, scroll_area_end};
use akar_core::list_clip;
use akar_core::AkarCore;
use akar_core::Z_FLOAT;
use akar_layout::{
    length, AkarDirection, AlignItems, AlignSelf, Dimension, Display, FlexDirection,
    JustifyContent, Layout, PageConfig, Size, Style,
};
use akar_winit::process_window_event;
use script::{parse_script, ScriptRunner};
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

mod catalog;
mod cli;
mod script;
mod showcase;

#[derive(serde::Serialize)]
struct InputSnapshot {
    mouse_pos: [f32; 2],
    mouse_buttons: [bool; 5],
    mouse_buttons_pressed: [bool; 5],
    mouse_buttons_released: [bool; 5],
    scroll_delta: [f32; 2],
    chars: Vec<String>,
    keys_pressed: Vec<String>,
    key_events: Vec<String>,
    modifiers: [bool; 4],
    focused_id: Option<u64>,
}

#[derive(serde::Serialize)]
struct FrameDump<'a> {
    recorded_calls: &'a [akar_core::draw_list::RecordedCall],
    labeled_rects: Vec<(String, [f32; 4])>,
    input: InputSnapshot,
}

fn input_snapshot(input: &akar_core::InputState) -> InputSnapshot {
    InputSnapshot {
        mouse_pos: [input.mouse_pos.x, input.mouse_pos.y],
        mouse_buttons: input.mouse_buttons,
        mouse_buttons_pressed: input.mouse_buttons_pressed,
        mouse_buttons_released: input.mouse_buttons_released,
        scroll_delta: [input.scroll_delta.x, input.scroll_delta.y],
        chars: input.chars.iter().map(|c| c.to_string()).collect(),
        keys_pressed: input
            .keys_pressed
            .iter()
            .map(|k| format!("{k:?}"))
            .collect(),
        key_events: input
            .key_events
            .iter()
            .map(|event| format!("{event:?}"))
            .collect(),
        modifiers: [
            input.modifiers.shift,
            input.modifiers.control,
            input.modifiers.alt,
            input.modifiers.super_key,
        ],
        focused_id: input.focused_id,
    }
}

struct AppState {
    heading_node: akar_layout::NodeId,
    paragraph_node: akar_layout::NodeId,
    link_node: akar_layout::NodeId,
    card_root: akar_layout::NodeId,
    card_header: akar_layout::NodeId,
    card_body: akar_layout::NodeId,
    card_footer: akar_layout::NodeId,
    card_slots: CardSlots,
    link_result: LinkResult,
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    core: AkarCore,
    layout: Layout,
    page: akar_layout::PageLayout,
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
    active_tab: usize,
    tab_bar_node: akar_layout::NodeId,
    panel_container: akar_layout::NodeId,
    canvas_wrapper: akar_layout::NodeId,
    stats_wrapper: akar_layout::NodeId,
    drawer_open: bool,
    drawer_progress: f32,
    navbar_new_btn_node: akar_layout::NodeId,
    navbar_dropdown_btn_node: akar_layout::NodeId,
    modal_open: bool,
    toasts_list: Vec<ToastItem>,
    dropdown_open: bool,
    prev_active_tab: usize,
    cursor_tick: u64,
    form_name: String,
    form_name_cursor: akar_components::TextEditState,
    form_notes: String,
    form_notes_cursor: akar_components::TextEditState,
    form_notes_scroll_y: f32,
    form_agreed: bool,
    form_theme_idx: usize,
    form_notifications_on: bool,
    form_font_size: f32,
    form_language_idx: usize,
    form_language_open: bool,
    form_container: akar_layout::NodeId,
    form_name_node: akar_layout::NodeId,
    form_notes_node: akar_layout::NodeId,
    form_agreement_node: akar_layout::NodeId,
    form_radio_nodes: [akar_layout::NodeId; 2],
    form_notifications_node: akar_layout::NodeId,
    form_font_size_node: akar_layout::NodeId,
    form_language_node: akar_layout::NodeId,
    form_submit_node: akar_layout::NodeId,
    i18n_container: akar_layout::NodeId,
    i18n_cjk_label_node: akar_layout::NodeId,
    i18n_cjk_node: akar_layout::NodeId,
    i18n_arabic_label_node: akar_layout::NodeId,
    i18n_arabic_node: akar_layout::NodeId,
    i18n_cjk_font: Option<u32>,
    i18n_arabic_font: Option<u32>,
    needs_repaint: bool,
    /// When true, the text-edit caret is forced visible instead of blinking.
    /// Script-driven runs (`--script`) are frame-precise but real-time-driven
    /// blink phase is not: a before/after arrow-key capture can land on a
    /// blink-off frame and look byte-identical, proving nothing. See epic
    /// 023 Task 13 tooling gotcha 1.
    force_caret_visible: bool,
    #[allow(dead_code)] // scaffolding for showcase overlay isolation (Tasks 3-7)
    drawer_root: akar_layout::NodeId,
    #[allow(dead_code)] // scaffolding for showcase overlay isolation (Tasks 3-7)
    modal_root: akar_layout::NodeId,
    #[allow(dead_code)] // scaffolding for showcase overlay isolation (Tasks 3-7)
    toasts_root: akar_layout::NodeId,
}

/// Font candidates tried in order. v1 accepts collections only when all loaded
/// faces resolve to one distinct primary family; multi-family collections are
/// rejected by `load_font_bytes`.
const I18N_CJK_FONT_CANDIDATES: &[&str] = &[
    "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
    "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
    "C:\\Windows\\Fonts\\msgothic.ttc",
];

const I18N_ARABIC_FONT_CANDIDATES: &[&str] = &[
    "/System/Library/Fonts/SFArabic.ttf",
    "/usr/share/fonts/truetype/noto/NotoNaskhArabic-Regular.ttf",
    "C:\\Windows\\Fonts\\arial.ttf",
];

/// Loads the first candidate font that exists and parses, through the public
/// `TextPipeline::load_font_bytes` API. Returns `None` when no candidate is
/// present, so the demo degrades gracefully on machines without these fonts.
fn load_i18n_font(core: &mut AkarCore, label: &str, candidates: &[&str]) -> Option<u32> {
    for path in candidates {
        let Ok(bytes) = std::fs::read(path) else {
            continue;
        };
        match core.text_pipeline.load_font_bytes(bytes) {
            Ok(handle) => {
                let family = core
                    .text_pipeline
                    .family_name(handle)
                    .unwrap_or("<unknown>")
                    .to_string();
                println!("{label} font loaded from {path} as family '{family}' (handle {handle})");
                return Some(handle);
            }
            Err(err) => {
                eprintln!("{label} font at {path} could not be loaded: {err}");
            }
        }
    }
    eprintln!("{label} font unavailable on this machine; i18n samples will show a notice instead");
    None
}

fn main() {
    let config = match cli::parse(std::env::args()) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };

    match config.mode {
        cli::RunMode::Help => {
            println!("{}", cli::HELP_TEXT);
            std::process::exit(0);
        }
        cli::RunMode::ListComponents => {
            cli::print_components();
            std::process::exit(0);
        }
        cli::RunMode::ListVariants(ref comp) => {
            if let Err(e) = cli::print_variants(comp) {
                eprintln!("{e}");
                std::process::exit(1);
            }
            std::process::exit(0);
        }
        cli::RunMode::Run => {}
    }

    let isolated_component = config.component.as_ref().and_then(|name| {
        Component::from_name(name).or_else(|| {
            eprintln!("Unknown component '{name}'. Valid components:");
            for n in Component::names() {
                eprintln!("  {n}");
            }
            std::process::exit(1);
        })
    });

    let script_runner = match config.script {
        Some(ref path) => match std::fs::read_to_string(path) {
            Ok(contents) => match parse_script(&contents) {
                Ok(steps) => Some(ScriptRunner::new(steps)),
                Err(e) => {
                    eprintln!("Failed to parse script '{path}': {e}");
                    std::process::exit(1);
                }
            },
            Err(e) => {
                eprintln!("Failed to read script '{path}': {e}");
                std::process::exit(1);
            }
        },
        None => None,
    };

    let event_loop = EventLoop::new().unwrap();
    event_loop
        .run_app(&mut App {
            state: None,
            screenshot_path: config.screenshot,
            exit_after: config.exit,
            delay_secs: config.delay as f64,
            script_runner,
            dump_layout: config.dump_layout,
            dump_frame_path: config.dump_frame,
            isolated_component,
            rtl: config.rtl,
            forced_initial_state: false,
            start_time: None,
            screenshot_taken: false,
            dump_frame_written: false,
            dump_layout_written: false,
        })
        .unwrap();
}

struct App {
    state: Option<AppState>,
    screenshot_path: Option<String>,
    exit_after: bool,
    delay_secs: f64,
    script_runner: Option<ScriptRunner>,
    dump_layout: bool,
    dump_frame_path: Option<String>,
    isolated_component: Option<Component>,
    rtl: bool,
    forced_initial_state: bool,
    start_time: Option<Instant>,
    screenshot_taken: bool,
    dump_frame_written: bool,
    dump_layout_written: bool,
}

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powf(3.0)
}

fn prepare_layout(
    state: &mut AppState,
    size: PhysicalSize<u32>,
    scale: f32,
    isolated_component: Option<&Component>,
) {
    if let Some(component) = isolated_component {
        component.prepare_isolated_layout(state, size, scale);
        return;
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

    match state.active_tab {
        0 => state
            .layout
            .set_children(state.panel_container, &[state.scroll_container]),
        1 => state
            .layout
            .set_children(state.panel_container, &[state.canvas_wrapper]),
        2 => state
            .layout
            .set_children(state.panel_container, &[state.stats_wrapper]),
        3 => state
            .layout
            .set_children(state.panel_container, &[state.form_container]),
        _ => {}
    }

    state.layout.compute(
        state.page.root,
        (
            Some(size.width as f32 / scale),
            Some(size.height as f32 / scale),
        ),
        |_, _, _, _, _| Size::ZERO,
    );
}

fn render_navbar(state: &mut AppState, _viewport_rect: [f32; 4]) {
    let navbar_id = state.page.header.unwrap();

    if state.navbar_slots.is_none() {
        let slots = akar_navbar_layout(&mut state.layout, navbar_id, &AKAR_THEME_DARK);
        state.layout.add_child(slots.start, state.navbar_title_node);
        state.layout.add_child(slots.end, state.navbar_badge_node);
        state.layout.add_child(slots.end, state.navbar_btn_node);
        state.layout.add_child(slots.end, state.navbar_new_btn_node);
        state
            .layout
            .add_child(slots.end, state.navbar_dropdown_btn_node);
        state.navbar_slots = Some(slots);
    }

    akar_container(
        &mut state.core,
        &state.layout,
        navbar_id,
        &BoxStyle {
            fill: AKAR_THEME_DARK.base_200,
            border_color: AKAR_THEME_DARK.base_300,
            border_width: AKAR_THEME_DARK.border_width,
            corner_radii: [
                0.0,
                0.0,
                AKAR_THEME_DARK.radius_box,
                AKAR_THEME_DARK.radius_box,
            ],
            shadow: None,
        },
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
    let menu_result = akar_button(
        &mut state.core,
        &state.layout,
        state.navbar_btn_node,
        "Menu",
        ButtonVariant::Ghost,
        &AKAR_THEME_DARK,
    );
    if menu_result.clicked {
        state.drawer_open = !state.drawer_open;
        state.needs_repaint = true;
    }

    let new_item_result = akar_button(
        &mut state.core,
        &state.layout,
        state.navbar_new_btn_node,
        "New Item",
        ButtonVariant::Ghost,
        &AKAR_THEME_DARK,
    );
    if new_item_result.clicked {
        state.modal_open = !state.modal_open;
        state.needs_repaint = true;
    }

    let dropdown_btn_result = akar_button(
        &mut state.core,
        &state.layout,
        state.navbar_dropdown_btn_node,
        "Dropdown",
        ButtonVariant::Ghost,
        &AKAR_THEME_DARK,
    );
    if dropdown_btn_result.clicked {
        state.dropdown_open = !state.dropdown_open;
        state.needs_repaint = true;
    }
}

fn render_containers(state: &mut AppState) {
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
}

fn render_alert(state: &mut AppState) {
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
}

fn render_tab_bar(state: &mut AppState) {
    let tab_result = akar_tab_bar(
        &mut state.core,
        &state.layout,
        state.tab_bar_node,
        &["List", "Canvas", "Stats", "Form"],
        state.active_tab,
        TabVariant::Underline,
        &AKAR_THEME_DARK,
    );
    if let Some(index) = tab_result.clicked {
        state.active_tab = index;
        state.needs_repaint = true;
    }

    if state.active_tab != state.prev_active_tab {
        let tab_names = ["List", "Canvas", "Stats", "Form"];
        state.toasts_list.push(ToastItem {
            variant: ToastVariant::Info,
            message: format!("Switched to {} tab", tab_names[state.active_tab]),
            dismiss_on_click: true,
        });
        state.prev_active_tab = state.active_tab;
    }
    while state.toasts_list.len() > 3 {
        state.toasts_list.remove(0);
    }
}

fn render_list_tab(state: &mut AppState, viewport_rect: [f32; 4]) {
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

        let tip_text = format!("{:.0}% complete", progress_value * 100.0);
        akar_tooltip(
            &mut state.core,
            progress_rect,
            &tip_text,
            TooltipSide::Top,
            &AKAR_THEME_DARK,
            viewport_rect,
        );
    }
    scroll_area_end(&mut state.core);
}

fn render_canvas_tab(state: &mut AppState) {
    let canvas_rect = state.layout.rect(state.canvas_wrapper);
    let text = "Canvas View — coming soon";
    let buffer_id = state.core.text_pipeline.set_text(
        Some(2000),
        text,
        glyphon::Metrics::new(18.0, 18.0 * 1.2),
        Some(canvas_rect[2]),
        None,
        None,
    );
    state.core.draw_list.push_text(akar_core::TextCall {
        buffer_id,
        x: canvas_rect[0] + 16.0,
        y: canvas_rect[1] + canvas_rect[3] / 2.0 - 10.0,
        clip: canvas_rect,
        color: [0.6, 0.6, 0.65, 1.0],
        z: 0.0,
    });
}

fn render_stats_tab(state: &mut AppState) {
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
        state.needs_repaint = true;
    }
}

fn render_form_tab(state: &mut AppState, viewport_rect: [f32; 4]) {
    state.cursor_tick += 1;
    let cursor_visible = state.force_caret_visible || (state.cursor_tick / 30).is_multiple_of(2);
    let form_rect = state.layout.rect(state.form_container);

    let title_buf = state.core.text_pipeline.set_text(
        Some(3000),
        "Form Demo",
        glyphon::Metrics::new(18.0, 18.0 * 1.2),
        Some(form_rect[2] - 32.0),
        None,
        None,
    );
    state.core.draw_list.push_text(akar_core::TextCall {
        buffer_id: title_buf,
        x: form_rect[0],
        y: form_rect[1],
        clip: form_rect,
        color: [0.9, 0.9, 0.95, 1.0],
        z: 0.0,
    });

    let name_rect = state.layout.rect(state.form_name_node);
    let name_label_buf = state.core.text_pipeline.set_text(
        Some(3001),
        "Name",
        glyphon::Metrics::new(14.0, 14.0 * 1.2),
        Some(name_rect[2]),
        None,
        None,
    );
    state.core.draw_list.push_text(akar_core::TextCall {
        buffer_id: name_label_buf,
        x: name_rect[0] + AKAR_THEME_DARK.padding_x,
        y: name_rect[1] - 20.0,
        clip: form_rect,
        color: [0.7, 0.7, 0.75, 1.0],
        z: 0.0,
    });

    let _name_result = akar_text_input(
        &mut state.core,
        &state.layout,
        state.form_name_node,
        &mut state.form_name,
        &mut state.form_name_cursor,
        "Enter your name",
        cursor_visible,
        &AKAR_THEME_DARK,
    );

    let notes_rect = state.layout.rect(state.form_notes_node);
    let notes_label_buf = state.core.text_pipeline.set_text(
        Some(3002),
        "Notes",
        glyphon::Metrics::new(14.0, 14.0 * 1.2),
        Some(notes_rect[2]),
        None,
        None,
    );
    state.core.draw_list.push_text(akar_core::TextCall {
        buffer_id: notes_label_buf,
        x: notes_rect[0] + AKAR_THEME_DARK.padding_x,
        y: notes_rect[1] - 20.0,
        clip: form_rect,
        color: [0.7, 0.7, 0.75, 1.0],
        z: 0.0,
    });

    let _notes_result = akar_textarea(
        &mut state.core,
        &state.layout,
        state.form_notes_node,
        &mut state.form_notes,
        &mut state.form_notes_cursor,
        &mut state.form_notes_scroll_y,
        "Enter notes...",
        cursor_visible,
        &AKAR_THEME_DARK,
    );

    let _ = akar_checkbox(
        &mut state.core,
        &state.layout,
        state.form_agreement_node,
        &mut state.form_agreed,
        "I agree to the terms",
        &AKAR_THEME_DARK,
    );

    let _theme_changed = akar_radio_group(
        &mut state.core,
        &state.layout,
        &state.form_radio_nodes,
        &["Dark", "Light"],
        &mut state.form_theme_idx,
        &AKAR_THEME_DARK,
    );

    let _notif_toggled = akar_switch(
        &mut state.core,
        &state.layout,
        state.form_notifications_node,
        &mut state.form_notifications_on,
        &AKAR_THEME_DARK,
    );

    let _font_changed = akar_slider(
        &mut state.core,
        &state.layout,
        state.form_font_size_node,
        &mut state.form_font_size,
        12.0,
        32.0,
        &AKAR_THEME_DARK,
    );

    let _lang_changed = akar_select(
        &mut state.core,
        &state.layout,
        state.form_language_node,
        &["English", "Spanish", "French", "German"],
        &mut state.form_language_idx,
        &mut state.form_language_open,
        &AKAR_THEME_DARK,
        viewport_rect,
    );

    let submit_result = akar_button(
        &mut state.core,
        &state.layout,
        state.form_submit_node,
        "Submit",
        ButtonVariant::Solid,
        &AKAR_THEME_DARK,
    );
    if submit_result.clicked {
        state.needs_repaint = true;
        if state.form_agreed {
            state.toasts_list.push(ToastItem {
                variant: ToastVariant::Success,
                message: "Form submitted successfully!".to_string(),
                dismiss_on_click: true,
            });
        } else {
            state.toasts_list.push(ToastItem {
                variant: ToastVariant::Warning,
                message: "Please agree to the terms.".to_string(),
                dismiss_on_click: true,
            });
        }
    }
}

fn render_drawer(state: &mut AppState, viewport_rect: [f32; 4]) {
    let max_width = 250.0_f32;
    let speed = 0.08;
    if state.drawer_open {
        state.drawer_progress = (state.drawer_progress + speed).min(1.0);
    } else {
        state.drawer_progress = (state.drawer_progress - speed).max(0.0);
    }
    if state.drawer_progress > 0.0 && state.drawer_progress < 1.0 {
        state.needs_repaint = true;
    }
    let panel_width = max_width * ease_out_cubic(state.drawer_progress);

    if panel_width > 1.0 {
        let drawer_resp = drawer_begin(
            &mut state.core,
            viewport_rect,
            DrawerEdge::Left,
            panel_width,
            &AKAR_THEME_DARK,
        );

        let padding = 16.0_f32;
        let y_offset = 24.0_f32;

        let avatar_rect = [padding, y_offset, 40.0, 40.0];
        state.core.draw_list.push_quad(akar_core::QuadCall {
            rect: avatar_rect,
            fill: [0.23, 0.51, 0.96, 1.0],
            border_color: [0.0; 4],
            corner_radii: [20.0; 4],
            border_width: 0.0,
            z: Z_FLOAT,
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            shadow_color: [0.0; 4],
            shadow_offset: [0.0; 2],
            _pad: [0.0; 2],
        });

        let initials_buf = state.core.text_pipeline.set_text(
            Some(9001),
            "AK",
            glyphon::Metrics::new(16.0, 16.0 * 1.2),
            Some(40.0),
            None,
            None,
        );
        state.core.draw_list.push_text(akar_core::TextCall {
            buffer_id: initials_buf,
            x: avatar_rect[0] + 10.0,
            y: avatar_rect[1] + 10.0,
            clip: avatar_rect,
            color: [1.0; 4],
            z: Z_FLOAT,
        });

        let nav_links = ["Dashboard", "Settings", "Profile", "Help"];
        let link_start_y = y_offset + 40.0 + 24.0;
        for (i, link) in nav_links.iter().enumerate() {
            let link_rect = [
                padding,
                link_start_y + i as f32 * 40.0,
                panel_width - 2.0 * padding,
                32.0,
            ];

            state.core.draw_list.push_quad(akar_core::QuadCall {
                rect: link_rect,
                fill: [0.15, 0.16, 0.17, 1.0],
                border_color: [0.0; 4],
                corner_radii: [4.0; 4],
                border_width: 0.0,
                z: Z_FLOAT,
                shadow_blur: 0.0,
                shadow_spread: 0.0,
                shadow_color: [0.0; 4],
                shadow_offset: [0.0; 2],
                _pad: [0.0; 2],
            });

            let link_buf = state.core.text_pipeline.set_text(
                Some(10000 + i as u64),
                link,
                glyphon::Metrics::new(14.0, 14.0 * 1.2),
                Some(link_rect[2]),
                None,
                None,
            );
            state.core.draw_list.push_text(akar_core::TextCall {
                buffer_id: link_buf,
                x: link_rect[0] + 8.0,
                y: link_rect[1] + 6.0,
                clip: link_rect,
                color: [0.9, 0.9, 0.92, 1.0],
                z: Z_FLOAT,
            });
        }

        drawer_end(&mut state.core);

        if drawer_resp.close_requested {
            state.drawer_open = false;
            state.needs_repaint = true;
        }
    }
}

fn render_modal(state: &mut AppState, viewport_rect: [f32; 4]) {
    if state.modal_open {
        let modal_resp = modal_begin(
            &mut state.core,
            &mut state.layout,
            viewport_rect,
            "New Item",
            400.0,
            300.0,
            &AKAR_THEME_DARK,
        );

        let content_rect = modal_resp.content_rect;

        let buffer_id = state.core.text_pipeline.set_text(
            Some(5000),
            "Modal content area \u{2014} add your form here.",
            glyphon::Metrics::new(16.0, 16.0 * 1.2),
            Some(content_rect[2] - 32.0),
            None,
            None,
        );
        state.core.draw_list.push_text(akar_core::TextCall {
            buffer_id,
            x: content_rect[0] + 16.0,
            y: content_rect[1] + 16.0,
            clip: content_rect,
            color: [0.8, 0.8, 0.85, 1.0],
            z: akar_core::Z_FLOAT,
        });

        modal_end(&mut state.core);

        if modal_resp.close_requested {
            state.modal_open = false;
            state.needs_repaint = true;
        }
    }
}

fn render_toasts(state: &mut AppState, viewport_rect: [f32; 4]) {
    let toast_resp = toasts(
        &mut state.core,
        viewport_rect,
        &mut state.toasts_list,
        &AKAR_THEME_DARK,
    );
    if let Some(index) = toast_resp.dismissed {
        state.toasts_list.remove(index);
        state.needs_repaint = true;
    }
}

fn render_dropdown(state: &mut AppState, viewport_rect: [f32; 4]) {
    let dropdown_btn_rect = state.layout.rect(state.navbar_dropdown_btn_node);
    let dropdown_state = dropdown_begin(
        &mut state.core,
        dropdown_btn_rect,
        28.0,
        viewport_rect,
        state.dropdown_open,
        &AKAR_THEME_DARK,
    );

    if dropdown_state.is_open {
        let items = ["Option A", "Option B", "Option C", "Option D"];
        for (i, item) in items.iter().enumerate() {
            let item_y = dropdown_state.content_rect[1] + i as f32 * 28.0;
            let item_rect = [
                dropdown_state.content_rect[0],
                item_y,
                dropdown_state.content_rect[2],
                28.0,
            ];

            if state.core.input.is_hovering(item_rect) {
                state.core.draw_list.push_quad(akar_core::QuadCall {
                    rect: item_rect,
                    fill: [0.2, 0.22, 0.25, 1.0],
                    border_color: [0.0; 4],
                    corner_radii: [0.0; 4],
                    border_width: 0.0,
                    z: akar_core::Z_OVERLAY,
                    shadow_blur: 0.0,
                    shadow_spread: 0.0,
                    shadow_color: [0.0; 4],
                    shadow_offset: [0.0; 2],
                    _pad: [0.0; 2],
                });
            }

            let item_buf = state.core.text_pipeline.set_text(
                Some(6000 + i as u64),
                item,
                glyphon::Metrics::new(14.0, 14.0 * 1.2),
                Some(item_rect[2] - 8.0),
                None,
                None,
            );
            state.core.draw_list.push_text(akar_core::TextCall {
                buffer_id: item_buf,
                x: item_rect[0] + 4.0,
                y: item_rect[1] + 5.0,
                clip: item_rect,
                color: [0.9, 0.9, 0.92, 1.0],
                z: akar_core::Z_OVERLAY,
            });

            if state.core.input.is_clicked(item_rect) {
                state.dropdown_open = false;
                state.needs_repaint = true;
            }
        }

        dropdown_end(&mut state.core);
    }
}

fn render_isolated_dropdown(state: &mut AppState, viewport_rect: [f32; 4]) {
    let trigger = akar_button(
        &mut state.core,
        &state.layout,
        state.navbar_dropdown_btn_node,
        "Dropdown",
        ButtonVariant::Ghost,
        &AKAR_THEME_DARK,
    );
    if trigger.clicked {
        state.dropdown_open = !state.dropdown_open;
        state.needs_repaint = true;
    }
    render_dropdown(state, viewport_rect);
}

fn render_all(state: &mut AppState, viewport_rect: [f32; 4]) {
    render_containers(state);
    render_navbar(state, viewport_rect);
    render_alert(state);
    render_tab_bar(state);
    match state.active_tab {
        0 => render_list_tab(state, viewport_rect),
        1 => render_canvas_tab(state),
        2 => render_stats_tab(state),
        3 => {
            render_form_tab(state, viewport_rect);
            state.needs_repaint = true;
        }
        _ => {}
    }
    render_drawer(state, viewport_rect);
    render_modal(state, viewport_rect);
    render_toasts(state, viewport_rect);
    render_dropdown(state, viewport_rect);
}

enum Component {
    Navbar,
    Alert,
    TabBar,
    ListTab,
    CanvasTab,
    StatsTab,
    FormTab,
    Drawer,
    Modal,
    Toasts,
    Dropdown,
    Heading,
    Paragraph,
    Link,
    Card,
    I18n,
}

fn render_i18n_sample(
    state: &mut AppState,
    label_node: akar_layout::NodeId,
    text_node: akar_layout::NodeId,
    font_handle: Option<u32>,
    label: &str,
    sample: &str,
    missing_notice: &str,
) {
    akar_paragraph(
        &mut state.core,
        &state.layout,
        label_node,
        label,
        None,
        &AKAR_THEME_DARK,
    );
    let (text, style) = match font_handle {
        Some(handle) => (
            sample,
            Some(TextStyle {
                font_size: Some(28.0),
                line_height: Some(44.0),
                font_family: Some(FontFamily::Named(handle)),
                ..TextStyle::empty()
            }),
        ),
        None => (missing_notice, None),
    };
    akar_paragraph(
        &mut state.core,
        &state.layout,
        text_node,
        text,
        style,
        &AKAR_THEME_DARK,
    );
}

fn ensure_navbar_slots(state: &mut AppState) {
    if state.navbar_slots.is_none() {
        let navbar_id = state.page.header.unwrap();
        let slots = akar_navbar_layout(&mut state.layout, navbar_id, &AKAR_THEME_DARK);
        state.layout.add_child(slots.start, state.navbar_title_node);
        state.layout.add_child(slots.end, state.navbar_badge_node);
        state.layout.add_child(slots.end, state.navbar_btn_node);
        state.layout.add_child(slots.end, state.navbar_new_btn_node);
        state
            .layout
            .add_child(slots.end, state.navbar_dropdown_btn_node);
        state.navbar_slots = Some(slots);
    }
}

fn compute_component_aabb(recorded: &[akar_core::draw_list::RecordedCall]) -> Option<[f32; 4]> {
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;

    for call in recorded {
        let z = match &call.call {
            akar_core::DrawCall::Quad(q) => q.z,
            akar_core::DrawCall::Text(t) => t.z,
        };
        if z == akar_core::Z_SCRIM {
            continue;
        }
        let rect = call.physical_rect;
        if let Some(scissor) = call.scissor {
            if rect[0] + rect[2] <= scissor[0]
                || rect[1] + rect[3] <= scissor[1]
                || rect[0] >= scissor[0] + scissor[2]
                || rect[1] >= scissor[1] + scissor[3]
            {
                continue;
            }
        }
        min_x = min_x.min(rect[0]);
        min_y = min_y.min(rect[1]);
        max_x = max_x.max(rect[0] + rect[2]);
        max_y = max_y.max(rect[1] + rect[3]);
    }

    if min_x == f32::MAX {
        None
    } else {
        Some([min_x, min_y, max_x - min_x, max_y - min_y])
    }
}

fn crop_and_write_png(
    frame: &akar_core::screenshot::CapturedFrame,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    path: &str,
) -> Result<(), String> {
    let mut cropped = vec![0u8; (w * h * 4) as usize];
    for row in 0..h {
        let src_start = ((y + row) * frame.width + x) as usize * 4;
        let dst_start = (row * w) as usize * 4;
        let row_bytes = w as usize * 4;
        let src_end = src_start + row_bytes;
        cropped[dst_start..dst_start + row_bytes].copy_from_slice(&frame.rgba[src_start..src_end]);
    }

    let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let mut encoder = png::Encoder::new(file, w, h);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
    writer
        .write_image_data(&cropped)
        .map_err(|e| e.to_string())?;
    Ok(())
}

impl Component {
    fn prepare_isolated_layout(&self, state: &mut AppState, size: PhysicalSize<u32>, scale: f32) {
        let (root, style) = match self {
            Self::Navbar => {
                ensure_navbar_slots(state);
                (
                    state.page.header.unwrap(),
                    Style {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        align_items: Some(AlignItems::CENTER),
                        size: Size {
                            width: length(600.0_f32),
                            height: length(48.0_f32),
                        },
                        ..Default::default()
                    },
                )
            }
            Self::Alert => (
                state.alert_node,
                Style {
                    flex_shrink: 0.0,
                    size: Size {
                        width: length(520.0_f32),
                        height: length(48.0_f32),
                    },
                    ..Default::default()
                },
            ),
            Self::TabBar => (
                state.tab_bar_node,
                Style {
                    flex_shrink: 0.0,
                    size: Size {
                        width: length(520.0_f32),
                        height: length(40.0_f32),
                    },
                    ..Default::default()
                },
            ),
            Self::ListTab => (
                state.scroll_container,
                Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    overflow: taffy::geometry::Point {
                        x: taffy::style::Overflow::Clip,
                        y: taffy::style::Overflow::Clip,
                    },
                    size: Size {
                        width: length(600.0_f32),
                        height: length(480.0_f32),
                    },
                    ..Default::default()
                },
            ),
            Self::CanvasTab => (
                state.canvas_wrapper,
                Style {
                    display: Display::Flex,
                    align_items: Some(AlignItems::CENTER),
                    justify_content: Some(JustifyContent::CENTER),
                    size: Size {
                        width: length(600.0_f32),
                        height: length(360.0_f32),
                    },
                    ..Default::default()
                },
            ),
            Self::StatsTab => (
                state.stats_wrapper,
                Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    gap: taffy::geometry::Size {
                        width: length(0.0_f32),
                        height: length(8.0_f32),
                    },
                    size: Size {
                        width: length(600.0_f32),
                        height: length(228.0_f32),
                    },
                    ..Default::default()
                },
            ),
            Self::FormTab => (
                state.form_container,
                Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    size: Size {
                        width: length(450.0_f32),
                        height: length(500.0_f32),
                    },
                    padding: taffy::geometry::Rect {
                        left: length(16.0_f32),
                        right: length(16.0_f32),
                        top: length(16.0_f32),
                        bottom: length(16.0_f32),
                    },
                    gap: taffy::geometry::Size {
                        width: length(0.0_f32),
                        height: length(12.0_f32),
                    },
                    ..Default::default()
                },
            ),
            Self::Dropdown => (
                state.navbar_dropdown_btn_node,
                Style {
                    flex_shrink: 0.0,
                    size: Size {
                        width: length(160.0_f32),
                        height: length(32.0_f32),
                    },
                    ..Default::default()
                },
            ),
            Self::Heading => (
                state.heading_node,
                Style {
                    flex_shrink: 0.0,
                    size: Size {
                        width: length(600.0_f32),
                        height: length(60.0_f32),
                    },
                    ..Default::default()
                },
            ),
            Self::Paragraph => (
                state.paragraph_node,
                Style {
                    flex_shrink: 0.0,
                    size: Size {
                        width: length(600.0_f32),
                        height: length(120.0_f32),
                    },
                    ..Default::default()
                },
            ),
            Self::Link => (
                state.link_node,
                Style {
                    flex_shrink: 0.0,
                    size: Size {
                        width: length(300.0_f32),
                        height: length(32.0_f32),
                    },
                    ..Default::default()
                },
            ),
            Self::Card => {
                state.layout.set_style(
                    state.card_root,
                    Style {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        size: Size {
                            width: length(400.0_f32),
                            height: length(280.0_f32),
                        },
                        padding: taffy::geometry::Rect {
                            left: length(16.0_f32),
                            right: length(16.0_f32),
                            top: length(16.0_f32),
                            bottom: length(16.0_f32),
                        },
                        gap: taffy::geometry::Size {
                            width: length(0.0_f32),
                            height: length(8.0_f32),
                        },
                        ..Default::default()
                    },
                );
                state.layout.set_style(
                    state.card_header,
                    Style {
                        flex_shrink: 0.0,
                        size: Size {
                            width: Dimension::percent(1.0),
                            height: length(32.0_f32),
                        },
                        ..Default::default()
                    },
                );
                state.layout.set_style(
                    state.card_body,
                    Style {
                        flex_grow: 1.0,
                        size: Size {
                            width: Dimension::percent(1.0),
                            height: length(120.0_f32),
                        },
                        ..Default::default()
                    },
                );
                state.layout.set_style(
                    state.card_footer,
                    Style {
                        flex_shrink: 0.0,
                        size: Size {
                            width: Dimension::percent(1.0),
                            height: length(28.0_f32),
                        },
                        ..Default::default()
                    },
                );
                state.layout.compute(
                    state.card_root,
                    (
                        Some(size.width as f32 / scale),
                        Some(size.height as f32 / scale),
                    ),
                    |_, _, _, _, _| Size::ZERO,
                );
                return;
            }
            Self::I18n => (
                state.i18n_container,
                Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    size: Size {
                        width: length(600.0_f32),
                        height: length(220.0_f32),
                    },
                    gap: taffy::geometry::Size {
                        width: length(0.0_f32),
                        height: length(8.0_f32),
                    },
                    ..Default::default()
                },
            ),
            Self::Drawer | Self::Modal | Self::Toasts => return,
        };

        state.layout.set_style(root, style);
        state.layout.compute(
            root,
            (
                Some(size.width as f32 / scale),
                Some(size.height as f32 / scale),
            ),
            |_, _, _, _, _| Size::ZERO,
        );
    }

    fn from_name(name: &str) -> Option<Self> {
        match name {
            "navbar" => Some(Self::Navbar),
            "alert" => Some(Self::Alert),
            "tab_bar" => Some(Self::TabBar),
            "list" => Some(Self::ListTab),
            "canvas" => Some(Self::CanvasTab),
            "stats" => Some(Self::StatsTab),
            "form" => Some(Self::FormTab),
            "drawer" => Some(Self::Drawer),
            "modal" => Some(Self::Modal),
            "toasts" => Some(Self::Toasts),
            "dropdown" => Some(Self::Dropdown),
            "heading" => Some(Self::Heading),
            "paragraph" => Some(Self::Paragraph),
            "link" => Some(Self::Link),
            "card" => Some(Self::Card),
            "i18n" => Some(Self::I18n),
            _ => None,
        }
    }

    fn names() -> &'static [&'static str] {
        &[
            "navbar",
            "alert",
            "tab_bar",
            "list",
            "canvas",
            "stats",
            "form",
            "drawer",
            "modal",
            "toasts",
            "dropdown",
            "heading",
            "paragraph",
            "link",
            "card",
            "i18n",
        ]
    }

    fn render(&self, state: &mut AppState, viewport_rect: [f32; 4]) {
        match self {
            Self::Navbar => render_navbar(state, viewport_rect),
            Self::Alert => render_alert(state),
            Self::TabBar => render_tab_bar(state),
            Self::ListTab => render_list_tab(state, viewport_rect),
            Self::CanvasTab => render_canvas_tab(state),
            Self::StatsTab => render_stats_tab(state),
            Self::FormTab => render_form_tab(state, viewport_rect),
            Self::Drawer => render_drawer(state, viewport_rect),
            Self::Modal => render_modal(state, viewport_rect),
            Self::Toasts => render_toasts(state, viewport_rect),
            Self::Dropdown => render_isolated_dropdown(state, viewport_rect),
            Self::Heading => {
                akar_heading(
                    &mut state.core,
                    &state.layout,
                    state.heading_node,
                    "Heading Level 1",
                    HeadingLevel::H1,
                    Some(TextStyle {
                        font_family: Some(FontFamily::Serif),
                        ..TextStyle::empty()
                    }),
                    &AKAR_THEME_DARK,
                );
            }
            Self::Paragraph => {
                akar_paragraph(
                    &mut state.core,
                    &state.layout,
                    state.paragraph_node,
                    "This is a paragraph component with automatic text wrapping. It demonstrates how akar handles multi-line text layout with intrinsic measurement.",
                    None,
                    &AKAR_THEME_DARK,
                );
            }
            Self::Link => {
                let result = akar_link(
                    &mut state.core,
                    &state.layout,
                    state.link_node,
                    "Visit akar docs",
                    None,
                    &AKAR_THEME_DARK,
                );
                state.link_result = result;
            }
            Self::Card => {
                akar_card(
                    &mut state.core,
                    &state.layout,
                    state.card_root,
                    &state.card_slots,
                    &CardStyle::default(&AKAR_THEME_DARK),
                );
                akar_heading(
                    &mut state.core,
                    &state.layout,
                    state.card_header,
                    "Card Header",
                    HeadingLevel::H3,
                    None,
                    &AKAR_THEME_DARK,
                );
                akar_paragraph(
                    &mut state.core,
                    &state.layout,
                    state.card_body,
                    "Card body content with a heading, paragraph, and footer slot.",
                    None,
                    &AKAR_THEME_DARK,
                );
                akar_heading(
                    &mut state.core,
                    &state.layout,
                    state.card_footer,
                    "Card Footer",
                    HeadingLevel::H4,
                    None,
                    &AKAR_THEME_DARK,
                );
            }
            Self::I18n => {
                render_i18n_sample(
                    state,
                    state.i18n_cjk_label_node,
                    state.i18n_cjk_node,
                    state.i18n_cjk_font,
                    "CJK via FontFamily::Named:",
                    "你好世界 汉字 ひらがな カタカナ 한국어",
                    "CJK font unavailable on this machine",
                );
                render_i18n_sample(
                    state,
                    state.i18n_arabic_label_node,
                    state.i18n_arabic_node,
                    state.i18n_arabic_font,
                    "Arabic via FontFamily::Named:",
                    "مرحبا بالعالم العربية",
                    "Arabic font unavailable on this machine",
                );
            }
        }
    }

    fn force_state_initial(&self, state: &mut AppState) {
        match self {
            Self::Alert => {
                state.alert_dismissed = false;
            }
            Self::ListTab => {
                state.active_tab = 0;
                state.prev_active_tab = 0;
            }
            Self::CanvasTab => {
                state.active_tab = 1;
                state.prev_active_tab = 1;
            }
            Self::StatsTab => {
                state.active_tab = 2;
                state.prev_active_tab = 2;
            }
            Self::FormTab => {
                state.active_tab = 3;
                state.prev_active_tab = 3;
            }
            Self::Drawer => {
                state.drawer_open = true;
                state.drawer_progress = 1.0;
            }
            Self::Dropdown => {
                state.dropdown_open = true;
            }
            Self::Modal => {
                state.modal_open = true;
            }
            Self::Toasts => {
                if state.toasts_list.is_empty() {
                    state.toasts_list.push(ToastItem {
                        variant: ToastVariant::Info,
                        message: "Sample toast".to_string(),
                        dismiss_on_click: false,
                    });
                }
            }
            Self::Heading
            | Self::Paragraph
            | Self::Link
            | Self::Card
            | Self::I18n
            | Self::Navbar
            | Self::TabBar => {}
        }
    }
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

        let mut core = AkarCore::new(
            &device,
            &queue,
            surface_format,
            akar_core::TextPipelineConfig::default(),
        );
        core.set_text_edit_keybindings(akar_core::TextEditKeybindings::platform_default());

        let i18n_cjk_font = load_i18n_font(&mut core, "CJK", I18N_CJK_FONT_CANDIDATES);
        let i18n_arabic_font = load_i18n_font(&mut core, "Arabic", I18N_ARABIC_FONT_CANDIDATES);

        let mut layout = Layout::new();
        if self.rtl {
            layout.set_direction(AkarDirection::Rtl);
        }

        let page = layout.page(PageConfig {
            header_height: Some(48.0),
            footer_height: None,
            sidebar_left_width: Some(200.0),
            sidebar_right_width: None,
        });

        layout.set_style(
            page.main,
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                gap: taffy::geometry::Size {
                    width: length(0.0_f32),
                    height: length(8.0_f32),
                },
                ..Default::default()
            },
        );

        let alert_node = layout.new_leaf(Style {
            flex_shrink: 0.0,
            size: Size {
                width: Dimension::percent(1.0),
                height: length(48.0_f32),
            },
            ..Default::default()
        });

        let tab_bar_node = layout.new_leaf(Style {
            flex_shrink: 0.0,
            size: Size {
                width: Dimension::percent(1.0),
                height: length(40.0_f32),
            },
            ..Default::default()
        });

        let panel_container = layout.new_leaf(Style {
            flex_grow: 1.0,
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            ..Default::default()
        });

        layout.set_children(page.main, &[alert_node, tab_bar_node, panel_container]);

        let stat_row = layout.new_leaf(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            flex_shrink: 0.0,
            size: Size {
                width: Dimension::percent(1.0),
                height: length(100.0_f32),
            },
            gap: taffy::geometry::Size {
                width: length(8.0_f32),
                height: length(0.0_f32),
            },
            padding: taffy::geometry::Rect {
                left: length(8.0_f32),
                right: length(8.0_f32),
                top: length(4.0_f32),
                bottom: length(4.0_f32),
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
                height: length(56.0_f32),
            },
            ..Default::default()
        });

        let avatar_row = layout.new_leaf(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            flex_shrink: 0.0,
            align_items: Some(AlignItems::CENTER),
            size: Size {
                width: Dimension::percent(1.0),
                height: length(56.0_f32),
            },
            gap: taffy::geometry::Size {
                width: length(8.0_f32),
                height: length(0.0_f32),
            },
            padding: taffy::geometry::Rect {
                left: length(8.0_f32),
                right: length(0.0_f32),
                top: length(0.0_f32),
                bottom: length(0.0_f32),
            },
            ..Default::default()
        });

        let avatar_1 = layout.new_leaf(Style {
            flex_shrink: 0.0,
            size: Size {
                width: length(40.0_f32),
                height: length(40.0_f32),
            },
            ..Default::default()
        });
        let avatar_2 = layout.new_leaf(Style {
            flex_shrink: 0.0,
            size: Size {
                width: length(40.0_f32),
                height: length(40.0_f32),
            },
            ..Default::default()
        });
        let avatar_3 = layout.new_leaf(Style {
            flex_shrink: 0.0,
            size: Size {
                width: length(40.0_f32),
                height: length(40.0_f32),
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
                width: length(140.0_f32),
                height: length(32.0_f32),
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

        let canvas_wrapper = layout.new_leaf(Style {
            flex_grow: 1.0,
            display: Display::Flex,
            align_items: Some(AlignItems::CENTER),
            justify_content: Some(JustifyContent::CENTER),
            ..Default::default()
        });

        let stats_wrapper = layout.new_with_children(
            Style {
                flex_grow: 1.0,
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                gap: taffy::geometry::Size {
                    width: length(0.0_f32),
                    height: length(8.0_f32),
                },
                ..Default::default()
            },
            &[stat_row, steps_node, avatar_row],
        );

        let navbar_title_node = layout.new_leaf(Style {
            flex_shrink: 0.0,
            size: Size {
                width: length(80.0_f32),
                height: length(48.0_f32),
            },
            ..Default::default()
        });
        let navbar_badge_node = layout.new_leaf(Style {
            flex_shrink: 0.0,
            size: Size {
                width: length(32.0_f32),
                height: length(24.0_f32),
            },
            ..Default::default()
        });
        let navbar_btn_node = layout.new_leaf(Style {
            flex_shrink: 0.0,
            size: Size {
                width: length(120.0_f32),
                height: length(32.0_f32),
            },
            ..Default::default()
        });
        let navbar_new_btn_node = layout.new_leaf(Style {
            flex_shrink: 0.0,
            size: Size {
                width: length(80.0_f32),
                height: length(32.0_f32),
            },
            ..Default::default()
        });
        let navbar_dropdown_btn_node = layout.new_leaf(Style {
            flex_shrink: 0.0,
            size: Size {
                width: length(100.0_f32),
                height: length(32.0_f32),
            },
            ..Default::default()
        });

        let form_container = layout.new_leaf(Style {
            flex_grow: 1.0,
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            padding: taffy::geometry::Rect {
                left: length(16.0_f32),
                right: length(16.0_f32),
                top: length(16.0_f32),
                bottom: length(16.0_f32),
            },
            gap: taffy::geometry::Size {
                width: length(0.0_f32),
                height: length(12.0_f32),
            },
            ..Default::default()
        });

        let form_name_node = layout.new_leaf(Style {
            flex_shrink: 0.0,
            size: Size {
                width: Dimension::percent(1.0),
                height: length(40.0_f32),
            },
            margin: taffy::geometry::Rect {
                top: length(20.0_f32),
                right: length(0.0_f32),
                bottom: length(0.0_f32),
                left: length(0.0_f32),
            },
            ..Default::default()
        });

        let form_notes_node = layout.new_leaf(Style {
            flex_shrink: 0.0,
            size: Size {
                width: Dimension::percent(1.0),
                height: length(100.0_f32),
            },
            margin: taffy::geometry::Rect {
                top: length(20.0_f32),
                right: length(0.0_f32),
                bottom: length(0.0_f32),
                left: length(0.0_f32),
            },
            ..Default::default()
        });

        let form_agreement_node = layout.new_leaf(Style {
            flex_shrink: 0.0,
            size: Size {
                width: Dimension::percent(1.0),
                height: length(32.0_f32),
            },
            ..Default::default()
        });

        let form_radio_row = layout.new_leaf(Style {
            flex_shrink: 0.0,
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            size: Size {
                width: Dimension::percent(1.0),
                height: length(32.0_f32),
            },
            gap: taffy::geometry::Size {
                width: length(16.0_f32),
                height: length(0.0_f32),
            },
            ..Default::default()
        });
        let form_radio_dark = layout.new_leaf(Style {
            size: Size {
                width: Dimension::auto(),
                height: Dimension::percent(1.0),
            },
            flex_grow: 1.0,
            ..Default::default()
        });
        let form_radio_light = layout.new_leaf(Style {
            size: Size {
                width: Dimension::auto(),
                height: Dimension::percent(1.0),
            },
            flex_grow: 1.0,
            ..Default::default()
        });
        layout.set_children(form_radio_row, &[form_radio_dark, form_radio_light]);

        let form_notifications_node = layout.new_leaf(Style {
            flex_shrink: 0.0,
            size: Size {
                width: Dimension::percent(1.0),
                height: length(32.0_f32),
            },
            ..Default::default()
        });

        let form_font_size_node = layout.new_leaf(Style {
            flex_shrink: 0.0,
            size: Size {
                width: Dimension::percent(1.0),
                height: length(32.0_f32),
            },
            ..Default::default()
        });

        let form_language_node = layout.new_leaf(Style {
            flex_shrink: 0.0,
            size: Size {
                width: Dimension::percent(1.0),
                height: length(40.0_f32),
            },
            ..Default::default()
        });

        let form_submit_node = layout.new_leaf(Style {
            flex_shrink: 0.0,
            size: Size {
                width: length(120.0_f32),
                height: length(36.0_f32),
            },
            align_self: Some(AlignSelf::CENTER),
            ..Default::default()
        });

        layout.set_children(
            form_container,
            &[
                form_name_node,
                form_notes_node,
                form_agreement_node,
                form_radio_row,
                form_notifications_node,
                form_font_size_node,
                form_language_node,
                form_submit_node,
            ],
        );

        let form_radio_nodes = [form_radio_dark, form_radio_light];

        let heading_node = layout.new_leaf(Style {
            flex_shrink: 0.0,
            size: Size {
                width: length(600.0_f32),
                height: length(60.0_f32),
            },
            ..Default::default()
        });
        let paragraph_node = layout.new_leaf(Style {
            flex_shrink: 0.0,
            size: Size {
                width: length(600.0_f32),
                height: length(120.0_f32),
            },
            ..Default::default()
        });
        let link_node = layout.new_leaf(Style {
            flex_shrink: 0.0,
            size: Size {
                width: length(300.0_f32),
                height: length(32.0_f32),
            },
            ..Default::default()
        });
        let card_root = layout.new_leaf(Style {
            flex_shrink: 0.0,
            size: Size {
                width: length(400.0_f32),
                height: length(280.0_f32),
            },
            padding: taffy::geometry::Rect {
                left: length(16.0_f32),
                right: length(16.0_f32),
                top: length(16.0_f32),
                bottom: length(16.0_f32),
            },
            gap: taffy::geometry::Size {
                width: length(0.0_f32),
                height: length(8.0_f32),
            },
            ..Default::default()
        });
        let i18n_container = layout.new_leaf(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            size: Size {
                width: length(600.0_f32),
                height: length(220.0_f32),
            },
            gap: taffy::geometry::Size {
                width: length(0.0_f32),
                height: length(8.0_f32),
            },
            ..Default::default()
        });
        let i18n_leaf = || Style {
            flex_shrink: 0.0,
            size: Size {
                width: Dimension::percent(1.0),
                height: length(24.0_f32),
            },
            ..Default::default()
        };
        let i18n_cjk_label_node = layout.new_leaf(i18n_leaf());
        let i18n_cjk_node = layout.new_leaf(Style {
            size: Size {
                width: Dimension::percent(1.0),
                height: length(56.0_f32),
            },
            ..i18n_leaf()
        });
        let i18n_arabic_label_node = layout.new_leaf(i18n_leaf());
        let i18n_arabic_node = layout.new_leaf(Style {
            size: Size {
                width: Dimension::percent(1.0),
                height: length(56.0_f32),
            },
            ..i18n_leaf()
        });
        layout.set_children(
            i18n_container,
            &[
                i18n_cjk_label_node,
                i18n_cjk_node,
                i18n_arabic_label_node,
                i18n_arabic_node,
            ],
        );

        let card_layout_opts = CardLayout::with_header_footer(&AKAR_THEME_DARK);
        let card_slots = akar_card_layout(&mut layout, card_root, &card_layout_opts);
        let card_header = card_slots.header.unwrap();
        let card_body = card_slots.body;
        let card_footer = card_slots.footer.unwrap();

        layout.register_label("navbar_btn", navbar_btn_node);
        layout.register_label("navbar_new_btn", navbar_new_btn_node);
        layout.register_label("navbar_dropdown", navbar_dropdown_btn_node);
        layout.register_label("alert", alert_node);
        layout.register_label("tab_bar", tab_bar_node);
        layout.register_label("stat_0", stat_nodes[0]);
        layout.register_label("stat_1", stat_nodes[1]);
        layout.register_label("stat_2", stat_nodes[2]);
        layout.register_label("steps", steps_node);
        layout.register_label("avatar_0", avatar_nodes[0]);
        layout.register_label("avatar_1", avatar_nodes[1]);
        layout.register_label("avatar_2", avatar_nodes[2]);
        layout.register_label("skeleton_toggle", skeleton_toggle_node);
        layout.register_label("form_name", form_name_node);
        layout.register_label("form_notes", form_notes_node);
        layout.register_label("form_agreement", form_agreement_node);
        layout.register_label("form_radio_dark", form_radio_nodes[0]);
        layout.register_label("form_radio_light", form_radio_nodes[1]);
        layout.register_label("form_notifications", form_notifications_node);
        layout.register_label("form_font_size", form_font_size_node);
        layout.register_label("form_language", form_language_node);
        layout.register_label("form_submit", form_submit_node);
        layout.register_label("heading", heading_node);
        layout.register_label("paragraph", paragraph_node);
        layout.register_label("link", link_node);
        layout.register_label("card", card_root);
        layout.register_label("i18n", i18n_container);
        layout.register_label("i18n_cjk", i18n_cjk_node);
        layout.register_label("i18n_arabic", i18n_arabic_node);

        let navbar_node = page.header.unwrap();
        layout.register_label("navbar", navbar_node);
        layout.register_label("list", scroll_container);
        layout.register_label("canvas", canvas_wrapper);
        layout.register_label("stats", stats_wrapper);
        layout.register_label("form", form_container);
        layout.register_label("dropdown", navbar_dropdown_btn_node);

        let drawer_root = layout.new_leaf(Style {
            display: Display::Flex,
            size: Size {
                width: length(400.0_f32),
                height: length(300.0_f32),
            },
            ..Default::default()
        });
        layout.register_label("drawer", drawer_root);

        let modal_root = layout.new_leaf(Style {
            display: Display::Flex,
            size: Size {
                width: length(400.0_f32),
                height: length(300.0_f32),
            },
            ..Default::default()
        });
        layout.register_label("modal", modal_root);

        let toasts_root = layout.new_leaf(Style {
            display: Display::Flex,
            size: Size {
                width: length(320.0_f32),
                height: length(200.0_f32),
            },
            ..Default::default()
        });
        layout.register_label("toasts", toasts_root);

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
            page,
            scroll_y: 0.0,
            scroll_container,
            navbar_slots: None,
            navbar_title_node,
            navbar_badge_node,
            navbar_btn_node,
            navbar_new_btn_node,
            navbar_dropdown_btn_node,
            alert_node,
            alert_dismissed: false,
            stat_nodes,
            steps_node,
            avatar_nodes,
            skeleton_toggle_node,
            show_skeleton: false,
            active_tab: 0,
            tab_bar_node,
            panel_container,
            canvas_wrapper,
            stats_wrapper,
            drawer_open: false,
            drawer_progress: 0.0,
            modal_open: false,
            toasts_list: Vec::new(),
            dropdown_open: false,
            prev_active_tab: 0,
            cursor_tick: 0,
            form_name: String::new(),
            form_name_cursor: akar_components::TextEditState::default(),
            form_notes: String::new(),
            form_notes_cursor: akar_components::TextEditState::default(),
            form_notes_scroll_y: 0.0,
            form_agreed: false,
            form_theme_idx: 0,
            form_notifications_on: true,
            form_font_size: 16.0,
            form_language_idx: 0,
            form_language_open: false,
            form_container,
            form_name_node,
            form_notes_node,
            form_agreement_node,
            form_radio_nodes,
            form_notifications_node,
            form_font_size_node,
            form_language_node,
            form_submit_node,
            heading_node,
            paragraph_node,
            link_node,
            card_root,
            card_header,
            card_body,
            card_footer,
            card_slots,
            i18n_container,
            i18n_cjk_label_node,
            i18n_cjk_node,
            i18n_arabic_label_node,
            i18n_arabic_node,
            i18n_cjk_font,
            i18n_arabic_font,
            link_result: LinkResult {
                clicked: false,
                hovered: false,
                pressed: false,
            },
            needs_repaint: false,
            force_caret_visible: self.script_runner.is_some(),
            drawer_root,
            modal_root,
            toasts_root,
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

                // Acquire the swapchain texture before doing ANY frame work (script
                // advancement, layout, rendering). Surface acquisition can transiently
                // fail (Outdated/Lost/Timeout/Occluded/Validation) on early frames.
                // `AkarCore::end_frame` is where queued single-frame input (typed
                // chars, key events, paste events) gets cleared; every other code
                // path in this handler runs *before* end_frame. If a script step were
                // consumed and rendered before this acquisition failed, the frame
                // would be discarded without ever calling end_frame, leaving that
                // input un-cleared — the next RedrawRequested would render again and
                // re-apply the same still-queued input (e.g. a scripted `type "A"`
                // landing twice). Bailing out here, before anything is touched, keeps
                // a failed acquisition a clean no-op that retries correctly.
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

                state.core.begin_frame(size.width, size.height, scale);
                let needs_recording = (self.dump_frame_path.is_some() && !self.dump_frame_written)
                    || self.isolated_component.is_some();
                if needs_recording {
                    state.core.draw_list.start_recording();
                }
                let viewport_rect = [
                    0.0,
                    0.0,
                    size.width as f32 / scale,
                    size.height as f32 / scale,
                ];

                if let Some(component) = &self.isolated_component {
                    if !self.forced_initial_state {
                        component.force_state_initial(state);
                        self.forced_initial_state = true;
                    }
                    if matches!(component, Component::Navbar) {
                        ensure_navbar_slots(state);
                    }
                }

                prepare_layout(state, size, scale, self.isolated_component.as_ref());

                if self.dump_layout {
                    if !self.dump_layout_written {
                        self.dump_layout_written = true;

                        // Widgets whose layout children are wired lazily (e.g. navbar
                        // slots) or that only enter the tree for the currently active
                        // tab are otherwise invisible to a single-pass dump: their
                        // registered label resolves against a node that was never
                        // added as a child this frame, producing a misleading
                        // "0 0 0 0" that looks like a legitimate zero-sized rect
                        // rather than "not laid out." Without `--component`, run one
                        // full-page render plus a pass over every tab so every
                        // registered label gets a real chance to resolve.
                        let mut rects: std::collections::BTreeMap<String, [f32; 4]> =
                            std::collections::BTreeMap::new();
                        if self.isolated_component.is_none() {
                            let original_tab = state.active_tab;
                            render_containers(state);
                            render_navbar(state, viewport_rect);
                            render_alert(state);
                            render_tab_bar(state);
                            for tab in 0..4 {
                                state.active_tab = tab;
                                state.prev_active_tab = tab;
                                prepare_layout(state, size, scale, None);
                                match tab {
                                    0 => render_list_tab(state, viewport_rect),
                                    1 => render_canvas_tab(state),
                                    2 => render_stats_tab(state),
                                    3 => render_form_tab(state, viewport_rect),
                                    _ => {}
                                }
                                for (name, rect) in state.layout.labeled_rects() {
                                    let entry = rects.entry(name).or_insert([0.0, 0.0, 0.0, 0.0]);
                                    if *entry == [0.0, 0.0, 0.0, 0.0] {
                                        *entry = rect;
                                    }
                                }
                            }
                            state.active_tab = original_tab;
                            state.prev_active_tab = original_tab;
                        } else {
                            for (name, rect) in state.layout.labeled_rects() {
                                rects.insert(name, rect);
                            }
                        }

                        for (name, rect) in &rects {
                            println!("{} {} {} {} {}", name, rect[0], rect[1], rect[2], rect[3]);
                        }
                    }
                    event_loop.exit();
                    return;
                }

                let script_capture_path = if let Some(runner) = self.script_runner.as_mut() {
                    runner.advance(
                        &mut state.core.input,
                        &mut state.core.text_edit_keybindings,
                        &state.layout,
                        Instant::now(),
                    )
                } else {
                    None
                };

                if let Some(component) = &self.isolated_component {
                    component.render(state, viewport_rect);
                } else {
                    render_all(state, viewport_rect);
                }

                let normal_capture = !self.screenshot_taken
                    && self.screenshot_path.is_some()
                    && self.start_time.is_some_and(|t| {
                        t.elapsed() >= std::time::Duration::from_secs_f64(self.delay_secs)
                    });
                let is_capture_frame = normal_capture || script_capture_path.is_some();

                if is_capture_frame {
                    state.core.request_screenshot();
                }

                let mut encoder = state
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

                {
                    let surface_view = output
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    let render_view = if is_capture_frame {
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

                let is_standalone = self.screenshot_path.is_none() && self.script_runner.is_none();
                let is_dump_frame = self.dump_frame_path.is_some()
                    && !self.dump_frame_written
                    && (is_capture_frame || is_standalone);
                if is_dump_frame {
                    let dump = FrameDump {
                        recorded_calls: state.core.draw_list.recorded_calls(),
                        labeled_rects: state.layout.labeled_rects(),
                        input: input_snapshot(&state.core.input),
                    };
                    let path = self.dump_frame_path.clone().unwrap();
                    match std::fs::File::create(&path) {
                        Ok(file) => {
                            if let Err(e) = serde_json::to_writer_pretty(file, &dump) {
                                eprintln!("Failed to write frame dump '{path}': {e}");
                                std::process::exit(1);
                            }
                            eprintln!("Frame dump written to {path}");
                        }
                        Err(e) => {
                            eprintln!("Failed to create file '{path}': {e}");
                            std::process::exit(1);
                        }
                    }
                    self.dump_frame_written = true;
                    state.core.draw_list.stop_recording();
                    if self.exit_after && is_standalone {
                        event_loop.exit();
                    }
                }

                if is_capture_frame {
                    let capture_path = if let Some(p) = &script_capture_path {
                        p.clone()
                    } else {
                        self.screenshot_path.clone().unwrap()
                    };
                    let captured =
                        state
                            .core
                            .take_screenshot(&state.device, &state.queue, encoder, &output);
                    match captured {
                        Ok(frame) => {
                            let path = &capture_path;
                            let mut cropped = false;
                            if let Some(_component) = &self.isolated_component {
                                let recorded = state.core.draw_list.recorded_calls();
                                let scale = state.window.scale_factor() as f32;
                                let crop_params = compute_component_aabb(recorded).map(|aabb| {
                                    let pad = 16.0 * scale;
                                    let x = (aabb[0] - pad).max(0.0) as u32;
                                    let y = (aabb[1] - pad).max(0.0) as u32;
                                    let right =
                                        (aabb[0] + aabb[2] + pad).min(frame.width as f32) as u32;
                                    let bottom =
                                        (aabb[1] + aabb[3] + pad).min(frame.height as f32) as u32;
                                    let w = right.saturating_sub(x);
                                    let h = bottom.saturating_sub(y);
                                    (x, y, w, h)
                                });
                                state.core.draw_list.stop_recording();
                                if let Some((x, y, w, h)) = crop_params {
                                    if w > 0 && h > 0 {
                                        if let Err(e) = crop_and_write_png(&frame, x, y, w, h, path)
                                        {
                                            eprintln!("Failed to crop PNG: {e}");
                                            std::process::exit(1);
                                        }
                                        eprintln!("Cropped screenshot saved to {path}");
                                        cropped = true;
                                    }
                                }
                            }
                            if !cropped {
                                match std::fs::File::create(path) {
                                    Ok(file) => {
                                        let mut png_encoder =
                                            png::Encoder::new(file, frame.width, frame.height);
                                        png_encoder.set_color(png::ColorType::Rgba);
                                        png_encoder.set_depth(png::BitDepth::Eight);
                                        match png_encoder.write_header() {
                                            Ok(mut writer) => {
                                                if let Err(e) = writer.write_image_data(&frame.rgba)
                                                {
                                                    eprintln!("Failed to write PNG data: {e}");
                                                    std::process::exit(1);
                                                }
                                                eprintln!("Screenshot saved to {path}");
                                            }
                                            Err(e) => {
                                                eprintln!("Failed to write PNG header: {e}");
                                                std::process::exit(1);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to create file '{path}': {e}");
                                        std::process::exit(1);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Screenshot failed: {e}");
                            std::process::exit(1);
                        }
                    }
                    if normal_capture {
                        self.screenshot_taken = true;
                        if self.exit_after {
                            event_loop.exit();
                        }
                    }
                } else {
                    state.queue.submit(std::iter::once(encoder.finish()));
                }
                output.present();

                if state.needs_repaint {
                    state.needs_repaint = false;
                    state.window.request_redraw();
                }
            }
            _ => {}
        }

        if let Some(runner) = &self.script_runner {
            if self.exit_after && runner.is_exhausted() {
                event_loop.exit();
            } else if !runner.is_exhausted() {
                state.window.request_redraw();
            }
        }

        process_window_event(&mut state.core.input, &event);

        if !matches!(event, WindowEvent::RedrawRequested) {
            state.window.request_redraw();
        }
    }
}

#[cfg(test)]
mod aabb_tests {
    use super::compute_component_aabb;
    use akar_core::draw_list::{QuadCall, RecordedCall, TextCall};
    use akar_core::{DrawCall, Z_SCRIM};

    fn quad_call(x: f32, y: f32, w: f32, h: f32, z: f32) -> RecordedCall {
        let q = QuadCall {
            rect: [x, y, w, h],
            fill: [1.0, 0.0, 0.0, 1.0],
            border_color: [0.0; 4],
            corner_radii: [0.0; 4],
            border_width: 0.0,
            z,
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            shadow_color: [0.0; 4],
            shadow_offset: [0.0; 2],
            _pad: [0.0; 2],
        };
        RecordedCall {
            call: DrawCall::Quad(q),
            scissor: None,
            physical_rect: [x, y, w, h],
        }
    }

    fn text_call(x: f32, y: f32, w: f32, h: f32, z: f32) -> RecordedCall {
        let t = TextCall {
            buffer_id: 0,
            x,
            y,
            clip: [x, y, w, h],
            color: [1.0; 4],
            z,
        };
        RecordedCall {
            call: DrawCall::Text(t),
            scissor: None,
            physical_rect: [x, y, w, h],
        }
    }

    fn with_scissor(mut rc: RecordedCall, scissor: [f32; 4]) -> RecordedCall {
        rc.scissor = Some(scissor);
        rc
    }

    fn with_physical_rect(mut rc: RecordedCall, pr: [f32; 4]) -> RecordedCall {
        rc.physical_rect = pr;
        rc
    }

    #[test]
    fn empty_returns_none() {
        assert!(compute_component_aabb(&[]).is_none());
    }

    #[test]
    fn single_quad_scale_1() {
        let calls = [quad_call(10.0, 20.0, 30.0, 40.0, 0.0)];
        let aabb = compute_component_aabb(&calls).unwrap();
        assert_eq!(aabb, [10.0, 20.0, 30.0, 40.0]);
    }

    #[test]
    fn single_quad_scale_2() {
        let calls = [with_physical_rect(
            quad_call(10.0, 20.0, 30.0, 40.0, 0.0),
            [20.0, 40.0, 60.0, 80.0],
        )];
        let aabb = compute_component_aabb(&calls).unwrap();
        assert_eq!(aabb, [20.0, 40.0, 60.0, 80.0]);
    }

    #[test]
    fn single_text_scale_1() {
        let calls = [text_call(5.0, 10.0, 50.0, 20.0, 0.0)];
        let aabb = compute_component_aabb(&calls).unwrap();
        assert_eq!(aabb, [5.0, 10.0, 50.0, 20.0]);
    }

    #[test]
    fn single_text_scale_2() {
        let calls = [with_physical_rect(
            text_call(5.0, 10.0, 50.0, 20.0, 0.0),
            [10.0, 20.0, 100.0, 40.0],
        )];
        let aabb = compute_component_aabb(&calls).unwrap();
        assert_eq!(aabb, [10.0, 20.0, 100.0, 40.0]);
    }

    #[test]
    fn mixed_quad_and_text() {
        let calls = [
            quad_call(10.0, 10.0, 20.0, 20.0, 0.0),
            text_call(50.0, 50.0, 30.0, 10.0, 0.0),
        ];
        let aabb = compute_component_aabb(&calls).unwrap();
        assert_eq!(aabb[0], 10.0);
        assert_eq!(aabb[1], 10.0);
        assert_eq!(aabb[2], 70.0);
        assert_eq!(aabb[3], 50.0);
    }

    #[test]
    fn mixed_at_scale_2() {
        let calls = [
            with_physical_rect(
                quad_call(10.0, 10.0, 20.0, 20.0, 0.0),
                [20.0, 20.0, 40.0, 40.0],
            ),
            with_physical_rect(
                text_call(50.0, 50.0, 30.0, 10.0, 0.0),
                [100.0, 100.0, 60.0, 20.0],
            ),
        ];
        let aabb = compute_component_aabb(&calls).unwrap();
        assert_eq!(aabb[0], 20.0);
        assert_eq!(aabb[1], 20.0);
        assert_eq!(aabb[2], 140.0);
        assert_eq!(aabb[3], 100.0);
    }

    #[test]
    fn scissored_out_excluded() {
        let calls = [with_scissor(
            quad_call(200.0, 200.0, 50.0, 50.0, 0.0),
            [0.0, 0.0, 100.0, 100.0],
        )];
        assert!(compute_component_aabb(&calls).is_none());
    }

    #[test]
    fn scissored_partial_included() {
        let calls = [with_scissor(
            quad_call(80.0, 80.0, 50.0, 50.0, 0.0),
            [0.0, 0.0, 200.0, 200.0],
        )];
        let aabb = compute_component_aabb(&calls).unwrap();
        assert_eq!(aabb, [80.0, 80.0, 50.0, 50.0]);
    }

    #[test]
    fn scrim_filtered() {
        let calls = [
            quad_call(10.0, 10.0, 20.0, 20.0, 0.0),
            quad_call(0.0, 0.0, 800.0, 600.0, Z_SCRIM),
        ];
        let aabb = compute_component_aabb(&calls).unwrap();
        assert_eq!(aabb, [10.0, 10.0, 20.0, 20.0]);
    }

    #[test]
    fn all_scrim_returns_none() {
        let calls = [quad_call(0.0, 0.0, 800.0, 600.0, Z_SCRIM)];
        assert!(compute_component_aabb(&calls).is_none());
    }

    #[test]
    fn uses_physical_rect_not_inner_call() {
        let mut rc = quad_call(10.0, 20.0, 30.0, 40.0, 0.0);
        rc.physical_rect = [100.0, 200.0, 300.0, 400.0];
        let aabb = compute_component_aabb(&[rc]).unwrap();
        assert_eq!(aabb, [100.0, 200.0, 300.0, 400.0]);
    }
}
