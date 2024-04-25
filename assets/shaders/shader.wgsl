struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) texCoords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) texCoords: vec2<f32>
};

@vertex
fn vertex_main(vertex_input: VertexInput) -> VertexOutput {
    var scaled_position: vec3<f32> = vertex_input.position * 0.1;  // Scale down the position by 0.1
    var output: VertexOutput;
    output.clip_position = vec4<f32>(scaled_position, 1.0);  // Convert position to vec4 with w=1
    output.texCoords = vertex_input.texCoords;  // Pass texture coordinates to fragment shader
    return output;
}



@group(0) @binding(0)
var diffuse: texture_2d<f32>;
@group(0) @binding(1)
var diffuse_sampler: sampler;

struct FragmentInput {
    @location(0) texCoords: vec2<f32>
};

@fragment
fn fragment_main(input: FragmentInput) -> @location(0) vec4<f32> {
    let color = textureSample(diffuse, diffuse_sampler, input.texCoords);
    return vec4<f32>(color.r, color.g, color.b, color.a);
}
