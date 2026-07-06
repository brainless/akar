struct Params {
    screen_resolution: vec2<u32>,
}

struct QuadInstance {
    rect: vec4<f32>,
    fill: vec4<f32>,
    border_color: vec4<f32>,
    corner_radii: vec4<f32>,
    border_width: f32,
    z: f32,
    shadow_blur: f32,
    shadow_spread: f32,
    shadow_color: vec4<f32>,
    shadow_offset: vec2<f32>,
    _pad: vec2<f32>,
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
    @location(6) @interpolate(flat) shadow_color: vec4<f32>,
    @location(7) @interpolate(flat) shadow_params: vec4<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    let q = quads[instance_index];

    let pad_l = max(0.0, -q.shadow_offset.x + q.shadow_blur + q.shadow_spread);
    let pad_r = max(0.0,  q.shadow_offset.x + q.shadow_blur + q.shadow_spread);
    let pad_t = max(0.0, -q.shadow_offset.y + q.shadow_blur + q.shadow_spread);
    let pad_b = max(0.0,  q.shadow_offset.y + q.shadow_blur + q.shadow_spread);

    let ex = vec4<f32>(
        q.rect.x - pad_l,
        q.rect.y - pad_t,
        q.rect.z + pad_l + pad_r,
        q.rect.w + pad_t + pad_b,
    );

    var uv: vec2<f32>;
    switch vertex_index {
        case 0u: { uv = vec2<f32>(0.0, 0.0); }
        case 1u: { uv = vec2<f32>(1.0, 0.0); }
        case 2u: { uv = vec2<f32>(0.0, 1.0); }
        case 3u: { uv = vec2<f32>(0.0, 1.0); }
        case 4u: { uv = vec2<f32>(1.0, 0.0); }
        default: { uv = vec2<f32>(1.0, 1.0); }
    }

    let pixel_pos = ex.xy + uv * ex.zw;
    let clip_pos = 2.0 * pixel_pos / vec2<f32>(params.screen_resolution) - 1.0;

    let box_center = q.rect.xy + q.rect.zw * 0.5;

    var out: VertexOutput;
    out.position       = vec4<f32>(clip_pos.x, -clip_pos.y, q.z, 1.0);
    out.local_pos      = pixel_pos - box_center;
    out.half_size      = q.rect.zw * 0.5;
    out.fill           = q.fill;
    out.border_color   = q.border_color;
    out.border_width   = q.border_width;
    out.corner_radii   = q.corner_radii;
    out.shadow_color   = q.shadow_color;
    out.shadow_params  = vec4<f32>(q.shadow_offset.x, q.shadow_offset.y, q.shadow_blur, q.shadow_spread);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let abs_pos      = abs(in.local_pos);
    let shad_offset  = in.shadow_params.xy;
    let shad_blur    = in.shadow_params.z;
    let shad_spread  = in.shadow_params.w;

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

    if corner_radius == 0.0 && in.border_width <= 0.0 && in.shadow_color.a == 0.0 {
        return in.fill;
    }

    let d = abs_pos - in.half_size + vec2<f32>(corner_radius);
    let outer_dist  = length(max(vec2<f32>(0.0), d)) + min(0.0, max(d.x, d.y)) - corner_radius;
    let outer_alpha = saturate(0.5 - outer_dist);

    var shadow_a = 0.0;
    if in.shadow_color.a > 0.0 {
        let s_pos  = abs(in.local_pos - shad_offset);
        let s_half = in.half_size + vec2<f32>(shad_spread);
        let s_d    = s_pos - s_half + vec2<f32>(corner_radius);
        let s_dist = length(max(vec2<f32>(0.0), s_d)) + min(0.0, max(s_d.x, s_d.y)) - corner_radius;
        shadow_a   = in.shadow_color.a * clamp(0.5 - s_dist / max(shad_blur, 0.001), 0.0, 1.0);
        shadow_a  *= (1.0 - outer_alpha);
    }

    if outer_alpha <= 0.0 && shadow_a <= 0.0 {
        discard;
    }

    var main_rgb = vec3<f32>(0.0);
    var main_a   = 0.0;
    if outer_alpha > 0.0 {
        if in.border_width <= 0.0 {
            main_rgb = in.fill.rgb;
            main_a   = in.fill.a * outer_alpha;
        } else {
            let inner_corner = max(0.0, corner_radius - in.border_width);
            let inner_half   = max(vec2<f32>(0.0), in.half_size - vec2<f32>(in.border_width));
            let di           = abs_pos - inner_half + vec2<f32>(inner_corner);
            let inner_dist   = length(max(vec2<f32>(0.0), di)) + min(0.0, max(di.x, di.y)) - inner_corner;
            let inner_alpha  = saturate(0.5 - inner_dist);
            let color        = mix(in.border_color, in.fill, inner_alpha);
            main_rgb         = color.rgb;
            main_a           = color.a * outer_alpha;
        }
    }

    let out_a = main_a + shadow_a * (1.0 - main_a);
    if out_a <= 0.0 { discard; }
    let out_rgb = (main_rgb * main_a + in.shadow_color.rgb * shadow_a * (1.0 - main_a)) / out_a;
    return vec4<f32>(out_rgb, out_a);
}
