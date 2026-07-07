#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_void};
use std::ptr;

use akar_components::{AkarTheme, ButtonVariant, AKAR_THEME_DARK};
use akar_core::{AkarCore, Key};
use akar_layout::Layout;

pub struct AkarCtx {
    core: AkarCore,
    layout: Layout,
    theme: AkarTheme,
    device: *const wgpu::Device,
    queue: *const wgpu::Queue,
}

unsafe impl Send for AkarCtx {}
unsafe impl Sync for AkarCtx {}

#[repr(C)]
pub struct AkarButtonResult {
    pub clicked: bool,
    pub hovered: bool,
    pub pressed: bool,
}

fn texture_format_from_raw(raw: u32) -> Option<wgpu::TextureFormat> {
    match raw {
        0 => Some(wgpu::TextureFormat::R8Unorm),
        1 => Some(wgpu::TextureFormat::R8Snorm),
        2 => Some(wgpu::TextureFormat::R8Uint),
        3 => Some(wgpu::TextureFormat::R8Sint),
        4 => Some(wgpu::TextureFormat::R16Uint),
        5 => Some(wgpu::TextureFormat::R16Sint),
        6 => Some(wgpu::TextureFormat::R16Unorm),
        7 => Some(wgpu::TextureFormat::R16Snorm),
        8 => Some(wgpu::TextureFormat::R16Float),
        9 => Some(wgpu::TextureFormat::Rg8Unorm),
        10 => Some(wgpu::TextureFormat::Rg8Snorm),
        11 => Some(wgpu::TextureFormat::Rg8Uint),
        12 => Some(wgpu::TextureFormat::Rg8Sint),
        13 => Some(wgpu::TextureFormat::R32Uint),
        14 => Some(wgpu::TextureFormat::R32Sint),
        15 => Some(wgpu::TextureFormat::R32Float),
        16 => Some(wgpu::TextureFormat::Rg16Uint),
        17 => Some(wgpu::TextureFormat::Rg16Sint),
        18 => Some(wgpu::TextureFormat::Rg16Unorm),
        19 => Some(wgpu::TextureFormat::Rg16Snorm),
        20 => Some(wgpu::TextureFormat::Rg16Float),
        21 => Some(wgpu::TextureFormat::Rgba8Unorm),
        22 => Some(wgpu::TextureFormat::Rgba8UnormSrgb),
        23 => Some(wgpu::TextureFormat::Rgba8Snorm),
        24 => Some(wgpu::TextureFormat::Rgba8Uint),
        25 => Some(wgpu::TextureFormat::Rgba8Sint),
        26 => Some(wgpu::TextureFormat::Bgra8Unorm),
        27 => Some(wgpu::TextureFormat::Bgra8UnormSrgb),
        28 => Some(wgpu::TextureFormat::Rgb9e5Ufloat),
        29 => Some(wgpu::TextureFormat::Rgb10a2Uint),
        30 => Some(wgpu::TextureFormat::Rgb10a2Unorm),
        31 => Some(wgpu::TextureFormat::Rg11b10Ufloat),
        32 => Some(wgpu::TextureFormat::R64Uint),
        33 => Some(wgpu::TextureFormat::Rg32Uint),
        34 => Some(wgpu::TextureFormat::Rg32Sint),
        35 => Some(wgpu::TextureFormat::Rg32Float),
        36 => Some(wgpu::TextureFormat::Rgba16Uint),
        37 => Some(wgpu::TextureFormat::Rgba16Sint),
        38 => Some(wgpu::TextureFormat::Rgba16Unorm),
        39 => Some(wgpu::TextureFormat::Rgba16Snorm),
        40 => Some(wgpu::TextureFormat::Rgba16Float),
        41 => Some(wgpu::TextureFormat::Rgba32Uint),
        42 => Some(wgpu::TextureFormat::Rgba32Sint),
        43 => Some(wgpu::TextureFormat::Rgba32Float),
        44 => Some(wgpu::TextureFormat::Stencil8),
        45 => Some(wgpu::TextureFormat::Depth16Unorm),
        46 => Some(wgpu::TextureFormat::Depth24Plus),
        47 => Some(wgpu::TextureFormat::Depth24PlusStencil8),
        48 => Some(wgpu::TextureFormat::Depth32Float),
        49 => Some(wgpu::TextureFormat::Depth32FloatStencil8),
        50 => Some(wgpu::TextureFormat::NV12),
        51 => Some(wgpu::TextureFormat::P010),
        52 => Some(wgpu::TextureFormat::Bc1RgbaUnorm),
        53 => Some(wgpu::TextureFormat::Bc1RgbaUnormSrgb),
        54 => Some(wgpu::TextureFormat::Bc2RgbaUnorm),
        55 => Some(wgpu::TextureFormat::Bc2RgbaUnormSrgb),
        56 => Some(wgpu::TextureFormat::Bc3RgbaUnorm),
        57 => Some(wgpu::TextureFormat::Bc3RgbaUnormSrgb),
        58 => Some(wgpu::TextureFormat::Bc4RUnorm),
        59 => Some(wgpu::TextureFormat::Bc4RSnorm),
        60 => Some(wgpu::TextureFormat::Bc5RgUnorm),
        61 => Some(wgpu::TextureFormat::Bc5RgSnorm),
        62 => Some(wgpu::TextureFormat::Bc6hRgbUfloat),
        63 => Some(wgpu::TextureFormat::Bc6hRgbFloat),
        64 => Some(wgpu::TextureFormat::Bc7RgbaUnorm),
        65 => Some(wgpu::TextureFormat::Bc7RgbaUnormSrgb),
        66 => Some(wgpu::TextureFormat::Etc2Rgb8Unorm),
        67 => Some(wgpu::TextureFormat::Etc2Rgb8UnormSrgb),
        68 => Some(wgpu::TextureFormat::Etc2Rgb8A1Unorm),
        69 => Some(wgpu::TextureFormat::Etc2Rgb8A1UnormSrgb),
        70 => Some(wgpu::TextureFormat::Etc2Rgba8Unorm),
        71 => Some(wgpu::TextureFormat::Etc2Rgba8UnormSrgb),
        72 => Some(wgpu::TextureFormat::EacR11Unorm),
        73 => Some(wgpu::TextureFormat::EacR11Snorm),
        74 => Some(wgpu::TextureFormat::EacRg11Unorm),
        75 => Some(wgpu::TextureFormat::EacRg11Snorm),
        _ => None,
    }
}

