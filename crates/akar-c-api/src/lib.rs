#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_void};
use std::ptr;

use akar_components::{AkarTheme, ButtonVariant, AKAR_THEME_DARK};
use akar_core::AkarCore;
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
    use akar_layout::{Style, Size, Dimension, length};
    let ctx = unsafe { &mut *ctx };
    let style = Style {
        size: Size {
            width:  if w > 0.0 { length(w) } else { Dimension::auto() },
            height: if h > 0.0 { length(h) } else { Dimension::auto() },
        },
        flex_shrink: 0.0,
        ..Default::default()
    };
    ctx.layout.new_leaf(style).into()
}

#[no_mangle]
pub unsafe extern "C" fn akar_new_flex_row(ctx: *mut AkarCtx) -> u64 {
    use akar_layout::{Style, Display, FlexDirection, Size, Dimension};
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
    use akar_layout::{Style, Display, FlexDirection, Size, Dimension};
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
    let child_node:  akar_layout::NodeId = child.into();
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
    ctx.layout.compute(
        root_node,
        (Some(width), Some(height)),
        |_, _, _, _, _| Size::ZERO,
    );
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

    let label_bytes =
        unsafe { std::slice::from_raw_parts(label as *const u8, label_len as usize) };
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
