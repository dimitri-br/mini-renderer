struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) texCoords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) texCoords: vec2<f32>
};

struct Transform {
    model: mat4x4<f32>,
};

struct Camera {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> transform: Transform;

@group(0) @binding(1)
var<uniform> camera: Camera;

@vertex
fn vertex_main(vertex_input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    output.clip_position = camera.projection * camera.view * transform.model * vec4<f32>(vertex_input.position, 1.0);
    output.texCoords = vertex_input.texCoords;  // Pass texture coordinates to fragment shader

    return output;
}



@group(1) @binding(0)
var diffuse: texture_2d<f32>;
@group(1) @binding(1)
var diffuse_sampler: sampler;

struct FragmentInput {
    @location(0) texCoords: vec2<f32>
};

@fragment
fn fragment_main(input: FragmentInput) -> @location(0) vec4<f32> {
    let color = textureSample(diffuse, diffuse_sampler, input.texCoords);
    return vec4<f32>(color.r, color.g, color.b, color.a);
}