#[no_mangle]
pub unsafe extern "C" fn akar_ctx_new(
    device: *const c_void,
    queue: *const c_void,
    surface_format_raw: u32,
) -> *mut AkarCtx {
    if device.is_null() || queue.is_null() {
        return ptr::null_mut();
    }

    let Some(format) = texture_format_from_raw(surface_format_raw) else {
        return ptr::null_mut();
    };

    let device_ref = unsafe { &*(device as *const wgpu::Device) };
    let queue_ref = unsafe { &*(queue as *const wgpu::Queue) };

    let core = AkarCore::new(device_ref, queue_ref, format);
    let layout = Layout::new();
    let theme = AKAR_THEME_DARK;

    Box::into_raw(Box::new(AkarCtx {
        core,
        layout,
        theme,
        device: device as *const wgpu::Device,
        queue: queue as *const wgpu::Queue,
    }))
}

#[no_mangle]
pub unsafe extern "C" fn akar_ctx_free(ctx: *mut AkarCtx) {
    if !ctx.is_null() {
        unsafe { drop(Box::from_raw(ctx)) };
    }
}

/// Creates a headless context suitable for testing layout and input logic.
/// The GPU pipeline is initialized against a headless wgpu adapter; no surface
/// or real window is required. Do not call `akar_end_frame` on a mock context.
#[no_mangle]
pub unsafe extern "C" fn akar_ctx_mock() -> *mut AkarCtx {
    let core = AkarCore::mock();
    let layout = Layout::new();
    let theme = AKAR_THEME_DARK;
    Box::into_raw(Box::new(AkarCtx {
        core,
        layout,
        theme,
        device: std::ptr::null(),
        queue: std::ptr::null(),
    }))
}

#[no_mangle]
pub unsafe extern "C" fn akar_begin_frame(
    ctx: *mut AkarCtx,
    width: u32,
    height: u32,
    scale_factor: f32,
) {
    let ctx = unsafe { &mut *ctx };
    ctx.core.begin_frame(width, height, scale_factor);
}

