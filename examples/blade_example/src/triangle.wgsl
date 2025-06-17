// Simple triangle shader for Blade Graphics validation
// Vertex shader
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    // Define triangle vertices in clip space
    var positions = array<vec2<f32>, 3>(
        vec2<f32>( 0.0,  0.5),  // Top
        vec2<f32>(-0.5, -0.5),  // Bottom left
        vec2<f32>( 0.5, -0.5)   // Bottom right
    );
    
    return vec4<f32>(positions[vertex_index], 0.0, 1.0);
}

// Fragment shader
@fragment  
fn fs_main() -> @location(0) vec4<f32> {
    // Return a bright red color
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}