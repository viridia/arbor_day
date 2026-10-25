// Strokes a polyline's lines, each `vec4(a.xy, b.xy)` in logical pixels from the node's top left,
// by distance to the nearest. A uniform array rather than a storage buffer, for WebGL2.
import bevy_ui_render::ui_vertex_output::UiVertexOutput;

// `MAX_LINES` in `plot.rs`.
const MAX_LINES: u32 = 256u;

struct Path {
    color: vec4<f32>,
    width: f32,
    // The inverse of the scale factor: `in.size` is in physical pixels.
    scale: f32,
    count: u32,
    lines: array<vec4<f32>, MAX_LINES>,
}

@group(1) @binding(0)
var<uniform> path: Path;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let point = in.size * path.scale * in.uv;
    var nearest = 1e12;
    for (var i = 0u; i < min(path.count, MAX_LINES); i = i + 1u) {
        let line = path.lines[i];
        nearest = min(nearest, distance_squared_to_line(point, line.xy, line.zw));
    }
    let distance = sqrt(nearest);
    let coverage = 1.0 - smoothstep(path.width * 0.5 - 0.5, path.width * 0.5 + 0.5, distance);
    return vec4<f32>(path.color.rgb, path.color.a * coverage);
}

fn distance_squared_to_line(point: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let along = b - a;
    // A zero-length line is its point.
    let t = clamp(dot(point - a, along) / max(dot(along, along), 1e-12), 0.0, 1.0);
    let offset = point - a - along * t;
    return dot(offset, offset);
}
