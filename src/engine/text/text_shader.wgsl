// Vertex shader
struct CameraUniform {
  view_pos: vec4<f32>,
  view_proj: mat4x4<f32>,
};
@group(1) @binding(0) // 1.
var<uniform> camera: CameraUniform;

struct Light {
  position: vec3<f32>,
  color: vec3<f32>
}
@group(2) @binding(0)
var<uniform> light: Light;

struct VertexInput {
  @location(0) position: vec3<f32>,
  @location(1) tex_coords: vec2<f32>,
}

struct ModelInput {
  @location(2) model_matrix_0: vec4<f32>,
  @location(3) model_matrix_1: vec4<f32>,
  @location(4) model_matrix_2: vec4<f32>,
  @location(5) model_matrix_3: vec4<f32>,
}

struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
  @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(
  vert: VertexInput,
  model: ModelInput,
) -> VertexOutput {
  var out: VertexOutput;
  let model_matrix = mat4x4<f32>(
    model.model_matrix_0,
    model.model_matrix_1,
    model.model_matrix_2,
    model.model_matrix_3,
  );
  let world_position: vec4<f32> = model_matrix * vec4<f32>(vert.position, 1.0);
  out.clip_position = camera.view_proj * world_position;
  out.tex_coords = vec2<f32>(vert.tex_coords.x, vert.tex_coords.y);
  return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(
  in: VertexOutput
) -> @location(0) vec4<f32> {
  // return vec4<f32>(0., 0., 0., 1.);
  return vec4<f32>(0., 0., 0., textureSample(t_diffuse, s_diffuse, in.tex_coords).r);
  // let sampled_val = textureSample(t_diffuse, s_diffuse, in.tex_coords);
  // return vec4<f32>(sampled_val.r, sampled_val.r, sampled_val.r, 1.0);
}
