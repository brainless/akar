#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_void};
use std::ptr;

use akar_components::{AkarTheme, ButtonVariant, AKAR_THEME_DARK};
use akar_core::{
    AkarCore, Key, KeyEvent, Modifiers, Shortcut, ShortcutModifiers, TextEditKeybindings,
};
use akar_layout::Layout;

const SENTINEL_F32: f32 = 0.0;
const SENTINEL_U8: u8 = 0xFF;
const SENTINEL_U32: u32 = 0xFF;

pub struct AkarCtx {
    core: AkarCore,
    layout: Layout,
    theme: AkarTheme,
    device: *const wgpu::Device,
    queue: *const wgpu::Queue,
}

unsafe impl Send for AkarCtx {}
unsafe impl Sync for AkarCtx {}

// ---- Typography C types ----

#[repr(C)]
pub struct AkarFontFamily {
    pub value: u32,
}

#[repr(C)]
pub struct AkarFontWeight {
    pub value: u32,
}

#[repr(C)]
pub struct AkarTextAlign {
    pub value: u32,
}

/// Unset value for `AkarTextStyle::font_family_name_handle`. Distinct from
/// `SENTINEL_U32` (0xFF), which would collide with a real font handle.
pub const AKAR_FONT_FAMILY_NAME_HANDLE_NONE: u32 = u32::MAX;

#[repr(C)]
pub struct AkarTextStyle {
    pub font_size: f32,
    pub line_height: f32,
    pub color: u32,
    pub font_weight: u32,
    pub font_family: u32,
    /// Handle returned by `akar_load_font_bytes`, selecting a runtime-loaded
    /// family and overriding `font_family`. Set to
    /// `AKAR_FONT_FAMILY_NAME_HANDLE_NONE` when unused.
    pub font_family_name_handle: u32,
    pub align: u32,
    pub wrap: u8,
}

#[repr(C)]
pub struct AkarHeadingLevel {
    pub value: u32,
}

// ---- Component style C types ----

#[repr(C)]
pub struct AkarCardLayout {
    pub direction: u32,
    pub gap: f32,
    pub padding: f32,
    pub has_header: u8,
    pub has_footer: u8,
}

#[repr(C)]
pub struct AkarCardStyle {
    pub background: u32,
    pub border_color: u32,
    pub border_width: f32,
    pub corner_radii: [f32; 4],
    pub shadow_blur: f32,
    pub shadow_spread: f32,
    pub shadow_color: u32,
    pub shadow_offset: [f32; 2],
    pub separator_color: u32,
}

#[repr(C)]
pub struct AkarCardSlots {
    pub header: u64,
    pub body: u64,
    pub footer: u64,
}

#[repr(C)]
pub struct AkarLinkResult {
    pub clicked: bool,
    pub hovered: bool,
    pub pressed: bool,
}

#[repr(C)]
pub struct AkarButtonStyle {
    pub fill: u32,
    pub hover_fill: u32,
    pub pressed_fill: u32,
    pub border_color: u32,
    pub content_color: u32,
    pub text_style: AkarTextStyle,
}

#[repr(C)]
pub struct AkarBadgeStyle {
    pub fill: u32,
    pub border_color: u32,
    pub content_color: u32,
    pub text_style: AkarTextStyle,
}

#[repr(C)]
pub struct AkarSeparatorStyle {
    pub color: u32,
    pub thickness: f32,
}

#[repr(C)]
pub struct AkarStatStyle {
    pub title_color: u32,
    pub value_color: u32,
    pub description_color: u32,
    pub title_text_style: AkarTextStyle,
    pub value_text_style: AkarTextStyle,
    pub description_text_style: AkarTextStyle,
}

#[repr(C)]
pub struct AkarNavbarStyle {
    pub background: u32,
    pub border_color: u32,
    pub border_width: f32,
    pub corner_radii: [f32; 4],
}

#[repr(C)]
pub struct AkarTabBarStyle {
    pub active_color: u32,
    pub inactive_color: u32,
    pub indicator_color: u32,
}

// ---- Existing result types ----

#[repr(C)]
pub struct AkarButtonResult {
    pub clicked: bool,
    pub hovered: bool,
    pub pressed: bool,
}

pub const AKAR_SHORTCUT_MODIFIER_PRIMARY: u32 = 1 << 0;
pub const AKAR_SHORTCUT_MODIFIER_CONTROL: u32 = 1 << 1;
pub const AKAR_SHORTCUT_MODIFIER_SUPER: u32 = 1 << 2;
pub const AKAR_SHORTCUT_MODIFIER_ALT: u32 = 1 << 3;
pub const AKAR_SHORTCUT_MODIFIER_SHIFT: u32 = 1 << 4;

