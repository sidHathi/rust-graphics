use std::collections::HashMap;

use anyhow::Error;
use cgmath::{Matrix4, Point3, Quaternion, Rotation3, Vector3};
use wgpu::{util::DeviceExt};

use crate::graphics::{load_model, Instance, InstanceRaw, Model};

use super::{component::Component, component_store::ComponentKey, errors::EngineError, renderable_model::{RenderInstance, RenderSettings}, transform_queue::TransformQueue, transforms::{ComponentTransform, GlobalTransform, ModelTransform, TransformType}};
use super::renderable_model::RenderableModel;


pub struct RenderData {
  model: Model,
  instances: Vec<Instance>,
  instance_buf: wgpu::Buffer,
  opacity: Option<f32>,
  scale: Option<Vector3<f32>>
}

pub struct ModelRenderer {
  // maps filenames to tuple of model + instance buffer
  next_idx: u32,
  render_list: Vec<RenderableModel>,
  models: HashMap<RenderableModel, RenderData>,
  transform_queue: TransformQueue,
  component_transform_cache: HashMap<ComponentKey, Matrix4<f32>>
}

impl ModelRenderer {
  pub fn new() -> ModelRenderer {
    Self {
      next_idx: 0,
      render_list: Vec::new(),
      models: HashMap::new(),
      transform_queue: TransformQueue::new(),
      component_transform_cache: HashMap::new()
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
    let default_inst = Instance {
      position: Vector3 { x: 0., y: 0., z: 0. },
      rotation: cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0)),
      opacity: 1.,
      scale: Vector3::new(1., 1., 1.)
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
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST
      }
    );

    let key = RenderableModel::new(self.next_idx, component_key, filename.into());
    
    let data: RenderData = RenderData {
      model,
      instances: instance_vec,
      instance_buf,
      opacity: None,
      scale: None
    };
    self.models.insert(key.clone(), data);
    Ok(key)
  }

  pub fn update_model_instances_forced(
    &mut self,
    model: &RenderableModel,
    new_render_instances: Vec<RenderInstance>,
    queue: &wgpu::Queue,
  ) -> Result<(), EngineError> {
    if !self.models.contains_key(model) {
      return Err(EngineError::ArgumentError { index: 1, name: "model".into() });
    }

    let instances = self.transform_queue.transform_instances(new_render_instances);
    let mut render_data = self.models.remove(model).unwrap();
    render_data.instances = instances.clone();
    let instance_data = instances
      .iter()
      .map(Instance::to_raw)
      .collect::<Vec<InstanceRaw>>();
    queue.write_buffer(&render_data.instance_buf, 0, bytemuck::cast_slice(&instance_data));

    self.models.insert(model.clone(), render_data);
    Ok(())
  }

  pub fn start_component_render(&mut self, transform: Option<ComponentTransform>, key: ComponentKey) {
    let transform_unwrapped = transform.unwrap_or(ComponentTransform::default());
    self.transform_queue.push(transform_unwrapped);
    self.component_transform_cache.insert(key, self.transform_queue.get_transform_matrix());
  }

  pub fn end_component_render(&mut self) {
    self.transform_queue.pop();
  }

  pub fn update_render_model(
    &mut self, 
    model: &RenderableModel,
    new_render_instances: Vec<RenderInstance>, 
    queue: &wgpu::Queue,
    device: &wgpu::Device,
  ) -> Result<(), EngineError> {
    if !self.models.contains_key(&model) {
      return Err(EngineError::ArgumentError { index: 1, name: "model".into() })
    }

    let new_instances = self.transform_queue.transform_instances(new_render_instances);
    let instance_vec = self.models.get(&model).unwrap().instances.clone();
    let mut needs_buf_update = false;
    for (i, instance) in new_instances.iter().enumerate() {
      if instance_vec[i] != instance.clone() {
        needs_buf_update = true;
        break;
      }
    }
    
    if needs_buf_update {
      let mut render_data = self.models.remove(&model).unwrap();
      render_data.instances = new_instances.clone();
      let instance_data = render_data.instances
        .iter()
        .map(Instance::to_raw)
        .collect::<Vec<InstanceRaw>>();

      queue.write_buffer(&render_data.instance_buf, 0, bytemuck::cast_slice(&instance_data));
      self.models.insert(model.clone(), render_data);
    }
    Ok(())
  }

  pub fn render_from_cache(&mut self, model: &RenderableModel) -> Result<(), EngineError> {
    if !self.models.contains_key(model) {
      return Err(EngineError::ArgumentError { index: 1, name: "model".into() })
    }
    self.render_list.push(model.clone());
    Ok(())
  }

  pub fn render(
    &mut self, 
    model: &RenderableModel, 
    render_settings: RenderSettings, 
    queue: &wgpu::Queue,
    device: &wgpu::Device
  ) -> Result<(), EngineError> {
    if !self.models.contains_key(model) {
      return Err(EngineError::ArgumentError { index: 1, name: "model".into() })
    }

    let render_instances = render_settings.to_render_instances(&self.models.get(model).unwrap().model);
    let res = self.update_render_model(model, render_instances, queue, device);
    self.render_list.push(model.clone());
    res
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

  pub fn get_position_cache(&self) -> &HashMap<ComponentKey, Matrix4<f32>> {
    &self.component_transform_cache
  }
}
