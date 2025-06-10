struct VertexInput {
    @location(0) clip_pos: vec2<f32>,
};

struct PrimitiveGroup {
    @location(5) aabb_center: vec2<f32>,
    @location(6) aabb_half: vec2<f32>,
    @location(7) start: u32,
    @location(8) end: u32,
};

@group(0) @binding(0)
var<uniform> map_world_clip: mat4x4<f32>;

struct PrimitiveIndex {index: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

@group(1) @binding(0)
var<storage, read> primitives_indices: array<PrimitiveIndex>;

struct Primitive {
    transform: mat4x4<f32>,
    color: vec4<f32>,
    shape: u32,

    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

@group(1) @binding(1)
var<storage, read> primitives: array<Primitive>;


@vertex
fn vs_main(
    vert: VertexInput,
    instance: PrimitiveGroup,
) -> FragmentInput {
    let world_pos = vert.clip_pos * instance.aabb_half + instance.aabb_center;

    var out: FragmentInput;
    out.clip_pos = map_world_clip * vec4<f32>(world_pos, 0.0, 1.0);;
    out.world_pos = world_pos;

    out.prim_group_start = instance.start;
    out.prim_group_end = instance.end;
    return out;
}


struct FragmentInput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) world_pos: vec2<f32>,
    @location(1) prim_group_start: u32,
    @location(2) prim_group_end: u32,
};

const K: f32 = 0.9;

@fragment
fn fs_main(in: FragmentInput) -> @location(0) vec4<f32> {
    let blend_scale = 10.0;
    let color_falloff = 10.0;
    let clamp_inside = -0.1;

    var weighted_sdf_sum: f32 = 0.0;
    var sdf_weight_sum: f32 = 0.0;

    var color_total: vec4<f32> = vec4<f32>(0.0);
    var weight_total: f32 = 0.0;

    for (var i = in.prim_group_start; i < in.prim_group_end; i = i + 1u) {
        let idx = primitives_indices[i].index;
        let primitive = primitives[idx];

// sdf
        let unit_pos = transform_2d_point(primitive.transform, in.world_pos);
        var sdf: f32;
        if (primitive.shape == 0u) {
            sdf = circle_sdf(unit_pos);
        } else {
            sdf = regular_polygon_sdf(primitive.shape, unit_pos);
        }

        let clamped_sdf = max(sdf, clamp_inside);

// dist
        let sdf_weight = exp(-blend_scale * clamped_sdf);
        weighted_sdf_sum += sdf * sdf_weight;
        sdf_weight_sum += sdf_weight;

// color
        let color_weight = exp(-color_falloff * abs(sdf));
        color_total += primitive.color * color_weight;
        weight_total += color_weight;
    }

    let blended_sdf = weighted_sdf_sum / sdf_weight_sum;

    let color = color_total / max(weight_total, 1e-6);

    let edge0 = 0.0;
    let edge1 = 0.1;

    let alpha = smoothstep(edge1, edge0, abs(blended_sdf));

    if (alpha < 1e-3) {
        discard;
    }

    return vec4<f32>(color.rgb, alpha);
}



fn transform_2d_point(_mat: mat4x4<f32>, _point: vec2<f32>) -> vec2<f32> {
    let extended = vec4<f32>(_point, 0.0, 1.0);
    let transformed = _mat * extended;
    return transformed.xy;
}

fn smooth_min_within_radius(a: f32, b: f32, k: f32) -> f32 {
    let h = clamp(0.5 + 0.5 * (b - a) / k, 0.0, 1.0);
    let blend = mix(b, a, h) - k * h * (1.0 - h);
    let hard_min = min(a, b);

    return select(hard_min, blend, abs(a - b) < k);
}