pub const AKAR_KEY_CHARACTER: u32 = 11;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct AkarShortcut {
    pub modifiers: u32,
    pub key: u32,
    pub character: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct AkarTextEditKeybindings {
    pub select_all: AkarShortcut,
    pub copy: AkarShortcut,
    pub paste: AkarShortcut,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct AkarTextEditState {
    pub cursor: u32,
    pub anchor: u32,
}

fn c_text_style_to_rust(ptr: *const AkarTextStyle) -> Option<akar_components::TextStyle> {
    if ptr.is_null() {
        return None;
    }
    let s = unsafe { &*ptr };
    let mut style = akar_components::TextStyle::empty();
    let mut any = false;
    if s.font_size > SENTINEL_F32 {
        style.font_size = Some(s.font_size);
        any = true;
    }
    if s.line_height > SENTINEL_F32 {
        style.line_height = Some(s.line_height);
        any = true;
    }
    if s.color != 0 {
        style.color = Some(s.color);
        any = true;
    }
    if s.font_weight != SENTINEL_U32 {
        style.font_weight = Some(match s.font_weight {
            0 => akar_components::FontWeight::Normal,
            1 => akar_components::FontWeight::Medium,
            2 => akar_components::FontWeight::Semibold,
            3 => akar_components::FontWeight::Bold,
            _ => return Some(style),
        });
        any = true;
    }
    if s.font_family != SENTINEL_U32 {
        style.font_family = Some(match s.font_family {
            0 => akar_components::FontFamily::SansSerif,
            1 => akar_components::FontFamily::Serif,
            2 => akar_components::FontFamily::Monospace,
            _ => return Some(style),
        });
        any = true;
    }
    if s.font_family_name_handle != AKAR_FONT_FAMILY_NAME_HANDLE_NONE {
        style.font_family = Some(akar_components::FontFamily::Named(
            s.font_family_name_handle,
        ));
        any = true;
    }
    if s.align != SENTINEL_U32 {
        style.align = Some(match s.align {
            0 => akar_components::TextAlign::Start,
            1 => akar_components::TextAlign::Center,
            2 => akar_components::TextAlign::End,
            _ => return Some(style),
        });
        any = true;
    }
    if s.wrap != SENTINEL_U8 {
        style.wrap = Some(s.wrap != 0);
        any = true;
    }
    if any {
        Some(style)
    } else {
        None
    }
}

fn c_heading_level_to_rust(value: u32) -> akar_components::HeadingLevel {
    match value {
        0 => akar_components::HeadingLevel::H1,
        1 => akar_components::HeadingLevel::H2,
        2 => akar_components::HeadingLevel::H3,
        3 => akar_components::HeadingLevel::H4,
        _ => akar_components::HeadingLevel::H1,
    }
}

fn c_button_variant_to_rust(value: u32) -> akar_components::ButtonVariant {
    match value {
        0 => akar_components::ButtonVariant::Solid,
        1 => akar_components::ButtonVariant::Outline,
        2 => akar_components::ButtonVariant::Ghost,
        _ => akar_components::ButtonVariant::Solid,
    }
}

fn c_badge_variant_to_rust(value: u32) -> akar_components::BadgeVariant {
    match value {
        0 => akar_components::BadgeVariant::Default,
        1 => akar_components::BadgeVariant::Primary,
        2 => akar_components::BadgeVariant::Success,
        3 => akar_components::BadgeVariant::Warning,
        4 => akar_components::BadgeVariant::Error,
        5 => akar_components::BadgeVariant::Info,
        _ => akar_components::BadgeVariant::Default,
    }
}

fn c_tab_variant_to_rust(value: u32) -> akar_components::TabVariant {
    match value {
        1 => akar_components::TabVariant::Lifted,
        2 => akar_components::TabVariant::Pills,
        3 => akar_components::TabVariant::Underline,
        _ => akar_components::TabVariant::Boxed,
    }
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

    let core = AkarCore::new(
        device_ref,
        queue_ref,
        format,
        akar_core::TextPipelineConfig::default(),
    );
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

pub const AKAR_FONT_LOAD_OK: u32 = 0;
/// Null context, null byte pointer, or zero length.
pub const AKAR_FONT_LOAD_INVALID_ARGUMENT: u32 = 1;
/// The bytes contain no parsable font face.
pub const AKAR_FONT_LOAD_INVALID_DATA: u32 = 2;
/// The bytes parsed but carry no font family.
pub const AKAR_FONT_LOAD_EMPTY_SOURCE: u32 = 3;
/// A collection spanning more than one family; v1 accepts exactly one.
pub const AKAR_FONT_LOAD_MULTIPLE_FAMILIES: u32 = 4;

/// Loads font bytes (TTF/OTF/TTC/OTC) into the context's font database.
///
/// Returns `AKAR_FONT_LOAD_OK` and writes the family handle to `out_handle`
/// (when non-NULL) on success, or one of the `AKAR_FONT_LOAD_*` error codes.
/// `out_handle` is left untouched on failure. Loading the same family twice
/// returns the same handle. Safe to call any time after context creation.
#[no_mangle]
pub unsafe extern "C" fn akar_load_font_bytes(
    ctx: *mut AkarCtx,
    bytes: *const u8,
    len: u32,
    out_handle: *mut u32,
) -> u32 {
    let Some(ctx) = (unsafe { ctx.as_mut() }) else {
        return AKAR_FONT_LOAD_INVALID_ARGUMENT;
    };
    if bytes.is_null() || len == 0 {
        return AKAR_FONT_LOAD_INVALID_ARGUMENT;
    }

    let data = unsafe { std::slice::from_raw_parts(bytes, len as usize) }.to_vec();
    match ctx.core.text_pipeline.load_font_bytes(data) {
        Ok(handle) => {
            if let Some(out) = unsafe { out_handle.as_mut() } {
                *out = handle;
            }
            AKAR_FONT_LOAD_OK
        }
        Err(akar_core::FontLoadError::InvalidFontData(_)) => AKAR_FONT_LOAD_INVALID_DATA,
        Err(akar_core::FontLoadError::EmptyFontSource) => AKAR_FONT_LOAD_EMPTY_SOURCE,
        Err(akar_core::FontLoadError::MultipleFamilies(_)) => AKAR_FONT_LOAD_MULTIPLE_FAMILIES,
    }
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
    let result =
        akar_components::drawer_begin(&mut ctx.core, rect, drawer_edge, panel_width, &ctx.theme);
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
    let slots =
        akar_components::akar_navbar_combined(&mut ctx.core, &mut ctx.layout, nid, &ctx.theme);
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

    let result =
        akar_components::toasts(&mut ctx.core, viewport_rect, &mut toast_items, &ctx.theme);

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

fn key_from_raw(key: u32, character: u32) -> Option<Key> {
    match key {
        AKAR_KEY_BACKSPACE => Some(Key::Backspace),
        AKAR_KEY_DELETE => Some(Key::Delete),
        AKAR_KEY_LEFT => Some(Key::Left),
        AKAR_KEY_RIGHT => Some(Key::Right),
        AKAR_KEY_UP => Some(Key::Up),
        AKAR_KEY_DOWN => Some(Key::Down),
        AKAR_KEY_HOME => Some(Key::Home),
        AKAR_KEY_END => Some(Key::End),
        AKAR_KEY_ENTER => Some(Key::Enter),
        AKAR_KEY_ESCAPE => Some(Key::Escape),
        AKAR_KEY_TAB => Some(Key::Tab),
        AKAR_KEY_CHARACTER => char::from_u32(character).map(Key::Character),
        _ => None,
    }
}

fn shortcut_modifiers_from_raw(raw: u32) -> ShortcutModifiers {
    let mut modifiers = ShortcutModifiers::NONE;
    if raw & AKAR_SHORTCUT_MODIFIER_PRIMARY != 0 {
        modifiers |= ShortcutModifiers::PRIMARY;
    }
    if raw & AKAR_SHORTCUT_MODIFIER_CONTROL != 0 {
        modifiers |= ShortcutModifiers::CONTROL;
    }
    if raw & AKAR_SHORTCUT_MODIFIER_SUPER != 0 {
        modifiers |= ShortcutModifiers::SUPER;
    }
    if raw & AKAR_SHORTCUT_MODIFIER_ALT != 0 {
        modifiers |= ShortcutModifiers::ALT;
    }
    if raw & AKAR_SHORTCUT_MODIFIER_SHIFT != 0 {
        modifiers |= ShortcutModifiers::SHIFT;
    }
    modifiers
}

fn modifiers_from_raw(raw: u32) -> Modifiers {
    let mut modifiers = Modifiers {
        shift: raw & AKAR_SHORTCUT_MODIFIER_SHIFT != 0,
        control: raw & AKAR_SHORTCUT_MODIFIER_CONTROL != 0,
        alt: raw & AKAR_SHORTCUT_MODIFIER_ALT != 0,
        super_key: raw & AKAR_SHORTCUT_MODIFIER_SUPER != 0,
    };
    if raw & AKAR_SHORTCUT_MODIFIER_PRIMARY != 0 {
        #[cfg(target_os = "macos")]
        {
            modifiers.super_key = true;
        }
        #[cfg(not(target_os = "macos"))]
        {
            modifiers.control = true;
        }
    }
    modifiers
}

fn shortcut_from_ffi(shortcut: AkarShortcut) -> Option<Shortcut> {
    Some(Shortcut::new(
        shortcut_modifiers_from_raw(shortcut.modifiers),
        key_from_raw(shortcut.key, shortcut.character)?,
    ))
}

#[no_mangle]
pub extern "C" fn akar_text_edit_keybindings_default() -> AkarTextEditKeybindings {
    AkarTextEditKeybindings {
        select_all: AkarShortcut {
            modifiers: AKAR_SHORTCUT_MODIFIER_PRIMARY,
            key: AKAR_KEY_CHARACTER,
            character: 'a' as u32,
        },
        copy: AkarShortcut {
            modifiers: AKAR_SHORTCUT_MODIFIER_PRIMARY,
            key: AKAR_KEY_CHARACTER,
            character: 'c' as u32,
        },
        paste: AkarShortcut {
            modifiers: AKAR_SHORTCUT_MODIFIER_PRIMARY,
            key: AKAR_KEY_CHARACTER,
            character: 'v' as u32,
        },
    }
}

#[no_mangle]
pub unsafe extern "C" fn akar_set_text_edit_keybindings(
    ctx: *mut AkarCtx,
    bindings: AkarTextEditKeybindings,
) -> bool {
    let Some(select_all) = shortcut_from_ffi(bindings.select_all) else {
        return false;
    };
    let Some(copy) = shortcut_from_ffi(bindings.copy) else {
        return false;
    };
    let Some(paste) = shortcut_from_ffi(bindings.paste) else {
        return false;
    };
    let Some(ctx) = (unsafe { ctx.as_mut() }) else {
        return false;
    };
    ctx.core.set_text_edit_keybindings(TextEditKeybindings {
        select_all,
        copy,
        paste,
    });
    true
}

#[no_mangle]
pub unsafe extern "C" fn akar_push_key(ctx: *mut AkarCtx, key: u32) {
    let ctx = unsafe { &mut *ctx };
    if let Some(key) = key_from_raw(key, 0) {
        ctx.core.input.push_key(key);
    }
}

#[no_mangle]
pub unsafe extern "C" fn akar_push_key_event(
    ctx: *mut AkarCtx,
    key: u32,
    character: u32,
    modifiers: u32,
    repeat: bool,
) {
    let Some(ctx) = (unsafe { ctx.as_mut() }) else {
        return;
    };
    let Some(key) = key_from_raw(key, character) else {
        return;
    };
    ctx.core.input.push_key_event(KeyEvent {
        key,
        modifiers: modifiers_from_raw(modifiers),
        repeat,
    });
}

#[no_mangle]
pub unsafe extern "C" fn akar_push_paste(
    ctx: *mut AkarCtx,
    widget_id: u64,
    utf8: *const u8,
    utf8_len: u32,
) -> bool {
    let Some(ctx) = (unsafe { ctx.as_mut() }) else {
        return false;
    };
    if utf8.is_null() && utf8_len != 0 {
        return false;
    }
    let bytes = if utf8_len == 0 {
        &[][..]
    } else {
        unsafe { std::slice::from_raw_parts(utf8, utf8_len as usize) }
    };
    let Ok(text) = std::str::from_utf8(bytes) else {
        return false;
    };
    ctx.core.input.push_paste(widget_id, text);
    true
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
    akar_components::akar_checkbox(
        &mut ctx.core,
        &ctx.layout,
        nid,
        unsafe { &mut *checked },
        label_str,
        &ctx.theme,
    )
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
    if nodes.is_null()
        || node_count == 0
        || labels.is_null()
        || label_lengths.is_null()
        || selected.is_null()
    {
        return false;
    }

    let mut node_ids = Vec::with_capacity(node_count as usize);
    let mut label_strs: Vec<&str> = Vec::with_capacity(node_count as usize);
    for i in 0..node_count as usize {
        let nid: akar_layout::NodeId = unsafe { *nodes.add(i) }.into();
        node_ids.push(nid);
        let ptr = unsafe { *labels.add(i) };
        let len = unsafe { *label_lengths.add(i) };
        if ptr.is_null() || len <= 0 {
            return false;
        }
        let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };
        match std::str::from_utf8(bytes) {
            Ok(s) => label_strs.push(s),
            Err(_) => return false,
        }
    }

    let mut sel = unsafe { *selected } as usize;
    let changed = akar_components::akar_radio_group(
        &mut ctx.core,
        &ctx.layout,
        &node_ids,
        &label_strs,
        &mut sel,
        &ctx.theme,
    );
    unsafe { *selected = sel as u32 };
    changed
}

#[no_mangle]
pub unsafe extern "C" fn akar_switch(ctx: *mut AkarCtx, node_id: u64, on: *mut bool) -> bool {
    let ctx = unsafe { &mut *ctx };
    if on.is_null() {
        return false;
    }
    let nid: akar_layout::NodeId = node_id.into();
    akar_components::akar_switch(
        &mut ctx.core,
        &ctx.layout,
        nid,
        unsafe { &mut *on },
        &ctx.theme,
    )
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
    if value.is_null() {
        return false;
    }
    let nid: akar_layout::NodeId = node_id.into();
    akar_components::akar_slider(
        &mut ctx.core,
        &ctx.layout,
        nid,
        unsafe { &mut *value },
        min,
        max,
        &ctx.theme,
    )
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
    if options.is_null()
        || option_count == 0
        || option_lengths.is_null()
        || selected.is_null()
        || open.is_null()
        || viewport_rect.is_null()
    {
        return AkarSelectResponse { changed: false };
    }

    let mut option_strs: Vec<&str> = Vec::with_capacity(option_count as usize);
    for i in 0..option_count as usize {
        let ptr = unsafe { *options.add(i) };
        let len = unsafe { *option_lengths.add(i) };
        if ptr.is_null() || len <= 0 {
            return AkarSelectResponse { changed: false };
        }
        let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };
        match std::str::from_utf8(bytes) {
            Ok(s) => option_strs.push(s),
            Err(_) => return AkarSelectResponse { changed: false },
        }
    }

    let viewport = unsafe { *(viewport_rect as *const [f32; 4]) };
    let nid: akar_layout::NodeId = node_id.into();
    let mut sel = unsafe { *selected } as usize;
    let mut is_open = unsafe { *open };
    let changed = akar_components::akar_select(
        &mut ctx.core,
        &ctx.layout,
        nid,
        &option_strs,
        &mut sel,
        &mut is_open,
        &ctx.theme,
        viewport,
    );
    unsafe { *selected = sel as u32 };
    unsafe { *open = is_open };
    AkarSelectResponse { changed }
}

#[repr(C)]
pub struct AkarTextInputResponse {
    pub changed: bool,
    pub submitted: bool,
    pub widget_id: u64,
    pub edit_state: AkarTextEditState,
    pub copy_len: u32,
    pub copy_required_len: u32,
    pub request_paste: bool,
}

fn empty_text_input_response() -> AkarTextInputResponse {
    AkarTextInputResponse {
        changed: false,
        submitted: false,
        widget_id: 0,
        edit_state: AkarTextEditState::default(),
        copy_len: 0,
        copy_required_len: 0,
        request_paste: false,
    }
}

fn empty_textarea_response() -> AkarTextAreaResponse {
    AkarTextAreaResponse {
        changed: false,
        widget_id: 0,
        edit_state: AkarTextEditState::default(),
        copy_len: 0,
        copy_required_len: 0,
        request_paste: false,
    }
}

unsafe fn value_from_ffi(
    value_buf: *mut u8,
    value_len: *mut u32,
    value_capacity: u32,
) -> Option<String> {
    if value_buf.is_null() || value_len.is_null() {
        return None;
    }
    let len = unsafe { *value_len } as usize;
    if len > value_capacity as usize {
        return None;
    }
    let bytes = unsafe { std::slice::from_raw_parts(value_buf, len) };
    std::str::from_utf8(bytes).ok().map(str::to_owned)
}

fn utf8_prefix_len(value: &str, capacity: usize) -> usize {
    let mut len = value.len().min(capacity);
    while !value.is_char_boundary(len) {
        len -= 1;
    }
    len
}

unsafe fn write_value_to_ffi(
    value_buf: *mut u8,
    value_len: *mut u32,
    value_capacity: u32,
    value: &str,
) -> usize {
    let capacity = value_capacity as usize;
    let len = utf8_prefix_len(value, capacity);
    if len != 0 {
        unsafe { std::ptr::copy_nonoverlapping(value.as_ptr(), value_buf, len) };
    }
    if len < capacity {
        unsafe { *value_buf.add(len) = 0 };
    }
    unsafe { *value_len = len as u32 };
    len
}

unsafe fn write_copy_to_ffi(
    copy_text: Option<&str>,
    copy_buf: *mut u8,
    copy_capacity: u32,
) -> (u32, u32) {
    let Some(copy_text) = copy_text else {
        return (0, 0);
    };
    let required = copy_text.len();
    if copy_buf.is_null() || copy_capacity == 0 {
        return (0, required as u32);
    }
    let capacity = copy_capacity as usize;
    let len = utf8_prefix_len(copy_text, capacity);
    if len != 0 {
        unsafe { std::ptr::copy_nonoverlapping(copy_text.as_ptr(), copy_buf, len) };
    }
    if len < capacity {
        unsafe { *copy_buf.add(len) = 0 };
    }
    (len as u32, required as u32)
}

#[no_mangle]
/// Edits a caller-owned UTF-8 buffer.
///
/// `value_len` is the meaningful byte length on input and receives the new
/// meaningful byte length. `value_capacity` is the allocation size in bytes.
/// Output is truncated only at a UTF-8 boundary and is NUL-terminated when the
/// resulting length is smaller than the capacity. Copy text is written to
/// `copy_buf`; `copy_len` reports bytes written and `copy_required_len` reports
/// the complete selected byte length.
pub unsafe extern "C" fn akar_text_input(
    ctx: *mut AkarCtx,
    node_id: u64,
    value_buf: *mut u8,
    value_len: *mut u32,
    value_capacity: u32,
    edit_state: *mut AkarTextEditState,
    placeholder: *const c_char,
    cursor_visible: bool,
    copy_buf: *mut u8,
    copy_capacity: u32,
) -> AkarTextInputResponse {
    let Some(ctx) = (unsafe { ctx.as_mut() }) else {
        return empty_text_input_response();
    };
    if edit_state.is_null() || placeholder.is_null() {
        return empty_text_input_response();
    }

    let Ok(placeholder_str) = unsafe { std::ffi::CStr::from_ptr(placeholder) }.to_str() else {
        return empty_text_input_response();
    };

    let Some(mut value) = (unsafe { value_from_ffi(value_buf, value_len, value_capacity) }) else {
        return empty_text_input_response();
    };

    let ffi_state = unsafe { &mut *edit_state };
    let mut rust_state = akar_components::TextEditState {
        cursor: ffi_state.cursor as usize,
        anchor: ffi_state.anchor as usize,
    };
    let nid: akar_layout::NodeId = node_id.into();
    let result = akar_components::akar_text_input(
        &mut ctx.core,
        &ctx.layout,
        nid,
        &mut value,
        &mut rust_state,
        placeholder_str,
        cursor_visible,
        &ctx.theme,
    );

    let written_len = unsafe { write_value_to_ffi(value_buf, value_len, value_capacity, &value) };
    rust_state.normalize(&value[..written_len]);
    *ffi_state = AkarTextEditState {
        cursor: rust_state.cursor as u32,
        anchor: rust_state.anchor as u32,
    };
    let (copy_len, copy_required_len) =
        unsafe { write_copy_to_ffi(result.copy_text.as_deref(), copy_buf, copy_capacity) };
    AkarTextInputResponse {
        changed: result.changed,
        submitted: result.submitted,
        widget_id: ctx.layout.widget_id(nid),
        edit_state: *ffi_state,
        copy_len,
        copy_required_len,
        request_paste: result.request_paste,
    }
}

#[repr(C)]
pub struct AkarTextAreaResponse {
    pub changed: bool,
    pub widget_id: u64,
    pub edit_state: AkarTextEditState,
    pub copy_len: u32,
    pub copy_required_len: u32,
    pub request_paste: bool,
}

// ---- Data Item / Data List ----

#[repr(C)]
pub struct AkarDataItemStyle {
    pub surface: [f32; 4],
    pub padding_x: f32,
    pub padding_y: f32,
    pub spacing: f32,
    pub color_normal: [f32; 4],
    pub color_hover: [f32; 4],
    pub color_pressed: [f32; 4],
    pub color_selected: [f32; 4],
    pub corner_radius: f32,
    pub border_width: f32,
    pub border_color: [f32; 4],
}

#[repr(C)]
pub struct AkarDataItemResponse {
    pub hovered: bool,
    pub pressed: bool,
    pub clicked: bool,
}

#[repr(C)]
pub struct AkarDataListState {
    pub scroll_y: f32,
}

#[repr(C)]
pub struct AkarDataListResponse {
    pub viewport_rect: [f32; 4],
    pub content_origin: [f32; 2],
    pub visible_range_start: u32,
    pub visible_range_end: u32,
}

#[no_mangle]
pub unsafe extern "C" fn akar_data_item_style_default(
    ctx: *mut AkarCtx,
    style_out: *mut AkarDataItemStyle,
) {
    let ctx = unsafe { &mut *ctx };
    if style_out.is_null() {
        return;
    }
    let s = akar_components::DataItemStyle::from_theme(&ctx.theme);
    let out = unsafe { &mut *style_out };
    out.surface = s.surface;
    out.padding_x = s.padding_x;
    out.padding_y = s.padding_y;
    out.spacing = s.spacing;
    out.color_normal = s.color_normal;
    out.color_hover = s.color_hover;
    out.color_pressed = s.color_pressed;
    out.color_selected = s.color_selected;
    out.corner_radius = s.corner_radius;
    out.border_width = s.border_width;
    out.border_color = s.border_color;
}

#[no_mangle]
pub unsafe extern "C" fn akar_data_item(
    ctx: *mut AkarCtx,
    node_id: u64,
    key: u64,
    style: *const AkarDataItemStyle,
) -> AkarDataItemResponse {
    let ctx = unsafe { &mut *ctx };
    if style.is_null() {
        return AkarDataItemResponse {
            hovered: false,
            pressed: false,
            clicked: false,
        };
    }
    let s = unsafe { &*style };
    let rust_style = akar_components::DataItemStyle {
        surface: s.surface,
        padding_x: s.padding_x,
        padding_y: s.padding_y,
        spacing: s.spacing,
        color_normal: s.color_normal,
        color_hover: s.color_hover,
        color_pressed: s.color_pressed,
        color_selected: s.color_selected,
        corner_radius: s.corner_radius,
        border_width: s.border_width,
        border_color: s.border_color,
    };
    let nid: akar_layout::NodeId = node_id.into();
    let result = akar_components::akar_data_item(&mut ctx.core, &ctx.layout, nid, key, &rust_style);
    AkarDataItemResponse {
        hovered: result.hovered,
        pressed: result.pressed,
        clicked: result.clicked,
    }
}

#[no_mangle]
pub unsafe extern "C" fn akar_data_list_begin(
    ctx: *mut AkarCtx,
    node_id: u64,
    state: *mut AkarDataListState,
    item_count: u32,
    item_height: f32,
    keys: *const u64,
    key_count: u32,
) -> AkarDataListResponse {
    let ctx = unsafe { &mut *ctx };
    if state.is_null() {
        return AkarDataListResponse {
            viewport_rect: [0.0; 4],
            content_origin: [0.0; 2],
            visible_range_start: 0,
            visible_range_end: 0,
        };
    }
    let nid: akar_layout::NodeId = node_id.into();
    let rust_state = unsafe { &mut *state };
    let mut data_state = akar_components::DataListState {
        scroll_y: rust_state.scroll_y,
    };
    let keys_slice = if keys.is_null() || key_count == 0 {
        &[][..]
    } else {
        unsafe { std::slice::from_raw_parts(keys, key_count as usize) }
    };
    let result = akar_components::data_list_begin(
        &mut ctx.core,
        &ctx.layout,
        nid,
        &mut data_state,
        item_count as usize,
        item_height,
        keys_slice,
    );
    rust_state.scroll_y = data_state.scroll_y;
    AkarDataListResponse {
        viewport_rect: result.viewport_rect,
        content_origin: result.content_origin,
        visible_range_start: result.visible_range.start as u32,
        visible_range_end: result.visible_range.end as u32,
    }
}

#[no_mangle]
pub unsafe extern "C" fn akar_data_list_end(ctx: *mut AkarCtx) {
    let ctx = unsafe { &mut *ctx };
    akar_components::data_list_end(&mut ctx.core);
}

#[no_mangle]
/// Edits a caller-owned multiline UTF-8 buffer.
///
/// Buffer and copy-output semantics match `akar_text_input`.
pub unsafe extern "C" fn akar_textarea(
    ctx: *mut AkarCtx,
    node_id: u64,
    value_buf: *mut u8,
    value_len: *mut u32,
    value_capacity: u32,
    edit_state: *mut AkarTextEditState,
    scroll_y: *mut f32,
    placeholder: *const c_char,
    cursor_visible: bool,
    copy_buf: *mut u8,
    copy_capacity: u32,
) -> AkarTextAreaResponse {
    let Some(ctx) = (unsafe { ctx.as_mut() }) else {
        return empty_textarea_response();
    };
    if edit_state.is_null() || scroll_y.is_null() || placeholder.is_null() {
        return empty_textarea_response();
    }

    let Ok(placeholder_str) = unsafe { std::ffi::CStr::from_ptr(placeholder) }.to_str() else {
        return empty_textarea_response();
    };

    let Some(mut value) = (unsafe { value_from_ffi(value_buf, value_len, value_capacity) }) else {
        return empty_textarea_response();
    };

    let ffi_state = unsafe { &mut *edit_state };
    let mut rust_state = akar_components::TextEditState {
        cursor: ffi_state.cursor as usize,
        anchor: ffi_state.anchor as usize,
    };
    let nid: akar_layout::NodeId = node_id.into();
    let result = akar_components::akar_textarea(
        &mut ctx.core,
        &ctx.layout,
        nid,
        &mut value,
        &mut rust_state,
        unsafe { &mut *scroll_y },
        placeholder_str,
        cursor_visible,
        &ctx.theme,
    );

    let written_len = unsafe { write_value_to_ffi(value_buf, value_len, value_capacity, &value) };
    rust_state.normalize(&value[..written_len]);
    *ffi_state = AkarTextEditState {
        cursor: rust_state.cursor as u32,
        anchor: rust_state.anchor as u32,
    };
    let (copy_len, copy_required_len) =
        unsafe { write_copy_to_ffi(result.copy_text.as_deref(), copy_buf, copy_capacity) };
    AkarTextAreaResponse {
        changed: result.changed,
        widget_id: ctx.layout.widget_id(nid),
        edit_state: *ffi_state,
        copy_len,
        copy_required_len,
        request_paste: result.request_paste,
    }
}

// ---- New component C API (Tasks 3-7) ----

#[no_mangle]
pub unsafe extern "C" fn akar_heading(
    ctx: *mut AkarCtx,
    node_id: u64,
    text: *const c_char,
    level: u32,
    style: *const AkarTextStyle,
) {
    let Some(ctx) = (unsafe { ctx.as_mut() }) else {
        return;
    };
    if text.is_null() {
        return;
    }
    let Ok(text_str) = unsafe { std::ffi::CStr::from_ptr(text) }.to_str() else {
        return;
    };
    let nid: akar_layout::NodeId = node_id.into();
    let overrides = c_text_style_to_rust(style);
    akar_components::akar_heading(
        &mut ctx.core,
        &ctx.layout,
        nid,
        text_str,
        c_heading_level_to_rust(level),
        overrides,
        &ctx.theme,
    );
}

#[no_mangle]
pub unsafe extern "C" fn akar_paragraph(
    ctx: *mut AkarCtx,
    node_id: u64,
    text: *const c_char,
    style: *const AkarTextStyle,
) {
    let Some(ctx) = (unsafe { ctx.as_mut() }) else {
        return;
    };
    if text.is_null() {
        return;
    }
    let Ok(text_str) = unsafe { std::ffi::CStr::from_ptr(text) }.to_str() else {
        return;
    };
    let nid: akar_layout::NodeId = node_id.into();
    let overrides = c_text_style_to_rust(style);
    akar_components::akar_paragraph(
        &mut ctx.core,
        &ctx.layout,
        nid,
        text_str,
        overrides,
        &ctx.theme,
    );
}

#[no_mangle]
pub unsafe extern "C" fn akar_link(
    ctx: *mut AkarCtx,
    node_id: u64,
    text: *const c_char,
    style: *const AkarTextStyle,
) -> AkarLinkResult {
    let Some(ctx) = (unsafe { ctx.as_mut() }) else {
        return AkarLinkResult {
            clicked: false,
            hovered: false,
            pressed: false,
        };
    };
    if text.is_null() {
        return AkarLinkResult {
            clicked: false,
            hovered: false,
            pressed: false,
        };
    }
    let Ok(text_str) = unsafe { std::ffi::CStr::from_ptr(text) }.to_str() else {
        return AkarLinkResult {
            clicked: false,
            hovered: false,
            pressed: false,
        };
    };
    let nid: akar_layout::NodeId = node_id.into();
    let overrides = c_text_style_to_rust(style);
    let result = akar_components::akar_link(
        &mut ctx.core,
        &ctx.layout,
        nid,
        text_str,
        overrides,
        &ctx.theme,
    );
    AkarLinkResult {
        clicked: result.clicked,
        hovered: result.hovered,
        pressed: result.pressed,
    }
}

#[no_mangle]
pub unsafe extern "C" fn akar_card_layout(
    ctx: *mut AkarCtx,
    node_id: u64,
    options: *const AkarCardLayout,
) -> AkarCardSlots {
    let Some(ctx) = (unsafe { ctx.as_mut() }) else {
        return AkarCardSlots {
            header: 0,
            body: 0,
            footer: 0,
        };
    };
    let opts = if options.is_null() {
        akar_components::CardLayout::body_only(&ctx.theme)
    } else {
        let o = unsafe { &*options };
        akar_components::CardLayout {
            direction: if o.direction == 1 {
                akar_layout::FlexDirection::Row
            } else {
                akar_layout::FlexDirection::Column
            },
            gap: o.gap,
            padding: o.padding,
            has_header: o.has_header != 0,
            has_footer: o.has_footer != 0,
        }
    };
    let nid: akar_layout::NodeId = node_id.into();
    let slots = akar_components::akar_card_layout(&mut ctx.layout, nid, &opts);
    AkarCardSlots {
        header: slots.header.map(|n| n.into()).unwrap_or(0),
        body: slots.body.into(),
        footer: slots.footer.map(|n| n.into()).unwrap_or(0),
    }
}

#[no_mangle]
pub unsafe extern "C" fn akar_card(
    ctx: *mut AkarCtx,
    node_id: u64,
    slots: *const AkarCardSlots,
    style: *const AkarCardStyle,
) {
    let Some(ctx) = (unsafe { ctx.as_mut() }) else {
        return;
    };
    if slots.is_null() {
        return;
    }
    let s = unsafe { &*slots };
    let nid: akar_layout::NodeId = node_id.into();
    let rust_slots = akar_components::CardSlots {
        header: if s.header != 0 {
            Some(s.header.into())
        } else {
            None
        },
        body: s.body.into(),
        footer: if s.footer != 0 {
            Some(s.footer.into())
        } else {
            None
        },
    };
    let rust_style = if style.is_null() {
        akar_components::CardStyle::default(&ctx.theme)
    } else {
        let cs = unsafe { &*style };
        akar_components::CardStyle {
            background: cs.background,
            border_color: cs.border_color,
            border_width: cs.border_width,
            corner_radii: cs.corner_radii,
            shadow_blur: cs.shadow_blur,
            shadow_spread: cs.shadow_spread,
            shadow_color: cs.shadow_color,
            shadow_offset: cs.shadow_offset,
            separator_color: cs.separator_color,
        }
    };
    akar_components::akar_card(&mut ctx.core, &ctx.layout, nid, &rust_slots, &rust_style);
}

#[no_mangle]
pub unsafe extern "C" fn akar_navbar_layout(ctx: *mut AkarCtx, node_id: u64) -> AkarNavbarSlots {
    let Some(ctx) = (unsafe { ctx.as_mut() }) else {
        return AkarNavbarSlots {
            start: 0,
            center: 0,
            end: 0,
        };
    };
    let nid: akar_layout::NodeId = node_id.into();
    let slots = akar_components::akar_navbar_layout(&mut ctx.layout, nid, &ctx.theme);
    AkarNavbarSlots {
        start: slots.start.into(),
        center: slots.center.into(),
        end: slots.end.into(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn akar_navbar_painted(
    ctx: *mut AkarCtx,
    node_id: u64,
    style: *const AkarNavbarStyle,
) {
    let Some(ctx) = (unsafe { ctx.as_mut() }) else {
        return;
    };
    let nid: akar_layout::NodeId = node_id.into();
    let rust_style = if style.is_null() {
        akar_components::NavbarStyle::default(&ctx.theme)
    } else {
        let ns = unsafe { &*style };
        akar_components::NavbarStyle {
            background: ns.background,
            border_color: ns.border_color,
            border_width: ns.border_width,
            corner_radii: ns.corner_radii,
        }
    };
    akar_components::akar_navbar(&mut ctx.core, &ctx.layout, nid, &rust_style);
}

#[no_mangle]
pub unsafe extern "C" fn akar_button_styled(
    ctx: *mut AkarCtx,
    node_id: u64,
    text: *const c_char,
    variant: u32,
    style: *const AkarButtonStyle,
) -> AkarButtonResult {
    let Some(ctx) = (unsafe { ctx.as_mut() }) else {
        return AkarButtonResult {
            clicked: false,
            hovered: false,
            pressed: false,
        };
    };
    if text.is_null() {
        return AkarButtonResult {
            clicked: false,
            hovered: false,
            pressed: false,
        };
    }
    let Ok(text_str) = unsafe { std::ffi::CStr::from_ptr(text) }.to_str() else {
        return AkarButtonResult {
            clicked: false,
            hovered: false,
            pressed: false,
        };
    };
    let nid: akar_layout::NodeId = node_id.into();
    let rust_style = if style.is_null() {
        akar_components::ButtonStyle::empty()
    } else {
        let bs = unsafe { &*style };
        akar_components::ButtonStyle {
            fill: if bs.fill != 0 { Some(bs.fill) } else { None },
            hover_fill: if bs.hover_fill != 0 {
                Some(bs.hover_fill)
            } else {
                None
            },
            pressed_fill: if bs.pressed_fill != 0 {
                Some(bs.pressed_fill)
            } else {
                None
            },
            border_color: if bs.border_color != 0 {
                Some(bs.border_color)
            } else {
                None
            },
            content_color: if bs.content_color != 0 {
                Some(bs.content_color)
            } else {
                None
            },
            text_style: c_text_style_to_rust(&bs.text_style),
        }
    };
    let result = akar_components::akar_button_styled(
        &mut ctx.core,
        &ctx.layout,
        nid,
        text_str,
        c_button_variant_to_rust(variant),
        &rust_style,
        &ctx.theme,
    );
    AkarButtonResult {
        clicked: result.clicked,
        hovered: result.hovered,
        pressed: result.pressed,
    }
}

#[no_mangle]
pub unsafe extern "C" fn akar_badge_styled(
    ctx: *mut AkarCtx,
    node_id: u64,
    text: *const c_char,
    variant: u32,
    style: *const AkarBadgeStyle,
) {
    let Some(ctx) = (unsafe { ctx.as_mut() }) else {
        return;
    };
    if text.is_null() {
        return;
    }
    let Ok(text_str) = unsafe { std::ffi::CStr::from_ptr(text) }.to_str() else {
        return;
    };
    let nid: akar_layout::NodeId = node_id.into();
    let rust_style = if style.is_null() {
        akar_components::BadgeStyle::empty()
    } else {
        let bs = unsafe { &*style };
        akar_components::BadgeStyle {
            fill: if bs.fill != 0 { Some(bs.fill) } else { None },
            border_color: if bs.border_color != 0 {
                Some(bs.border_color)
            } else {
                None
            },
            content_color: if bs.content_color != 0 {
                Some(bs.content_color)
            } else {
                None
            },
            text_style: c_text_style_to_rust(&bs.text_style),
        }
    };
    akar_components::akar_badge_styled(
        &mut ctx.core,
        &ctx.layout,
        nid,
        text_str,
        c_badge_variant_to_rust(variant),
        &rust_style,
        &ctx.theme,
    );
}

#[no_mangle]
pub unsafe extern "C" fn akar_separator_styled(
    ctx: *mut AkarCtx,
    node_id: u64,
    style: *const AkarSeparatorStyle,
) {
    let Some(ctx) = (unsafe { ctx.as_mut() }) else {
        return;
    };
    let nid: akar_layout::NodeId = node_id.into();
    let rust_style = if style.is_null() {
        akar_components::SeparatorStyle::empty()
    } else {
        let ss = unsafe { &*style };
        akar_components::SeparatorStyle {
            color: if ss.color != 0 { Some(ss.color) } else { None },
            thickness: if ss.thickness > 0.0 {
                Some(ss.thickness)
            } else {
                None
            },
        }
    };
    akar_components::akar_separator_styled(
        &mut ctx.core,
        &ctx.layout,
        nid,
        &rust_style,
        &ctx.theme,
    );
}

#[no_mangle]
pub unsafe extern "C" fn akar_stat_styled(
    ctx: *mut AkarCtx,
    node_id: u64,
    title: *const c_char,
    value: *const c_char,
    description: *const c_char,
    style: *const AkarStatStyle,
) {
    let Some(ctx) = (unsafe { ctx.as_mut() }) else {
        return;
    };
    if title.is_null() || value.is_null() {
        return;
    }
    let Ok(title_str) = unsafe { std::ffi::CStr::from_ptr(title) }.to_str() else {
        return;
    };
    let Ok(value_str) = unsafe { std::ffi::CStr::from_ptr(value) }.to_str() else {
        return;
    };
    let desc_str = if description.is_null() {
        None
    } else {
        unsafe { std::ffi::CStr::from_ptr(description) }
            .to_str()
            .ok()
    };
    let nid: akar_layout::NodeId = node_id.into();
    let rust_style = if style.is_null() {
        akar_components::StatStyle::empty()
    } else {
        let ss = unsafe { &*style };
        akar_components::StatStyle {
            title_color: if ss.title_color != 0 {
                Some(ss.title_color)
            } else {
                None
            },
            value_color: if ss.value_color != 0 {
                Some(ss.value_color)
            } else {
                None
            },
            description_color: if ss.description_color != 0 {
                Some(ss.description_color)
            } else {
                None
            },
            title_text_style: c_text_style_to_rust(&ss.title_text_style),
            value_text_style: c_text_style_to_rust(&ss.value_text_style),
            description_text_style: c_text_style_to_rust(&ss.description_text_style),
        }
    };
    akar_components::akar_stat_styled(
        &mut ctx.core,
        &ctx.layout,
        nid,
        title_str,
        value_str,
        desc_str,
        &rust_style,
        &ctx.theme,
    );
}

#[no_mangle]
pub unsafe extern "C" fn akar_tab_bar_styled(
    ctx: *mut AkarCtx,
    node_id: u64,
    tabs: *const *const c_char,
    tab_count: u32,
    active_tab: u32,
    variant: u32,
    style: *const AkarTabBarStyle,
) -> AkarTabBarResponse {
    let Some(ctx) = (unsafe { ctx.as_mut() }) else {
        return AkarTabBarResponse { clicked_index: -1 };
    };
    if tabs.is_null() || tab_count == 0 {
        return AkarTabBarResponse { clicked_index: -1 };
    }
    let mut tab_strs: Vec<&str> = Vec::with_capacity(tab_count as usize);
    for i in 0..tab_count as usize {
        let ptr = unsafe { *tabs.add(i) };
        if ptr.is_null() {
            return AkarTabBarResponse { clicked_index: -1 };
        }
        match unsafe { std::ffi::CStr::from_ptr(ptr) }.to_str() {
            Ok(s) => tab_strs.push(s),
            Err(_) => return AkarTabBarResponse { clicked_index: -1 },
        }
    }
    let nid: akar_layout::NodeId = node_id.into();
    let rust_style = if style.is_null() {
        akar_components::TabBarStyle::empty()
    } else {
        let ts = unsafe { &*style };
        akar_components::TabBarStyle {
            active_color: if ts.active_color != 0 {
                Some(ts.active_color)
            } else {
                None
            },
            inactive_color: if ts.inactive_color != 0 {
                Some(ts.inactive_color)
            } else {
                None
            },
            indicator_color: if ts.indicator_color != 0 {
                Some(ts.indicator_color)
            } else {
                None
            },
        }
    };
    let result = akar_components::akar_tab_bar_styled(
        &mut ctx.core,
        &ctx.layout,
        nid,
        &tab_strs,
        active_tab as usize,
        c_tab_variant_to_rust(variant),
        &rust_style,
        &ctx.theme,
    );
    AkarTabBarResponse {
        clicked_index: result.clicked.map(|i| i as i32).unwrap_or(-1),
    }
}

#[cfg(test)]
mod tests {
    use super::{utf8_prefix_len, write_copy_to_ffi, write_value_to_ffi};

    #[test]
    fn utf8_prefix_never_splits_a_character() {
        assert_eq!(utf8_prefix_len("aé日", 0), 0);
        assert_eq!(utf8_prefix_len("aé日", 2), 1);
        assert_eq!(utf8_prefix_len("aé日", 3), 3);
        assert_eq!(utf8_prefix_len("aé日", 5), 3);
        assert_eq!(utf8_prefix_len("aé日", 6), 6);
    }

    #[test]
    fn value_write_updates_length_and_terminates_when_possible() {
        let mut buffer = [0x55; 5];
        let mut len = 0;
        let written = unsafe { write_value_to_ffi(buffer.as_mut_ptr(), &mut len, 5, "ééé") };
        assert_eq!(written, 4);
        assert_eq!(len, 4);
        assert_eq!(std::str::from_utf8(&buffer[..4]).unwrap(), "éé");
        assert_eq!(buffer[4], 0);
    }

    #[test]
    fn copy_write_reports_full_required_length() {
        let mut buffer = [0x55; 3];
        let (written, required) =
            unsafe { write_copy_to_ffi(Some("éé"), buffer.as_mut_ptr(), 3) };
        assert_eq!(written, 2);
        assert_eq!(required, 4);
        assert_eq!(std::str::from_utf8(&buffer[..2]).unwrap(), "é");
        assert_eq!(buffer[2], 0);
    }

    #[test]
    fn no_selection_copy_leaves_caller_buffer_untouched() {
        let mut buffer = [0x55; 4];
        let result = unsafe { write_copy_to_ffi(None, buffer.as_mut_ptr(), 4) };
        assert_eq!(result, (0, 0));
        assert_eq!(buffer, [0x55; 4]);
    }
}

#[cfg(test)]
mod component_c_api_tests {
    use super::*;
    use std::ffi::CString;

    fn sized_node(ctx: *mut AkarCtx, w: f32, h: f32) -> u64 {
        unsafe {
            let root = akar_new_flex_col(ctx);
            let node = akar_new_fixed_leaf(ctx, w, h);
            akar_add_child(ctx, root, node);
            akar_layout_compute(ctx, root, 800.0, 600.0);
            node
        }
    }

    #[test]
    fn heading_null_style_uses_defaults() {
        let ctx = unsafe { akar_ctx_mock() };
        unsafe { akar_begin_frame(ctx, 800, 600, 1.0) };
        let node = sized_node(ctx, 400.0, 60.0);
        let text = CString::new("Hello").unwrap();
        unsafe {
            akar_heading(ctx, node, text.as_ptr(), 0, std::ptr::null());
        }
        unsafe { akar_ctx_free(ctx) };
    }

    #[test]
    fn heading_with_style_override() {
        let ctx = unsafe { akar_ctx_mock() };
        unsafe { akar_begin_frame(ctx, 800, 600, 1.0) };
        let node = sized_node(ctx, 400.0, 60.0);
        let text = CString::new("Styled").unwrap();
        let style = AkarTextStyle {
            font_size: 48.0,
            color: 0xff0000ff,
            ..default_c_text_style_for_test()
        };
        unsafe {
            akar_heading(ctx, node, text.as_ptr(), 1, &style);
        }
        unsafe { akar_ctx_free(ctx) };
    }

    #[test]
    fn heading_levels_all_work() {
        let ctx = unsafe { akar_ctx_mock() };
        let text = CString::new("Level").unwrap();
        for level in 0..=3 {
            unsafe { akar_begin_frame(ctx, 800, 600, 1.0) };
            let node = sized_node(ctx, 400.0, 60.0);
            unsafe {
                akar_heading(ctx, node, text.as_ptr(), level, std::ptr::null());
            }
        }
        unsafe { akar_ctx_free(ctx) };
    }

    #[test]
    fn paragraph_null_style_uses_defaults() {
        let ctx = unsafe { akar_ctx_mock() };
        unsafe { akar_begin_frame(ctx, 800, 600, 1.0) };
        let node = sized_node(ctx, 300.0, 200.0);
        let text = CString::new("A paragraph.").unwrap();
        unsafe {
            akar_paragraph(ctx, node, text.as_ptr(), std::ptr::null());
        }
        unsafe { akar_ctx_free(ctx) };
    }

    #[test]
    fn paragraph_with_style_override() {
        let ctx = unsafe { akar_ctx_mock() };
        unsafe { akar_begin_frame(ctx, 800, 600, 1.0) };
        let node = sized_node(ctx, 300.0, 200.0);
        let text = CString::new("Styled paragraph").unwrap();
        let style = AkarTextStyle {
            color: 0x00ff00ff,
            ..default_c_text_style_for_test()
        };
        unsafe {
            akar_paragraph(ctx, node, text.as_ptr(), &style);
        }
        unsafe { akar_ctx_free(ctx) };
    }

    #[test]
    fn link_returns_hit_state() {
        let ctx = unsafe { akar_ctx_mock() };
        unsafe {
            akar_begin_frame(ctx, 800, 600, 1.0);
            akar_input_begin(ctx);
            akar_set_mouse_pos(ctx, 500.0, 500.0);
            akar_input_end(ctx);
        };
        let node = sized_node(ctx, 400.0, 60.0);
        let text = CString::new("Click").unwrap();
        let result = unsafe { akar_link(ctx, node, text.as_ptr(), std::ptr::null()) };
        assert!(!result.clicked);
        assert!(!result.hovered);
        assert!(!result.pressed);
        unsafe { akar_ctx_free(ctx) };
    }

    #[test]
    fn link_null_text_returns_all_false() {
        let ctx = unsafe { akar_ctx_mock() };
        unsafe { akar_begin_frame(ctx, 800, 600, 1.0) };
        let node = sized_node(ctx, 400.0, 60.0);
        let result = unsafe { akar_link(ctx, node, std::ptr::null(), std::ptr::null()) };
        assert!(!result.clicked);
        assert!(!result.hovered);
        assert!(!result.pressed);
        unsafe { akar_ctx_free(ctx) };
    }

    #[test]
    fn card_layout_body_only() {
        let ctx = unsafe { akar_ctx_mock() };
        unsafe { akar_begin_frame(ctx, 800, 600, 1.0) };
        let node = sized_node(ctx, 300.0, 200.0);
        let result = unsafe { akar_card_layout(ctx, node, std::ptr::null()) };
        assert_eq!(result.header, 0, "body-only should have no header");
        assert!(result.body != 0, "body should be non-zero");
        assert_eq!(result.footer, 0, "body-only should have no footer");
        unsafe { akar_ctx_free(ctx) };
    }

    #[test]
    fn card_layout_header_body_footer() {
        let ctx = unsafe { akar_ctx_mock() };
        unsafe { akar_begin_frame(ctx, 800, 600, 1.0) };
        let node = sized_node(ctx, 300.0, 200.0);
        let opts = AkarCardLayout {
            direction: 0,
            gap: 0.0,
            padding: 12.0,
            has_header: 1,
            has_footer: 1,
        };
        let result = unsafe { akar_card_layout(ctx, node, &opts) };
        assert!(result.header != 0, "header should be non-zero");
        assert!(result.body != 0, "body should be non-zero");
        assert!(result.footer != 0, "footer should be non-zero");
        assert_ne!(result.header, result.body);
        assert_ne!(result.body, result.footer);
        unsafe { akar_ctx_free(ctx) };
    }

    #[test]
    fn card_paint_with_null_style() {
        let ctx = unsafe { akar_ctx_mock() };
        unsafe { akar_begin_frame(ctx, 800, 600, 1.0) };
        let node = sized_node(ctx, 300.0, 200.0);
        let slots = unsafe { akar_card_layout(ctx, node, std::ptr::null()) };
        unsafe {
            akar_card(ctx, node, &slots, std::ptr::null());
        }
        unsafe { akar_ctx_free(ctx) };
    }

    #[test]
    fn navbar_layout_returns_three_distinct_slots() {
        let ctx = unsafe { akar_ctx_mock() };
        unsafe { akar_begin_frame(ctx, 800, 600, 1.0) };
        let node = sized_node(ctx, 800.0, 60.0);
        let slots = unsafe { akar_navbar_layout(ctx, node) };
        assert!(slots.start != 0);
        assert!(slots.center != 0);
        assert!(slots.end != 0);
        assert_ne!(slots.start, slots.center);
        assert_ne!(slots.center, slots.end);
        unsafe { akar_ctx_free(ctx) };
    }

    #[test]
    fn navbar_painted_null_style() {
        let ctx = unsafe { akar_ctx_mock() };
        unsafe { akar_begin_frame(ctx, 800, 600, 1.0) };
        let node = sized_node(ctx, 800.0, 60.0);
        unsafe {
            akar_navbar_painted(ctx, node, std::ptr::null());
        }
        unsafe { akar_ctx_free(ctx) };
    }

    #[test]
    fn button_styled_null_style() {
        let ctx = unsafe { akar_ctx_mock() };
        unsafe { akar_begin_frame(ctx, 800, 600, 1.0) };
        let node = sized_node(ctx, 120.0, 40.0);
        let text = CString::new("Click").unwrap();
        let result = unsafe { akar_button_styled(ctx, node, text.as_ptr(), 0, std::ptr::null()) };
        assert!(!result.clicked);
        unsafe { akar_ctx_free(ctx) };
    }

    #[test]
    fn button_styled_with_custom_fill() {
        let ctx = unsafe { akar_ctx_mock() };
        unsafe { akar_begin_frame(ctx, 800, 600, 1.0) };
        let node = sized_node(ctx, 120.0, 40.0);
        let text = CString::new("Click").unwrap();
        let style = AkarButtonStyle {
            fill: 0xFF0000FF,
            hover_fill: 0,
            pressed_fill: 0,
            border_color: 0,
            content_color: 0,
            text_style: default_c_text_style_for_test(),
        };
        let result = unsafe { akar_button_styled(ctx, node, text.as_ptr(), 0, &style) };
        assert!(!result.clicked);
        unsafe { akar_ctx_free(ctx) };
    }

    #[test]
    fn badge_styled_null_style() {
        let ctx = unsafe { akar_ctx_mock() };
        unsafe { akar_begin_frame(ctx, 800, 600, 1.0) };
        let node = sized_node(ctx, 100.0, 30.0);
        let text = CString::new("Badge").unwrap();
        unsafe {
            akar_badge_styled(ctx, node, text.as_ptr(), 1, std::ptr::null());
        }
        unsafe { akar_ctx_free(ctx) };
    }

    #[test]
    fn badge_styled_with_custom_fill() {
        let ctx = unsafe { akar_ctx_mock() };
        unsafe { akar_begin_frame(ctx, 800, 600, 1.0) };
        let node = sized_node(ctx, 100.0, 30.0);
        let text = CString::new("Badge").unwrap();
        let style = AkarBadgeStyle {
            fill: 0xFF0000FF,
            border_color: 0,
            content_color: 0,
            text_style: default_c_text_style_for_test(),
        };
        unsafe {
            akar_badge_styled(ctx, node, text.as_ptr(), 1, &style);
        }
        unsafe { akar_ctx_free(ctx) };
    }

    #[test]
    fn separator_styled_null_style() {
        let ctx = unsafe { akar_ctx_mock() };
        unsafe { akar_begin_frame(ctx, 800, 600, 1.0) };
        let node = sized_node(ctx, 200.0, 4.0);
        unsafe {
            akar_separator_styled(ctx, node, std::ptr::null());
        }
        unsafe { akar_ctx_free(ctx) };
    }

    #[test]
    fn separator_styled_with_custom_color() {
        let ctx = unsafe { akar_ctx_mock() };
        unsafe { akar_begin_frame(ctx, 800, 600, 1.0) };
        let node = sized_node(ctx, 200.0, 4.0);
        let style = AkarSeparatorStyle {
            color: 0xFF0000FF,
            thickness: 2.0,
        };
        unsafe {
            akar_separator_styled(ctx, node, &style);
        }
        unsafe { akar_ctx_free(ctx) };
    }

    #[test]
    fn stat_styled_null_style() {
        let ctx = unsafe { akar_ctx_mock() };
        unsafe { akar_begin_frame(ctx, 800, 600, 1.0) };
        let node = sized_node(ctx, 200.0, 120.0);
        let title = CString::new("Revenue").unwrap();
        let value = CString::new("$12,345").unwrap();
        let desc = CString::new("vs last month").unwrap();
        unsafe {
            akar_stat_styled(
                ctx,
                node,
                title.as_ptr(),
                value.as_ptr(),
                desc.as_ptr(),
                std::ptr::null(),
            );
        }
        unsafe { akar_ctx_free(ctx) };
    }

    #[test]
    fn stat_styled_without_description() {
        let ctx = unsafe { akar_ctx_mock() };
        unsafe { akar_begin_frame(ctx, 800, 600, 1.0) };
        let node = sized_node(ctx, 200.0, 120.0);
        let title = CString::new("Revenue").unwrap();
        let value = CString::new("$12,345").unwrap();
        unsafe {
            akar_stat_styled(
                ctx,
                node,
                title.as_ptr(),
                value.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
            );
        }
        unsafe { akar_ctx_free(ctx) };
    }

    #[test]
    fn stat_styled_with_custom_colors() {
        let ctx = unsafe { akar_ctx_mock() };
        unsafe { akar_begin_frame(ctx, 800, 600, 1.0) };
        let node = sized_node(ctx, 200.0, 120.0);
        let title = CString::new("Revenue").unwrap();
        let value = CString::new("$12,345").unwrap();
        let style = AkarStatStyle {
            title_color: 0xFF0000FF,
            value_color: 0x00FF00FF,
            description_color: 0,
            title_text_style: default_c_text_style_for_test(),
            value_text_style: default_c_text_style_for_test(),
            description_text_style: default_c_text_style_for_test(),
        };
        unsafe {
            akar_stat_styled(
                ctx,
                node,
                title.as_ptr(),
                value.as_ptr(),
                std::ptr::null(),
                &style,
            );
        }
        unsafe { akar_ctx_free(ctx) };
    }

    #[test]
    fn tab_bar_styled_null_style() {
        let ctx = unsafe { akar_ctx_mock() };
        unsafe { akar_begin_frame(ctx, 800, 600, 1.0) };
        let node = sized_node(ctx, 400.0, 40.0);
        let tab1 = CString::new("Tab A").unwrap();
        let tab2 = CString::new("Tab B").unwrap();
        let tabs = [tab1.as_ptr(), tab2.as_ptr()];
        let result =
            unsafe { akar_tab_bar_styled(ctx, node, tabs.as_ptr(), 2, 0, 0, std::ptr::null()) };
        assert_eq!(result.clicked_index, -1);
        unsafe { akar_ctx_free(ctx) };
    }

    #[test]
    fn tab_bar_styled_with_custom_colors() {
        let ctx = unsafe { akar_ctx_mock() };
        unsafe { akar_begin_frame(ctx, 800, 600, 1.0) };
        let node = sized_node(ctx, 400.0, 40.0);
        let tab1 = CString::new("Tab A").unwrap();
        let tab2 = CString::new("Tab B").unwrap();
        let tabs = [tab1.as_ptr(), tab2.as_ptr()];
        let style = AkarTabBarStyle {
            active_color: 0xFF0000FF,
            inactive_color: 0x333333FF,
            indicator_color: 0x00FF00FF,
        };
        let result = unsafe { akar_tab_bar_styled(ctx, node, tabs.as_ptr(), 2, 0, 0, &style) };
        assert_eq!(result.clicked_index, -1);
        unsafe { akar_ctx_free(ctx) };
    }

    #[test]
    fn c_text_style_sentinel_values_produce_none() {
        let style = AkarTextStyle {
            font_size: 0.0,
            line_height: 0.0,
            color: 0,
            font_weight: 0xFF,
            font_family: 0xFF,
            font_family_name_handle: AKAR_FONT_FAMILY_NAME_HANDLE_NONE,
            align: 0xFF,
            wrap: 0xFF,
        };
        let result = c_text_style_to_rust(&style);
        assert!(result.is_none());
    }

    #[test]
    fn c_text_style_overrides_produce_some() {
        let style = AkarTextStyle {
            font_size: 24.0,
            line_height: 0.0,
            color: 0xff0000ff,
            font_weight: 3,
            font_family: 1,
            font_family_name_handle: AKAR_FONT_FAMILY_NAME_HANDLE_NONE,
            align: 1,
            wrap: 1,
        };
        let result = c_text_style_to_rust(&style);
        assert!(result.is_some());
        let s = result.unwrap();
        assert_eq!(s.font_size, Some(24.0));
        assert_eq!(s.color, Some(0xff0000ff));
        assert_eq!(s.font_weight, Some(akar_components::FontWeight::Bold));
        assert_eq!(s.font_family, Some(akar_components::FontFamily::Serif));
        assert_eq!(s.align, Some(akar_components::TextAlign::Center));
        assert_eq!(s.wrap, Some(true));
    }

    #[test]
    fn heading_level_mapping() {
        assert!(matches!(
            c_heading_level_to_rust(0),
            akar_components::HeadingLevel::H1
        ));
        assert!(matches!(
            c_heading_level_to_rust(1),
            akar_components::HeadingLevel::H2
        ));
        assert!(matches!(
            c_heading_level_to_rust(2),
            akar_components::HeadingLevel::H3
        ));
        assert!(matches!(
            c_heading_level_to_rust(3),
            akar_components::HeadingLevel::H4
        ));
        assert!(matches!(
            c_heading_level_to_rust(99),
            akar_components::HeadingLevel::H1
        ));
    }

    #[test]
    fn button_variant_mapping() {
        assert!(matches!(
            c_button_variant_to_rust(0),
            akar_components::ButtonVariant::Solid
        ));
        assert!(matches!(
            c_button_variant_to_rust(1),
            akar_components::ButtonVariant::Outline
        ));
        assert!(matches!(
            c_button_variant_to_rust(2),
            akar_components::ButtonVariant::Ghost
        ));
    }

    #[test]
    fn badge_variant_mapping() {
        assert!(matches!(
            c_badge_variant_to_rust(0),
            akar_components::BadgeVariant::Default
        ));
        assert!(matches!(
            c_badge_variant_to_rust(1),
            akar_components::BadgeVariant::Primary
        ));
        assert!(matches!(
            c_badge_variant_to_rust(2),
            akar_components::BadgeVariant::Success
        ));
        assert!(matches!(
            c_badge_variant_to_rust(3),
            akar_components::BadgeVariant::Warning
        ));
        assert!(matches!(
            c_badge_variant_to_rust(4),
            akar_components::BadgeVariant::Error
        ));
        assert!(matches!(
            c_badge_variant_to_rust(5),
            akar_components::BadgeVariant::Info
        ));
    }

    #[test]
    fn c_text_style_name_handle_overrides_generic_family() {
        let style = AkarTextStyle {
            font_family: 1,
            font_family_name_handle: 7,
            ..default_c_text_style_for_test()
        };
        let resolved = c_text_style_to_rust(&style).expect("handle sets an override");
        assert_eq!(
            resolved.font_family,
            Some(akar_components::FontFamily::Named(7))
        );
    }

    #[test]
    fn load_font_bytes_rejects_invalid_arguments() {
        let ctx = unsafe { akar_ctx_mock() };
        let mut handle = 12345u32;
        assert_eq!(
            unsafe {
                akar_load_font_bytes(std::ptr::null_mut(), [0u8; 4].as_ptr(), 4, &mut handle)
            },
            AKAR_FONT_LOAD_INVALID_ARGUMENT
        );
        assert_eq!(
            unsafe { akar_load_font_bytes(ctx, std::ptr::null(), 4, &mut handle) },
            AKAR_FONT_LOAD_INVALID_ARGUMENT
        );
        assert_eq!(
            unsafe { akar_load_font_bytes(ctx, [0u8; 4].as_ptr(), 0, &mut handle) },
            AKAR_FONT_LOAD_INVALID_ARGUMENT
        );
        assert_eq!(handle, 12345, "out_handle untouched on failure");
        unsafe { akar_ctx_free(ctx) };
    }

    #[test]
    fn load_font_bytes_rejects_garbage_data() {
        let ctx = unsafe { akar_ctx_mock() };
        let garbage = [0u8; 64];
        let mut handle = 12345u32;
        assert_eq!(
            unsafe { akar_load_font_bytes(ctx, garbage.as_ptr(), 64, &mut handle) },
            AKAR_FONT_LOAD_INVALID_DATA
        );
        assert_eq!(handle, 12345);
        unsafe { akar_ctx_free(ctx) };
    }

    #[test]
    fn load_font_bytes_accepts_valid_font_and_null_out_handle() {
        let ctx = unsafe { akar_ctx_mock() };
        let bytes = akar_core::font_source::IBM_PLEX_SANS_REGULAR;
        let mut handle = u32::MAX;
        assert_eq!(
            unsafe { akar_load_font_bytes(ctx, bytes.as_ptr(), bytes.len() as u32, &mut handle) },
            AKAR_FONT_LOAD_OK
        );
        assert_ne!(handle, u32::MAX);
        assert_eq!(
            unsafe {
                akar_load_font_bytes(ctx, bytes.as_ptr(), bytes.len() as u32, ptr::null_mut())
            },
            AKAR_FONT_LOAD_OK
        );
        unsafe { akar_ctx_free(ctx) };
    }

    fn default_c_text_style_for_test() -> AkarTextStyle {
        AkarTextStyle {
            font_size: 0.0,
            line_height: 0.0,
            color: 0,
            font_weight: 0xFF,
            font_family: 0xFF,
            font_family_name_handle: AKAR_FONT_FAMILY_NAME_HANDLE_NONE,
            align: 0xFF,
            wrap: 0xFF,
        }
    }
}
