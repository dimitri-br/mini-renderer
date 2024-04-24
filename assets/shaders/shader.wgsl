// Vertex Shader
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>
};

@vertex
fn vertex_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 0.5),  // Top of the triangle
        vec2<f32>(-0.5, -0.5),  // Left bottom corner of the triangle
        vec2<f32>(0.5, -0.5)  // Right bottom corner of the triangle
    );

    var colors = array<vec4<f32>, 3>(
        vec4<f32>(1.0, 0.0, 0.0, 1.0),  // Red
        vec4<f32>(0.0, 1.0, 0.0, 1.0),  // Green
        vec4<f32>(0.0, 0.0, 1.0, 1.0)  // Blue
    );

    var output: VertexOutput;
    output.clip_position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    output.color = colors[vertex_index];
    return output;
}

// Fragment Shader
@fragment
fn fragment_main(@location(0) color: vec4<f32>) -> @location(0) vec4<f32> {
    return color;  // Output the color passed from the vertex shader
}
