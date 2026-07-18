pub(crate) fn scale_color(c: u32, factor: f32) -> u32 {
    let r = (((c >> 24) & 0xFF) as f32 * factor).min(255.0) as u32;
    let g = (((c >> 16) & 0xFF) as f32 * factor).min(255.0) as u32;
    let b = (((c >> 8) & 0xFF) as f32 * factor).min(255.0) as u32;
    let a = c & 0xFF;
    (r << 24) | (g << 16) | (b << 8) | a
}

pub(crate) fn color_to_f32(c: u32) -> [f32; 4] {
    [
        ((c >> 24) & 0xFF) as f32 / 255.0,
        ((c >> 16) & 0xFF) as f32 / 255.0,
        ((c >> 8) & 0xFF) as f32 / 255.0,
        (c & 0xFF) as f32 / 255.0,
    ]
}

pub(crate) fn f32_to_color(c: [f32; 4]) -> u32 {
    let r = (c[0] * 255.0).clamp(0.0, 255.0) as u32;
    let g = (c[1] * 255.0).clamp(0.0, 255.0) as u32;
    let b = (c[2] * 255.0).clamp(0.0, 255.0) as u32;
    let a = (c[3] * 255.0).clamp(0.0, 255.0) as u32;
    (r << 24) | (g << 16) | (b << 8) | a
}
