
struct CameraUniform {
    view_proj: mat4x4<f32>,
}

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;


@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) color: vec4<f32>,
};

struct InstanceInput {
    @location(5) model_matrix_col_1: vec4<f32>,
    @location(6) model_matrix_col_2: vec4<f32>,
    @location(7) model_matrix_col_3: vec4<f32>,
    @location(8) model_matrix_col_4: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_col_1,
        instance.model_matrix_col_2,
        instance.model_matrix_col_3,
        instance.model_matrix_col_4,
    );

    var out: VertexOutput;

    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);
    out.tex_coords = model.tex_coords;
    out.normal = (model_matrix * vec4<f32>(model.normal, 0.0)).xyz;
    out.color = model.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texture_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    return texture_color * in.color;
}