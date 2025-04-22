// Vertex shader

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

// Define canvas size constants (replace with uniforms later if needed)
const CANVAS_WIDTH: f32 = 350.0;
const CANVAS_HEIGHT: f32 = 350.0;

@vertex
fn vs_main(
    input: VertexInput, // Use VertexInput again
) -> VertexOutput {
    var out: VertexOutput;

    // Normalize pixel coordinates (0..width, 0..height) to clip space (-1..1, 1..-1)
    // Note: Y is flipped because clip space Y goes up, while pixel space Y often goes down.
    let normalized_pos = vec2<f32>(
        (input.position.x / CANVAS_WIDTH) * 2.0 - 1.0,
        (1.0 - (input.position.y / CANVAS_HEIGHT)) * 2.0 - 1.0 // Flip Y
    );

    out.clip_position = vec4<f32>(normalized_pos, 0.0, 1.0);
    out.color = input.color; // Pass color through
    return out;
}

// Fragment shader

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color; // Return the interpolated vertex color
}
