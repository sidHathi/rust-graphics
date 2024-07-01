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
  @location(2) normal: vec3<f32>,
  @location(3) tangent: vec3<f32>,
  @location(4) bitangent: vec3<f32>,
}

struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
  @location(0) tex_coords: vec2<f32>,
  @location(1) world_normal: vec3<f32>,
  @location(2) world_position: vec3<f32>,
};

@vertex
fn vs_main(
  model: VertexInput
) -> VertexOutput {
  var out: VertexOutput;
  out.tex_coords = model.tex_coords;
  out.world_normal = model.normal;
  var world_position: vec4<f32> = vec4<f32>(model.position, 1.0);
  out.world_position = world_position.xyz;
  out.clip_position = camera.view_proj * world_position;
  return out;
}


@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;
@group(0) @binding(2)
var t_normal: texture_2d<f32>;
@group(0) @binding(3)
var s_normal: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let object_color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);
  
  // We don't need (or want) much ambient light, so 0.1 is fine
  let ambient_strength = 0.1;
  let ambient_color = light.color * ambient_strength;

  let light_dir = normalize(light.position - in.world_position);

  let diffuse_strength = max(dot(in.world_normal, light_dir), 0.0);
  let diffuse_color = light.color * diffuse_strength;

  let result = (ambient_color + diffuse_color) * object_color.xyz;

  return vec4<f32>(result, object_color.a);
}