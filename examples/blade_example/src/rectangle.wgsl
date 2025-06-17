// Rectangle shader for Blade Graphics
// Uses instanced rendering to draw multiple rectangles efficiently

struct VertexInput {
    @location(0) position: vec2<f32>,     // QuadVertex data
    @location(1) rect_pos: vec2<f32>,     // RectangleInstance: rectangle position
    @location(2) rect_size: vec2<f32>,    // RectangleInstance: rectangle size
    @location(3) rect_color: vec4<f32>,   // RectangleInstance: rectangle color
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    // Transform unit quad (0,0 to 1,1) to rectangle coordinates
    let world_pos = input.rect_pos + input.position * input.rect_size;
    
    // Convert to clip space (assuming window coordinates)
    // This is a simple orthographic projection for 2D
    let clip_x = (world_pos.x / 400.0) - 1.0; // Assuming 800px width, center at 400
    let clip_y = 1.0 - (world_pos.y / 300.0); // Assuming 600px height, center at 300, flip Y
    
    output.clip_position = vec4<f32>(clip_x, clip_y, 0.0, 1.0);
    output.color = input.rect_color;
    
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}