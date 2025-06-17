// Simple rectangle shader without instancing
// Each rectangle is rendered separately with 6 vertices (2 triangles)

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    // Convert to clip space (simple orthographic projection for 2D)
    let clip_x = (input.position.x / 400.0) - 1.0; // Assuming 800px width
    let clip_y = 1.0 - (input.position.y / 300.0); // Assuming 600px height, flip Y
    
    output.clip_position = vec4<f32>(clip_x, clip_y, 0.0, 1.0);
    output.color = input.color;
    
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}