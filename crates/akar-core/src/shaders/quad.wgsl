struct Params {
    screen_resolution: vec2<u32>,
}

struct QuadInstance {
    rect: vec4<f32>,           // x, y, w, h in physical pixels
    fill: vec4<f32>,           // RGBA
    border_color: vec4<f32>,   // RGBA
    border_width: f32,
    corner_radii: vec4<f32>,   // tl, tr, br, bl
    z: f32,
    _pad: f32,
}

@group(0) @binding(0) var<storage, read> quads: array<QuadInstance>;
@group(0) @binding(1) var<uniform> params: Params;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) local_pos: vec2<f32>,
    @location(1) half_size: vec2<f32>,
    @location(2) fill: vec4<f32>,
    @location(3) border_color: vec4<f32>,
    @location(4) @interpolate(flat) border_width: f32,
    @location(5) @interpolate(flat) corner_radii: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32, @builtin(instance_index) instance_index: u32) -> VertexOutput {
    let q = quads[instance_index];

    var uv: vec2<f32>;
    switch vertex_index {
        case 0u: { uv = vec2<f32>(0.0, 0.0); }
        case 1u: { uv = vec2<f32>(1.0, 0.0); }
        case 2u: { uv = vec2<f32>(0.0, 1.0); }
        case 3u: { uv = vec2<f32>(0.0, 1.0); }
        case 4u: { uv = vec2<f32>(1.0, 0.0); }
        default: { uv = vec2<f32>(1.0, 1.0); }
    }

    let pixel_pos = q.rect.xy + uv * q.rect.zw;
    let clip_pos = 2.0 * pixel_pos / vec2<f32>(params.screen_resolution) - 1.0;

    var out: VertexOutput;
    out.position = vec4<f32>(clip_pos.x, -clip_pos.y, q.z, 1.0);
    out.local_pos = uv * q.rect.zw - q.rect.zw * 0.5;
    out.half_size = q.rect.zw * 0.5;
    out.fill = q.fill;
    out.border_color = q.border_color;
    out.border_width = q.border_width;
    out.corner_radii = q.corner_radii;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let abs_pos = abs(in.local_pos);

    var corner_radius: f32;
    if in.local_pos.x < 0.0 && in.local_pos.y < 0.0 {
        corner_radius = in.corner_radii.x;
    } else if in.local_pos.x >= 0.0 && in.local_pos.y < 0.0 {
        corner_radius = in.corner_radii.y;
    } else if in.local_pos.x >= 0.0 && in.local_pos.y >= 0.0 {
        corner_radius = in.corner_radii.z;
    } else {
        corner_radius = in.corner_radii.w;
    }

    if corner_radius == 0.0 && in.border_width <= 0.0 {
        return in.fill;
    }

    let d = abs_pos - in.half_size + vec2<f32>(corner_radius);
    let outer_dist = length(max(vec2<f32>(0.0), d)) + min(0.0, max(d.x, d.y)) - corner_radius;
    let outer_alpha = saturate(0.5 - outer_dist);

    if outer_alpha <= 0.0 {
        discard;
    }

    if in.border_width <= 0.0 {
        return vec4<f32>(in.fill.rgb, in.fill.a * outer_alpha);
    }

    let inner_corner = max(0.0, corner_radius - in.border_width);
    let inner_half = max(vec2<f32>(0.0), in.half_size - vec2<f32>(in.border_width));
    let di = abs_pos - inner_half + vec2<f32>(inner_corner);
    let inner_dist = length(max(vec2<f32>(0.0), di)) + min(0.0, max(di.x, di.y)) - inner_corner;
    let inner_alpha = saturate(0.5 - inner_dist);

    let color = mix(in.border_color, in.fill, inner_alpha);
    return vec4<f32>(color.rgb, color.a * outer_alpha);
}
