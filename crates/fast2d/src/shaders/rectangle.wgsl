// Vertex shader

// Uniforms structure matching the Rust struct (with padding)
struct CanvasUniforms {
    width: f32,
    height: f32,
    // Add padding to meet 16-byte alignment requirement
    _padding1: f32,
    _padding2: f32,
};

// Bind group 0, binding 0 for the uniforms
@group(0) @binding(0)
var<uniform> canvas: CanvasUniforms;

// Input vertex structure matching Rust's ColoredVertex
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>, // Expect linear color input
};

// Output structure to pass data to the fragment shader
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>, // Pass linear color to fragment shader
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    // Transform pixel coordinates to Normalized Device Coordinates (NDC)
    // NDC X: (pixel_x / width) * 2.0 - 1.0
    // NDC Y: (pixel_y / height) * -2.0 + 1.0  (Invert Y)
    let ndc_x = (model.position.x / canvas.width) * 2.0 - 1.0;
    let ndc_y = (model.position.y / canvas.height) * -2.0 + 1.0; // Invert Y axis
    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.color = model.color; // Pass linear color through
    return out;
}

// Fragment shader

// sRGB conversion function (always used)
fn linear_to_srgb(linear: vec3<f32>) -> vec3<f32> {
    let cutoff = linear < vec3<f32>(0.0031308);
    let higher = 1.055 * pow(linear, vec3<f32>(1.0 / 2.4)) - 0.055;
    let lower = linear * 12.92;
    return select(higher, lower, cutoff);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let linear_color = in.color;
    // Always apply manual sRGB conversion
    return vec4<f32>(linear_to_srgb(linear_color.rgb), linear_color.a);
}
