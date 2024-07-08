use std::collections::HashMap;

use anyhow::Error;
use cgmath::{Point3, Quaternion, Rotation3, Vector3};
use wgpu::{util::DeviceExt};

use crate::graphics::{load_model, Instance, InstanceRaw, Model};

use super::{component::Component, component_store::ComponentKey, errors::EngineError};

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct RenderableModel {
  pub index: u32,
  component: ComponentKey,
  filename: String,
}

pub struct RenderData {
  model: Model,
  instanced: bool,
  global_pos: Vector3<f32>,
  global_rot: Quaternion<f32>,
  instances: Vec<Instance>,
  instance_buf: wgpu::Buffer
}

pub struct ModelRenderer {
  // maps filenames to tuple of model + instance buffer
  next_idx: u32,
  render_list: Vec<RenderableModel>,
  models: HashMap<RenderableModel, RenderData>,
}

impl ModelRenderer {
  pub fn new() -> ModelRenderer {
    Self {
      next_idx: 0,
      render_list: Vec::new(),
      models: HashMap::new()
    }
  }

  // loads a model into gpu memory adds and indexes it for retreival
  // returns key
  pub async fn load_model(
    &mut self,
    filename: &str,
    instances: Option<Vec<Instance>>,
    component_key: ComponentKey,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    tex_layout: &wgpu::BindGroupLayout,
  ) -> Result<RenderableModel, EngineError> {
    let model_res = load_model(filename, device, queue, tex_layout).await;
    if let Err(err) = model_res {
      println!("model load failed!");
      return Err(EngineError::ModelLoadError { err, filename: filename.into() } );
    }

    let model = model_res.unwrap();
    let instanced = !(instances.is_none());
    let default_inst = Instance {
      position: Vector3 { x: 0., y: 0., z: 0. },
      rotation: cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
    };
    let instance_vec: Vec<Instance> = instances.unwrap_or([default_inst.clone()].into());
    let instance_data = instance_vec
      .iter()
      .map(Instance::to_raw)
      .collect::<Vec<InstanceRaw>>();
    let instance_buf = device.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label: Some("Instance buffer"),
        contents: bytemuck::cast_slice(&instance_data),
        usage: wgpu::BufferUsages::VERTEX
      }
    );

    let key = RenderableModel {
      index: self.next_idx,
      component: component_key,
      filename: filename.into()
    };
    
    let data: RenderData = RenderData {
      model,
      instanced,
      global_pos: instance_vec.get(0).unwrap_or(&default_inst.clone()).position.clone(),
      global_rot: instance_vec.get(0).unwrap_or(&default_inst.clone()).rotation.clone(),
      instances: instance_vec,
      instance_buf
    };
    self.models.insert(key.clone(), data);
    Ok(key)
  }

  pub fn position_model(
    &mut self,
    model: &RenderableModel,
    new_pos: Vector3<f32>,
    queue: &wgpu::Queue,
  ) -> Result<(), EngineError> {
    if !self.models.contains_key(model) {
      return Err(EngineError::ArgumentError { index: 1, name: "model".into() });
    }
    let mut render_data = self.models.remove(model).unwrap();
    if render_data.global_pos == new_pos && render_data.instances[0].position == new_pos {
      return Ok(());
    }
    let current_rot = render_data.global_rot.clone();
    render_data.instances[0] = Instance {
      position: new_pos.clone(),
      rotation: current_rot
    };
    render_data.global_pos = new_pos.clone();

    let instance_data = render_data.instances
      .iter()
      .map(Instance::to_raw)
      .collect::<Vec<InstanceRaw>>();
    queue.write_buffer(&render_data.instance_buf, 0, bytemuck::cast_slice(&instance_data));
    self.models.insert(model.clone(), render_data);
    Ok(())
  }

  pub fn rotate_model(
    &mut self,
    model: &RenderableModel,
    new_rot: Quaternion<f32>,
    queue: &wgpu::Queue,
  ) -> Result<(), EngineError> {
    if !self.models.contains_key(model) {
      return Err(EngineError::ArgumentError { index: 1, name: "model".into() });
    }
    let mut render_data = self.models.remove(model).unwrap();
    if render_data.global_rot == new_rot && render_data.instances[0].rotation == new_rot {
      return Ok(());
    }
    let current_pos = render_data.global_pos.clone();
    render_data.instances[0] = Instance {
      position: current_pos,
      rotation: new_rot
    };
    render_data.global_rot = new_rot.clone();

    let instance_data = render_data.instances
      .iter()
      .map(Instance::to_raw)
      .collect::<Vec<InstanceRaw>>();
    queue.write_buffer(&render_data.instance_buf, 0, bytemuck::cast_slice(&instance_data));
    self.models.insert(model.clone(), render_data);
    Ok(())
  }

  pub fn update_model_instances(
    &mut self,
    model: &RenderableModel,
    new_instance_vec: Vec<Instance>,
    queue: &wgpu::Queue,
  ) -> Result<(), EngineError> {
    if !self.models.contains_key(model) {
      return Err(EngineError::ArgumentError { index: 1, name: "model".into() });
    }

    let mut render_data = self.models.remove(model).unwrap();
    render_data.instances = new_instance_vec.clone();
    if new_instance_vec.len() > 0 {
      render_data.global_pos = new_instance_vec[0].position.clone();
      render_data.global_rot = new_instance_vec[0].rotation.clone();
    }
    let instance_data = new_instance_vec
      .iter()
      .map(Instance::to_raw)
      .collect::<Vec<InstanceRaw>>();
    queue.write_buffer(&render_data.instance_buf, 0, bytemuck::cast_slice(&instance_data));

    self.models.insert(model.clone(), render_data);
    Ok(())
  }

  pub fn render(&mut self, model: &RenderableModel) -> Result<(), EngineError> {
    if !self.models.contains_key(model) {
      return Err(EngineError::ArgumentError { index: 1, name: "model".into() })
    }
    self.render_list.push(model.clone());
    Ok(())
  }

  pub fn clear(&mut self) {
    self.render_list.clear()
  }

  pub fn get_rendering_models(&self) -> Vec<(&Model, &wgpu::Buffer)> {
    self.render_list.iter()
      .map(|rm| self.models.get(rm))
      .filter(|rd| !rd.is_none())
      .map(|rd| (&rd.unwrap().model,&rd.unwrap().instance_buf))
      .into_iter()
      .collect::<Vec<(&Model, &wgpu::Buffer)>>()
  }
}
