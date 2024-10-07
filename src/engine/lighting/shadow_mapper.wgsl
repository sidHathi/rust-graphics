struct LightShadowUniform {
  view_pos: vec4<f32>,
  view_proj: mat4x4<f32>,
  direction: vec4<f32>,
}
@group(0) @binding(0)
var<uniform> light: LightShadowUniform;

struct VertexInput {
  @location(0) position: vec3<f32>,
  @location(1) tex_coords: vec2<f32>,
  @location(2) normal: vec3<f32>,
  @location(3) tangent: vec3<f32>,
  @location(4) bitangent: vec3<f32>,
}

struct InstanceInput {
  @location(5) model_matrix_0: vec4<f32>,
  @location(6) model_matrix_1: vec4<f32>,
  @location(7) model_matrix_2: vec4<f32>,
  @location(8) model_matrix_3: vec4<f32>,
  @location(9) normal_matrix_0: vec3<f32>,
  @location(10) normal_matrix_1: vec3<f32>,
  @location(11) normal_matrix_2: vec3<f32>,
  @location(12) opacity: f32,
}

@vertex
fn vs_main(
  model: VertexInput,
  instance: InstanceInput
) -> @builtin(position) vec4f {
  let model_matrix = mat4x4<f32>(
    instance.model_matrix_0,
    instance.model_matrix_1,
    instance.model_matrix_2,
    instance.model_matrix_3,
  );
  return light.view_proj * model_matrix * vec4(model.position, 1.0);
}