#[no_mangle]
pub unsafe extern "C" fn akar_end_frame(ctx: *mut AkarCtx, pass: *mut c_void) {
    let ctx = unsafe { &mut *ctx };
    if pass.is_null() || ctx.device.is_null() || ctx.queue.is_null() {
        return;
    }
    let device = unsafe { &*ctx.device };
    let queue = unsafe { &*ctx.queue };
    let pass = unsafe { &mut *(pass as *mut wgpu::RenderPass<'_>) };
    let _ = ctx.core.end_frame(device, queue, pass);
}

#[no_mangle]
pub unsafe extern "C" fn akar_input_begin(ctx: *mut AkarCtx) {
    let ctx = unsafe { &mut *ctx };
    ctx.core.input.begin_frame();
}

#[no_mangle]
pub unsafe extern "C" fn akar_set_mouse_pos(ctx: *mut AkarCtx, x: f32, y: f32) {
    let ctx = unsafe { &mut *ctx };
    ctx.core.input.set_mouse_pos(x, y);
}

#[no_mangle]
pub unsafe extern "C" fn akar_push_mouse_button(ctx: *mut AkarCtx, button: u32, pressed: bool) {
    let ctx = unsafe { &mut *ctx };
    ctx.core.input.push_mouse_button(button as usize, pressed);
}

#[no_mangle]
pub unsafe extern "C" fn akar_push_scroll(ctx: *mut AkarCtx, dx: f32, dy: f32) {
    let ctx = unsafe { &mut *ctx };
    ctx.core.input.push_scroll(dx, dy);
}

#[no_mangle]
pub unsafe extern "C" fn akar_push_char(ctx: *mut AkarCtx, codepoint: u32) {
    let ctx = unsafe { &mut *ctx };
    if let Some(ch) = char::from_u32(codepoint) {
        ctx.core.input.push_char(ch);
    }
}

#[no_mangle]
pub unsafe extern "C" fn akar_input_end(_ctx: *mut AkarCtx) {}

#[repr(C)]
pub struct AkarRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[no_mangle]
pub unsafe extern "C" fn akar_new_leaf(ctx: *mut AkarCtx, flex_grow: f32) -> u64 {
    use akar_layout::Style;
    let ctx = unsafe { &mut *ctx };
    let style = Style {
        flex_grow,
        flex_shrink: 1.0,
        ..Default::default()
    };
    ctx.layout.new_leaf(style).into()
}

#[no_mangle]
pub unsafe extern "C" fn akar_new_fixed_leaf(ctx: *mut AkarCtx, w: f32, h: f32) -> u64 {
    use akar_layout::{length, Dimension, Size, Style};
    let ctx = unsafe { &mut *ctx };
    let style = Style {
        size: Size {
            width: if w > 0.0 {
                length(w)
            } else {
                Dimension::auto()
            },
            height: if h > 0.0 {
                length(h)
            } else {
                Dimension::auto()
            },
        },
        flex_shrink: 0.0,
        ..Default::default()
    };
    ctx.layout.new_leaf(style).into()
}

#[no_mangle]
pub unsafe extern "C" fn akar_new_flex_row(ctx: *mut AkarCtx) -> u64 {
    use akar_layout::{Dimension, Display, FlexDirection, Size, Style};
    let ctx = unsafe { &mut *ctx };
    let style = Style {
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        size: Size {
            width: Dimension::percent(1.0),
            height: Dimension::percent(1.0),
        },
        ..Default::default()
    };
    ctx.layout.new_with_children(style, &[]).into()
}

#[no_mangle]
pub unsafe extern "C" fn akar_new_flex_col(ctx: *mut AkarCtx) -> u64 {
    use akar_layout::{Dimension, Display, FlexDirection, Size, Style};
    let ctx = unsafe { &mut *ctx };
    let style = Style {
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        size: Size {
            width: Dimension::percent(1.0),
            height: Dimension::percent(1.0),
        },
        ..Default::default()
    };
    ctx.layout.new_with_children(style, &[]).into()
}

#[no_mangle]
pub unsafe extern "C" fn akar_add_child(ctx: *mut AkarCtx, parent: u64, child: u64) {
    let ctx = unsafe { &mut *ctx };
    let parent_node: akar_layout::NodeId = parent.into();
    let child_node: akar_layout::NodeId = child.into();
    ctx.layout.add_child(parent_node, child_node);
}

#[no_mangle]
pub unsafe extern "C" fn akar_layout_compute(
    ctx: *mut AkarCtx,
    root: u64,
    width: f32,
    height: f32,
) {
    use akar_layout::Size;
    let ctx = unsafe { &mut *ctx };
    let root_node: akar_layout::NodeId = root.into();
    ctx.layout
        .compute(root_node, (Some(width), Some(height)), |_, _, _, _, _| {
            Size::ZERO
        });
}

#[no_mangle]
pub unsafe extern "C" fn akar_layout_rect(ctx: *mut AkarCtx, node: u64) -> AkarRect {
    let ctx = unsafe { &mut *ctx };
    let node_id: akar_layout::NodeId = node.into();
    let [x, y, w, h] = ctx.layout.rect(node_id);
    AkarRect { x, y, w, h }
}

#[no_mangle]
pub unsafe extern "C" fn akar_button(
    ctx: *mut AkarCtx,
    node_id: u64,
    label: *const c_char,
    label_len: i32,
) -> AkarButtonResult {
    let ctx = unsafe { &mut *ctx };

    if label.is_null() || label_len <= 0 {
        return AkarButtonResult {
            clicked: false,
            hovered: false,
            pressed: false,
        };
    }

    let label_bytes = unsafe { std::slice::from_raw_parts(label as *const u8, label_len as usize) };
    let Ok(label_str) = std::str::from_utf8(label_bytes) else {
        return AkarButtonResult {
            clicked: false,
            hovered: false,
            pressed: false,
        };
    };

    let nid: akar_layout::NodeId = node_id.into();
    let result = akar_components::akar_button(
        &mut ctx.core,
        &ctx.layout,
        nid,
        label_str,
        ButtonVariant::Solid,
        &ctx.theme,
    );

    AkarButtonResult {
        clicked: result.clicked,
        hovered: result.hovered,
        pressed: result.pressed,
    }
}

#[no_mangle]
pub unsafe extern "C" fn akar_label(
    ctx: *mut AkarCtx,
    node_id: u64,
    text: *const c_char,
    text_len: i32,
    color: u32,
) {
    let ctx = unsafe { &mut *ctx };

    if text.is_null() || text_len <= 0 {
        return;
    }

    let bytes = unsafe { std::slice::from_raw_parts(text as *const u8, text_len as usize) };
    let Ok(text_str) = std::str::from_utf8(bytes) else {
        return;
    };

    let nid: akar_layout::NodeId = node_id.into();
    akar_components::akar_label(&mut ctx.core, &ctx.layout, nid, text_str, color, &ctx.theme);
}

#[repr(C)]
pub struct AkarBoxStyle {
    pub fill: u32,
    pub border_color: u32,
    pub border_width: f32,
    pub corner_radii: [f32; 4],
    pub shadow_color: u32,
    pub shadow_offset: [f32; 2],
    pub shadow_blur: f32,
    pub shadow_spread: f32,
}

#[no_mangle]
pub unsafe extern "C" fn akar_container(ctx: *mut AkarCtx, node_id: u64, style: AkarBoxStyle) {
    let ctx = unsafe { &mut *ctx };
    let nid: akar_layout::NodeId = node_id.into();

    let shadow = if (style.shadow_color & 0xFF) > 0 {
        Some(akar_components::BoxShadow {
            color: style.shadow_color,
            offset: style.shadow_offset,
            blur: style.shadow_blur,
            spread: style.shadow_spread,
        })
    } else {
        None
    };

    let box_style = akar_components::BoxStyle {
        fill: style.fill,
        border_color: style.border_color,
        border_width: style.border_width,
        corner_radii: style.corner_radii,
        shadow,
    };

    akar_components::akar_container(&mut ctx.core, &ctx.layout, nid, &box_style);
}

#[repr(C)]
pub struct AkarDrawerResponse {
    pub close_requested: bool,
}

#[no_mangle]
pub unsafe extern "C" fn akar_drawer_begin(
    ctx: *mut AkarCtx,
    edge: u32,
    panel_width: f32,
    viewport_rect: *const f32,
) -> AkarDrawerResponse {
    let ctx = unsafe { &mut *ctx };
    let rect = unsafe { *(viewport_rect as *const [f32; 4]) };
    let drawer_edge = match edge {
        1 => akar_components::DrawerEdge::Right,
        _ => akar_components::DrawerEdge::Left,
    };
    let result = akar_components::drawer_begin(&mut ctx.core, rect, drawer_edge, panel_width, &ctx.theme);
    AkarDrawerResponse {
        close_requested: result.close_requested,
    }
}

#[no_mangle]
pub unsafe extern "C" fn akar_drawer_end(ctx: *mut AkarCtx) {
    let ctx = unsafe { &mut *ctx };
    akar_components::drawer_end(&mut ctx.core);
}

#[no_mangle]
pub unsafe extern "C" fn akar_set_padding(
    ctx: *mut AkarCtx,
    node_id: u64,
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,
) {
    let ctx = unsafe { &mut *ctx };
    let nid: akar_layout::NodeId = node_id.into();
    ctx.layout.set_padding(nid, top, right, bottom, left);
}

#[no_mangle]
pub unsafe extern "C" fn akar_set_margin(
    ctx: *mut AkarCtx,
    node_id: u64,
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,
) {
    let ctx = unsafe { &mut *ctx };
    let nid: akar_layout::NodeId = node_id.into();
    ctx.layout.set_margin(nid, top, right, bottom, left);
}

#[repr(C)]
pub struct AkarRange {
    pub start: u32,
    pub end: u32,
}

#[no_mangle]
pub extern "C" fn akar_list_clip(
    total: u32,
    item_height: f32,
    scroll_y: f32,
    viewport_height: f32,
) -> AkarRange {
    let r = akar_core::list_clip(total as usize, item_height, scroll_y, viewport_height);
    AkarRange {
        start: r.start as u32,
        end: r.end as u32,
    }
}

#[no_mangle]
pub unsafe extern "C" fn akar_scroll_area_begin(
    ctx: *mut AkarCtx,
    rect: *const f32,
    scroll_y: *mut f32,
    content_height: f32,
) -> f32 {
    let ctx = unsafe { &mut *ctx };
    let rect = unsafe { *(rect as *const [f32; 4]) };
    let resp = akar_components::scroll_area_begin(
        &mut ctx.core,
        rect,
        unsafe { &mut *scroll_y },
        content_height,
    );
    resp.content_y
}

#[no_mangle]
pub unsafe extern "C" fn akar_scroll_area_end(ctx: *mut AkarCtx) {
    let ctx = unsafe { &mut *ctx };
    akar_components::scroll_area_end(&mut ctx.core);
}

#[no_mangle]
pub unsafe extern "C" fn akar_progress(
    ctx: *mut AkarCtx,
    node_id: u64,
    value: f32,
    track_color: u32,
    fill_color: u32,
    corner_radius: f32,
) {
    let ctx = unsafe { &mut *ctx };
    let nid: akar_layout::NodeId = node_id.into();
    let style = akar_components::ProgressStyle {
        track_color,
        fill_color,
        corner_radius,
    };
    akar_components::akar_progress(&mut ctx.core, &ctx.layout, nid, value, &style);
}

#[no_mangle]
pub unsafe extern "C" fn akar_badge(
    ctx: *mut AkarCtx,
    node_id: u64,
    text: *const std::ffi::c_char,
    variant: u32,
) {
    if text.is_null() {
        return;
    }
    let ctx = unsafe { &mut *ctx };
    let nid: akar_layout::NodeId = node_id.into();
    let text = unsafe { std::ffi::CStr::from_ptr(text) }
        .to_str()
        .unwrap_or("");
    let variant = match variant {
        1 => akar_components::BadgeVariant::Primary,
        2 => akar_components::BadgeVariant::Success,
        3 => akar_components::BadgeVariant::Warning,
        4 => akar_components::BadgeVariant::Error,
        5 => akar_components::BadgeVariant::Info,
        _ => akar_components::BadgeVariant::Default,
    };
    akar_components::akar_badge(&mut ctx.core, &ctx.layout, nid, text, variant, &ctx.theme);
}

#[repr(C)]
pub struct AkarAlertResult {
    pub dismissed: bool,
}

#[no_mangle]
pub unsafe extern "C" fn akar_alert(
    ctx: *mut AkarCtx,
    node_id: u64,
    text: *const c_char,
    text_len: i32,
    variant: u32,
    closable: bool,
) -> AkarAlertResult {
    let ctx = unsafe { &mut *ctx };

    if text.is_null() || text_len <= 0 {
        return AkarAlertResult { dismissed: false };
    }

    let bytes = unsafe { std::slice::from_raw_parts(text as *const u8, text_len as usize) };
    let Ok(text_str) = std::str::from_utf8(bytes) else {
        return AkarAlertResult { dismissed: false };
    };

    let variant = match variant {
        0 => akar_components::AlertVariant::Info,
        1 => akar_components::AlertVariant::Success,
        2 => akar_components::AlertVariant::Warning,
        3 => akar_components::AlertVariant::Error,
        _ => akar_components::AlertVariant::Info,
    };

    let nid: akar_layout::NodeId = node_id.into();
    let result = akar_components::akar_alert(
        &mut ctx.core,
        &ctx.layout,
        nid,
        text_str,
        variant,
        closable,
        &ctx.theme,
    );

    AkarAlertResult {
        dismissed: result.dismissed,
    }
}

#[no_mangle]
pub unsafe extern "C" fn akar_stat(
    ctx: *mut AkarCtx,
    node_id: u64,
    title: *const c_char,
    title_len: i32,
    value: *const c_char,
    value_len: i32,
    description: *const c_char,
    description_len: i32,
) {
    let ctx = unsafe { &mut *ctx };

    if title.is_null() || title_len <= 0 || value.is_null() || value_len <= 0 {
        return;
    }

    let title_bytes = unsafe { std::slice::from_raw_parts(title as *const u8, title_len as usize) };
    let Ok(title_str) = std::str::from_utf8(title_bytes) else {
        return;
    };

    let value_bytes = unsafe { std::slice::from_raw_parts(value as *const u8, value_len as usize) };
    let Ok(value_str) = std::str::from_utf8(value_bytes) else {
        return;
    };

    let description_str = if description.is_null() || description_len <= 0 {
        None
    } else {
        let desc_bytes = unsafe {
            std::slice::from_raw_parts(description as *const u8, description_len as usize)
        };
        std::str::from_utf8(desc_bytes).ok()
    };

    let nid: akar_layout::NodeId = node_id.into();
    akar_components::akar_stat(
        &mut ctx.core,
        &ctx.layout,
        nid,
        title_str,
        value_str,
        description_str,
        &ctx.theme,
    );
}

#[no_mangle]
pub unsafe extern "C" fn akar_skeleton(ctx: *mut AkarCtx, node_id: u64, variant: u32) {
    let ctx = unsafe { &mut *ctx };

    let variant = match variant {
        0 => akar_components::SkeletonVariant::Text,
        1 => akar_components::SkeletonVariant::Card,
        2 => akar_components::SkeletonVariant::Circle,
        _ => akar_components::SkeletonVariant::Text,
    };

    let nid: akar_layout::NodeId = node_id.into();
    akar_components::akar_skeleton(&mut ctx.core, &ctx.layout, nid, variant, &ctx.theme);
}

#[repr(C)]
pub struct AkarNavbarSlots {
    pub start: u64,
    pub center: u64,
    pub end: u64,
}

#[no_mangle]
pub unsafe extern "C" fn akar_navbar(ctx: *mut AkarCtx, node_id: u64) -> AkarNavbarSlots {
    let ctx = unsafe { &mut *ctx };
    let nid: akar_layout::NodeId = node_id.into();
    let slots = akar_components::akar_navbar(&mut ctx.core, &mut ctx.layout, nid, &ctx.theme);
    AkarNavbarSlots {
        start: slots.start.into(),
        center: slots.center.into(),
        end: slots.end.into(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn akar_steps(
    ctx: *mut AkarCtx,
    node_id: u64,
    labels: *const *const c_char,
    label_count: u32,
    label_lengths: *const i32,
    current: u32,
) {
    let ctx = unsafe { &mut *ctx };

    if labels.is_null() || label_lengths.is_null() || label_count == 0 {
        return;
    }

    let mut label_strs: Vec<&str> = Vec::with_capacity(label_count as usize);
    for i in 0..label_count as usize {
        let ptr = unsafe { *labels.add(i) };
        let len = unsafe { *label_lengths.add(i) };
        if ptr.is_null() || len <= 0 {
            return;
        }
        let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };
        match std::str::from_utf8(bytes) {
            Ok(s) => label_strs.push(s),
            Err(_) => return,
        }
    }

    let nid: akar_layout::NodeId = node_id.into();
    akar_components::akar_steps(
        &mut ctx.core,
        &ctx.layout,
        nid,
        &label_strs,
        current as usize,
        &ctx.theme,
    );
}

#[repr(C)]
pub struct AkarTabBarResponse {
    pub clicked_index: i32,
}

#[no_mangle]
pub unsafe extern "C" fn akar_tab_bar(
    ctx: *mut AkarCtx,
    node_id: u64,
    labels: *const *const c_char,
    label_count: u32,
    label_lengths: *const i32,
    active_index: u32,
    variant: u32,
) -> AkarTabBarResponse {
    let ctx = unsafe { &mut *ctx };

    if labels.is_null() || label_lengths.is_null() || label_count == 0 {
        return AkarTabBarResponse { clicked_index: -1 };
    }

    let mut label_strs: Vec<&str> = Vec::with_capacity(label_count as usize);
    for i in 0..label_count as usize {
        let ptr = unsafe { *labels.add(i) };
        let len = unsafe { *label_lengths.add(i) };
        if ptr.is_null() || len <= 0 {
            return AkarTabBarResponse { clicked_index: -1 };
        }
        let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };
        match std::str::from_utf8(bytes) {
            Ok(s) => label_strs.push(s),
            Err(_) => return AkarTabBarResponse { clicked_index: -1 },
        }
    }

    let nid: akar_layout::NodeId = node_id.into();
    let tab_variant = match variant {
        1 => akar_components::TabVariant::Lifted,
        2 => akar_components::TabVariant::Pills,
        3 => akar_components::TabVariant::Underline,
        _ => akar_components::TabVariant::Boxed,
    };

    let result = akar_components::akar_tab_bar(
        &mut ctx.core,
        &ctx.layout,
        nid,
        &label_strs,
        active_index as usize,
        tab_variant,
        &ctx.theme,
    );

    AkarTabBarResponse {
        clicked_index: result.clicked.map(|i| i as i32).unwrap_or(-1),
    }
}

#[no_mangle]
pub unsafe extern "C" fn akar_avatar(
    ctx: *mut AkarCtx,
    node_id: u64,
    initials: *const c_char,
    initials_len: i32,
    color: u32,
) {
    let ctx = unsafe { &mut *ctx };

    if initials.is_null() || initials_len <= 0 {
        return;
    }

    let bytes = unsafe { std::slice::from_raw_parts(initials as *const u8, initials_len as usize) };
    let Ok(initials_str) = std::str::from_utf8(bytes) else {
        return;
    };

    let color = if color == 0 { None } else { Some(color) };

    let nid: akar_layout::NodeId = node_id.into();
    akar_components::akar_avatar(
        &mut ctx.core,
        &ctx.layout,
        nid,
        initials_str,
        color,
        &ctx.theme,
    );
}

// ---- Tooltip ----

#[repr(C)]
pub struct AkarTooltipResponse {
    pub visible: bool,
}

#[no_mangle]
pub unsafe extern "C" fn akar_tooltip(
    ctx: *mut AkarCtx,
    trigger_rect: *const f32,
    text: *const c_char,
    preferred_side: u32,
    viewport_rect: *const f32,
) -> AkarTooltipResponse {
    let ctx = unsafe { &mut *ctx };

    if trigger_rect.is_null() || text.is_null() || viewport_rect.is_null() {
        return AkarTooltipResponse { visible: false };
    }

    let trigger_rect = unsafe { *(trigger_rect as *const [f32; 4]) };
    let viewport_rect = unsafe { *(viewport_rect as *const [f32; 4]) };

    let Ok(text_str) = unsafe { std::ffi::CStr::from_ptr(text) }.to_str() else {
        return AkarTooltipResponse { visible: false };
    };

    let side = match preferred_side {
        0 => akar_components::TooltipSide::Top,
        1 => akar_components::TooltipSide::Bottom,
        2 => akar_components::TooltipSide::Left,
        3 => akar_components::TooltipSide::Right,
        _ => akar_components::TooltipSide::Top,
    };

    let result = akar_components::akar_tooltip(
        &mut ctx.core,
        trigger_rect,
        text_str,
        side,
        &ctx.theme,
        viewport_rect,
    );

    AkarTooltipResponse {
        visible: result.visible,
    }
}

// ---- Modal ----

#[repr(C)]
pub struct AkarModalResponse {
    pub close_requested: bool,
    pub content_node: u64,
}

#[no_mangle]
pub unsafe extern "C" fn akar_modal_begin(
    ctx: *mut AkarCtx,
    title: *const c_char,
    title_len: i32,
    width: f32,
    height: f32,
    viewport_rect: *const f32,
) -> AkarModalResponse {
    let ctx = unsafe { &mut *ctx };

    if title.is_null() || title_len <= 0 || viewport_rect.is_null() {
        return AkarModalResponse {
            close_requested: false,
            content_node: 0,
        };
    }

    let viewport_rect = unsafe { *(viewport_rect as *const [f32; 4]) };
    let bytes = unsafe { std::slice::from_raw_parts(title as *const u8, title_len as usize) };
    let Ok(title_str) = std::str::from_utf8(bytes) else {
        return AkarModalResponse {
            close_requested: false,
            content_node: 0,
        };
    };

    let result = akar_components::modal_begin(
        &mut ctx.core,
        &mut ctx.layout,
        viewport_rect,
        title_str,
        width,
        height,
        &ctx.theme,
    );

    AkarModalResponse {
        close_requested: result.close_requested,
        content_node: result.content_node.into(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn akar_modal_end(ctx: *mut AkarCtx) {
    let ctx = unsafe { &mut *ctx };
    akar_components::modal_end(&mut ctx.core);
}

// ---- Toast ----

#[repr(C)]
pub struct AkarToastItem {
    pub variant: u32,
    pub message: *const c_char,
    pub dismiss_on_click: bool,
}

#[repr(C)]
pub struct AkarToastResponse {
    pub dismissed: i32,
}

#[no_mangle]
pub unsafe extern "C" fn akar_toasts(
    ctx: *mut AkarCtx,
    items: *const AkarToastItem,
    item_count: u32,
    viewport_rect: *const f32,
) -> AkarToastResponse {
    let ctx = unsafe { &mut *ctx };

    if items.is_null() || item_count == 0 || viewport_rect.is_null() {
        return AkarToastResponse { dismissed: -1 };
    }

    let viewport_rect = unsafe { *(viewport_rect as *const [f32; 4]) };

    let mut toast_items: Vec<akar_components::ToastItem> = Vec::with_capacity(item_count as usize);
    for i in 0..item_count as usize {
        let item = unsafe { &*items.add(i) };
        let variant = match item.variant {
            0 => akar_components::ToastVariant::Info,
            1 => akar_components::ToastVariant::Success,
            2 => akar_components::ToastVariant::Warning,
            3 => akar_components::ToastVariant::Error,
            _ => akar_components::ToastVariant::Info,
        };
        let message = if item.message.is_null() {
            String::new()
        } else {
            unsafe { std::ffi::CStr::from_ptr(item.message) }
                .to_string_lossy()
                .into_owned()
        };
        toast_items.push(akar_components::ToastItem {
            variant,
            message,
            dismiss_on_click: item.dismiss_on_click,
        });
    }

    let result = akar_components::toasts(&mut ctx.core, viewport_rect, &mut toast_items, &ctx.theme);

    AkarToastResponse {
        dismissed: result.dismissed.map(|i| i as i32).unwrap_or(-1),
    }
}

// ---- Dropdown ----

#[repr(C)]
pub struct AkarDropdownState {
    pub is_open: bool,
    pub content_rect: [f32; 4],
}

#[no_mangle]
pub unsafe extern "C" fn akar_dropdown_begin(
    ctx: *mut AkarCtx,
    anchor_rect: *const f32,
    item_height: f32,
    viewport_rect: *const f32,
    is_open: bool,
) -> AkarDropdownState {
    let ctx = unsafe { &mut *ctx };

    if anchor_rect.is_null() || viewport_rect.is_null() {
        return AkarDropdownState {
            is_open: false,
            content_rect: [0.0; 4],
        };
    }

    let anchor_rect = unsafe { *(anchor_rect as *const [f32; 4]) };
    let viewport_rect = unsafe { *(viewport_rect as *const [f32; 4]) };

    let result = akar_components::dropdown_begin(
        &mut ctx.core,
        anchor_rect,
        item_height,
        viewport_rect,
        is_open,
        &ctx.theme,
    );

    AkarDropdownState {
        is_open: result.is_open,
        content_rect: result.content_rect,
    }
}

#[no_mangle]
pub unsafe extern "C" fn akar_dropdown_end(ctx: *mut AkarCtx) {
    let ctx = unsafe { &mut *ctx };
    akar_components::dropdown_end(&mut ctx.core);
}

pub const AKAR_KEY_BACKSPACE: u32 = 0;
pub const AKAR_KEY_DELETE: u32 = 1;
pub const AKAR_KEY_LEFT: u32 = 2;
pub const AKAR_KEY_RIGHT: u32 = 3;
pub const AKAR_KEY_UP: u32 = 4;
pub const AKAR_KEY_DOWN: u32 = 5;
pub const AKAR_KEY_HOME: u32 = 6;
pub const AKAR_KEY_END: u32 = 7;
pub const AKAR_KEY_ENTER: u32 = 8;
pub const AKAR_KEY_ESCAPE: u32 = 9;
pub const AKAR_KEY_TAB: u32 = 10;

#[no_mangle]
pub unsafe extern "C" fn akar_push_key(ctx: *mut AkarCtx, key: u32) {
    let ctx = unsafe { &mut *ctx };
    let k = match key {
        0 => Key::Backspace,
        1 => Key::Delete,
        2 => Key::Left,
        3 => Key::Right,
        4 => Key::Up,
        5 => Key::Down,
        6 => Key::Home,
        7 => Key::End,
        8 => Key::Enter,
        9 => Key::Escape,
        10 => Key::Tab,
        _ => return,
    };
    ctx.core.input.push_key(k);
}

#[no_mangle]
pub unsafe extern "C" fn akar_checkbox(
    ctx: *mut AkarCtx,
    node_id: u64,
    label: *const c_char,
    label_len: i32,
    checked: *mut bool,
) -> bool {
    let ctx = unsafe { &mut *ctx };
    if label.is_null() || label_len <= 0 || checked.is_null() {
        return false;
    }
    let label_bytes = unsafe { std::slice::from_raw_parts(label as *const u8, label_len as usize) };
    let Ok(label_str) = std::str::from_utf8(label_bytes) else {
        return false;
    };
    let nid: akar_layout::NodeId = node_id.into();
    akar_components::akar_checkbox(&mut ctx.core, &ctx.layout, nid, unsafe { &mut *checked }, label_str, &ctx.theme)
}

#[no_mangle]
pub unsafe extern "C" fn akar_radio_group(
    ctx: *mut AkarCtx,
    nodes: *const u64,
    node_count: u32,
    labels: *const *const c_char,
    label_lengths: *const i32,
    selected: *mut u32,
) -> bool {
    let ctx = unsafe { &mut *ctx };
    if nodes.is_null() || node_count == 0 || labels.is_null() || label_lengths.is_null() || selected.is_null() {
        return false;
    }

    let mut node_ids = Vec::with_capacity(node_count as usize);
    let mut label_strs: Vec<&str> = Vec::with_capacity(node_count as usize);
    for i in 0..node_count as usize {
        let nid: akar_layout::NodeId = unsafe { *nodes.add(i) }.into();
        node_ids.push(nid);
        let ptr = unsafe { *labels.add(i) };
        let len = unsafe { *label_lengths.add(i) };
        if ptr.is_null() || len <= 0 { return false; }
        let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };
        match std::str::from_utf8(bytes) { Ok(s) => label_strs.push(s), Err(_) => return false }
    }

    let mut sel = unsafe { *selected } as usize;
    let changed = akar_components::akar_radio_group(&mut ctx.core, &ctx.layout, &node_ids, &label_strs, &mut sel, &ctx.theme);
    unsafe { *selected = sel as u32 };
    changed
}

#[no_mangle]
pub unsafe extern "C" fn akar_switch(
    ctx: *mut AkarCtx,
    node_id: u64,
    on: *mut bool,
) -> bool {
    let ctx = unsafe { &mut *ctx };
    if on.is_null() { return false; }
    let nid: akar_layout::NodeId = node_id.into();
    akar_components::akar_switch(&mut ctx.core, &ctx.layout, nid, unsafe { &mut *on }, &ctx.theme)
}

#[no_mangle]
pub unsafe extern "C" fn akar_slider(
    ctx: *mut AkarCtx,
    node_id: u64,
    value: *mut f32,
    min: f32,
    max: f32,
) -> bool {
    let ctx = unsafe { &mut *ctx };
    if value.is_null() { return false; }
    let nid: akar_layout::NodeId = node_id.into();
    akar_components::akar_slider(&mut ctx.core, &ctx.layout, nid, unsafe { &mut *value }, min, max, &ctx.theme)
}

#[repr(C)]
pub struct AkarSelectResponse {
    pub changed: bool,
}

#[no_mangle]
pub unsafe extern "C" fn akar_select(
    ctx: *mut AkarCtx,
    node_id: u64,
    options: *const *const c_char,
    option_count: u32,
    option_lengths: *const i32,
    selected: *mut u32,
    open: *mut bool,
    viewport_rect: *const f32,
) -> AkarSelectResponse {
    let ctx = unsafe { &mut *ctx };
    if options.is_null() || option_count == 0 || option_lengths.is_null() || selected.is_null() || open.is_null() || viewport_rect.is_null() {
        return AkarSelectResponse { changed: false };
    }

    let mut option_strs: Vec<&str> = Vec::with_capacity(option_count as usize);
    for i in 0..option_count as usize {
        let ptr = unsafe { *options.add(i) };
        let len = unsafe { *option_lengths.add(i) };
        if ptr.is_null() || len <= 0 { return AkarSelectResponse { changed: false }; }
        let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };
        match std::str::from_utf8(bytes) { Ok(s) => option_strs.push(s), Err(_) => return AkarSelectResponse { changed: false } }
    }

    let viewport = unsafe { *(viewport_rect as *const [f32; 4]) };
    let nid: akar_layout::NodeId = node_id.into();
    let mut sel = unsafe { *selected } as usize;
    let mut is_open = unsafe { *open };
    let changed = akar_components::akar_select(&mut ctx.core, &ctx.layout, nid, &option_strs, &mut sel, &mut is_open, &ctx.theme, viewport);
    unsafe { *selected = sel as u32 };
    unsafe { *open = is_open };
    AkarSelectResponse { changed }
}

#[repr(C)]
pub struct AkarTextInputResponse {
    pub changed: bool,
    pub submitted: bool,
    pub new_cursor_pos: u32,
}

#[no_mangle]
pub unsafe extern "C" fn akar_text_input(
    ctx: *mut AkarCtx,
    node_id: u64,
    value_buf: *mut u8,
    buf_len: u32,
    cursor_pos: *mut u32,
    placeholder: *const c_char,
    cursor_visible: bool,
) -> AkarTextInputResponse {
    let ctx = unsafe { &mut *ctx };
    if value_buf.is_null() || buf_len == 0 || cursor_pos.is_null() || placeholder.is_null() {
        return AkarTextInputResponse { changed: false, submitted: false, new_cursor_pos: 0 };
    }

    let Ok(placeholder_str) = unsafe { std::ffi::CStr::from_ptr(placeholder) }.to_str() else {
        return AkarTextInputResponse { changed: false, submitted: false, new_cursor_pos: 0 };
    };

    let slice = unsafe { std::slice::from_raw_parts_mut(value_buf, buf_len as usize) };
    let Ok(mut value) = String::from_utf8(slice.to_vec()) else {
        return AkarTextInputResponse { changed: false, submitted: false, new_cursor_pos: 0 };
    };

    let mut cp = unsafe { *cursor_pos } as usize;
    let nid: akar_layout::NodeId = node_id.into();
    let result = akar_components::akar_text_input(&mut ctx.core, &ctx.layout, nid, &mut value, &mut cp, placeholder_str, cursor_visible, &ctx.theme);

    let value_bytes = value.as_bytes();
    let copy_len = value_bytes.len().min(buf_len as usize);
    unsafe { std::ptr::copy_nonoverlapping(value_bytes.as_ptr(), slice.as_mut_ptr(), copy_len) };

    if copy_len < buf_len as usize {
        unsafe { *slice.as_mut_ptr().add(copy_len) = 0 };
    }

    unsafe { *cursor_pos = cp as u32 };
    AkarTextInputResponse {
        changed: result.changed,
        submitted: result.submitted,
        new_cursor_pos: cp as u32,
    }
}

#[repr(C)]
pub struct AkarTextAreaResponse {
    pub changed: bool,
    pub new_cursor_pos: u32,
}

#[no_mangle]
pub unsafe extern "C" fn akar_textarea(
    ctx: *mut AkarCtx,
    node_id: u64,
    value_buf: *mut u8,
    buf_len: u32,
    cursor_pos: *mut u32,
    scroll_y: *mut f32,
    placeholder: *const c_char,
    cursor_visible: bool,
) -> AkarTextAreaResponse {
    let ctx = unsafe { &mut *ctx };
    if value_buf.is_null() || buf_len == 0 || cursor_pos.is_null() || scroll_y.is_null() || placeholder.is_null() {
        return AkarTextAreaResponse { changed: false, new_cursor_pos: 0 };
    }

    let Ok(placeholder_str) = unsafe { std::ffi::CStr::from_ptr(placeholder) }.to_str() else {
        return AkarTextAreaResponse { changed: false, new_cursor_pos: 0 };
    };

    let slice = unsafe { std::slice::from_raw_parts_mut(value_buf, buf_len as usize) };
    let Ok(mut value) = String::from_utf8(slice.to_vec()) else {
        return AkarTextAreaResponse { changed: false, new_cursor_pos: 0 };
    };

    let mut cp = unsafe { *cursor_pos } as usize;
    let nid: akar_layout::NodeId = node_id.into();
    let result = akar_components::akar_textarea(&mut ctx.core, &ctx.layout, nid, &mut value, &mut cp, unsafe { &mut *scroll_y }, placeholder_str, cursor_visible, &ctx.theme);

    let value_bytes = value.as_bytes();
    let copy_len = value_bytes.len().min(buf_len as usize);
    unsafe { std::ptr::copy_nonoverlapping(value_bytes.as_ptr(), slice.as_mut_ptr(), copy_len) };
    if copy_len < buf_len as usize {
        unsafe { *slice.as_mut_ptr().add(copy_len) = 0 };
    }

    unsafe { *cursor_pos = cp as u32 };
    AkarTextAreaResponse {
        changed: result.changed,
        new_cursor_pos: cp as u32,
    }
}
