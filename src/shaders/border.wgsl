struct BorderInfo {
    size: vec2<f32>,
    width: f32,
};

@group(0) @binding(0)
var<uniform> border: BorderInfo;

@vertex
fn vs_main(@location(0) position: vec2<f32>) -> FragmentInput {
    var out: FragmentInput;
    let ndc = (position / (border.size / 2));
    out.position = vec4(ndc, 0.0, 1.0);
    out.window_pos = position;
    return out;
}

struct FragmentInput {
    @builtin(position) position: vec4<f32>,
    @location(0) window_pos: vec2<f32>,
};


@fragment
fn fs_main(in: FragmentInput) -> @location(0) vec4<f32> {
    let dist = sdBox(in.window_pos, border.size * 0.5 - vec2(border.width * 0.5));
    let edge = smoothstep(1.0, 3.0, abs(dist));
    return vec4(vec3(1.0 - edge), 1.0);
}

fn sdBox(p: vec2f, b: vec2f) -> f32 {
  let d = abs(p) - b;
  return length(max(d, vec2f(0.))) + min(max(d.x, d.y), 0.);
}