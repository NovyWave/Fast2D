// Vertex shader

// Uniforms structure matching the Rust struct (with padding)
// Note: WGSL does not support @size attribute; manual padding is used for alignment
//
// Why is padding needed?
// Uniform buffer objects (UBOs) in WGSL and wgpu must follow strict alignment rules:
// - Each field must be aligned to a 4-byte boundary (f32), but the struct as a whole must be aligned to 16 bytes.
// - If the struct is not a multiple of 16 bytes, the GPU may read invalid data or cause validation errors.
// - Adding two f32 padding fields ensures the struct size is 16 bytes, matching what Rust expects and what the GPU requires.
struct CanvasUniforms {
    width: f32,
    height: f32,
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
    in: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    // Transform pixel coordinates to Normalized Device Coordinates (NDC)
    // NDC X: (pixel_x / width) * 2.0 - 1.0
    // NDC Y: (pixel_y / height) * -2.0 + 1.0  (Invert Y)
    let ndc_x = (in.position.x / canvas.width) * 2.0 - 1.0;
    let ndc_y = (in.position.y / canvas.height) * -2.0 + 1.0; // Invert Y axis
    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.color = in.color; // Pass linear color through
    return out;
}

// Fragment shader

// sRGB conversion function (always used)
// Clamps input to [0,1] to avoid out-of-range values
fn linear_to_srgb(linear: vec3<f32>) -> vec3<f32> {
    let clamped = clamp(linear, vec3<f32>(0.0), vec3<f32>(1.0));
    let cutoff = clamped < vec3<f32>(0.0031308);
    let higher = 1.055 * pow(clamped, vec3<f32>(1.0 / 2.4)) - 0.055;
    let lower = clamped * 12.92;
    return select(higher, lower, cutoff);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let linear_color = in.color;
    // Always apply manual sRGB conversion
    return vec4<f32>(linear_to_srgb(linear_color.rgb), linear_color.a);
}
