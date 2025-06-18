// Simple centered triangle shader - guaranteed visible
// Vertex shader
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    // Create a simple centered triangle
    var positions = array<vec2<f32>, 3>(
        vec2<f32>( 0.0,  0.8),  // Top center
        vec2<f32>(-0.8, -0.8),  // Bottom left
        vec2<f32>( 0.8, -0.8)   // Bottom right
    );
    
    return vec4<f32>(positions[vertex_index], 0.0, 1.0);
}

// Fragment shader
@fragment  
fn fs_main() -> @location(0) vec4<f32> {
    // Return bright MAGENTA - this should be impossible to miss
    return vec4<f32>(1.0, 0.0, 1.0, 1.0);
}