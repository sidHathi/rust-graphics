use std::collections::HashMap;

use anyhow::Error;
use cgmath::{Point3, Quaternion, Rotation3, Vector3};
use wgpu::{util::DeviceExt};

use crate::graphics::{load_model, Instance, InstanceRaw, Model};

use super::{component::Component, component_store::ComponentKey, transforms::{ComponentTransform, ModelTransform, TransformType}, errors::EngineError, transform_queue::TransformQueue};

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
  transform_queue: TransformQueue
}

impl ModelRenderer {
  pub fn new() -> ModelRenderer {
    Self {
      next_idx: 0,
      render_list: Vec::new(),
      models: HashMap::new(),
      transform_queue: TransformQueue::new()
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
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST
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

  pub fn start_component_render(&mut self, transform: Option<ComponentTransform>) {
    let transform_unwrapped = transform.unwrap_or(ComponentTransform::default());
    self.transform_queue.push(transform_unwrapped);
  }

  pub fn end_component_render(&mut self) {
    self.transform_queue.pop();
  }

  pub fn update_render_model(
    &mut self, 
    model: &RenderableModel,
    transform: ModelTransform, 
    queue: &wgpu::Queue,
    device: &wgpu::Device,
  ) -> Result<(), EngineError> {
    if !self.models.contains_key(&model) {
      return Err(EngineError::ArgumentError { index: 1, name: "model".into() })
    }

    let mut instanced = self.models.get(&model).unwrap().instanced;
    let mut global_pos = self.models.get(&model).unwrap().global_pos;
    let mut global_rot = self.models.get(&model).unwrap().global_rot;
    let mut instance_vec = self.models.get(&model).unwrap().instances.clone();
    let mut needs_buf_update = false;
    match transform.clone() {
      ModelTransform::Single { transform_type, pos, rot } => {
        if transform_type == TransformType::Global {
          if global_pos != pos || global_rot != rot {
            needs_buf_update = true;
            global_pos = pos;
            global_rot = rot;
            instance_vec[0] = Instance {
              position: pos.clone(),
              rotation: rot.clone()
            }
          }
        } else {
          let transformed = self.transform_queue.transform_model(&transform);
          match transformed {
            ModelTransform::Single { transform_type: _, pos: pos_t, rot: rot_t } => {
              if global_pos != pos_t || global_rot != rot_t {
                needs_buf_update = true;
                global_pos = pos_t;
                global_rot = rot_t;
                instance_vec[0] = Instance {
                  position: pos_t.clone(),
                  rotation: rot_t.clone()
                }
              }
            }
            _ => {}
          }

        }
      }
      ModelTransform::Instanced { transform_type, instances } => {
        if !instanced {
          instanced = true;
          needs_buf_update = true;
        }
        match transform_type {
          TransformType::Global => {
            for (idx, instance) in instances.iter().enumerate() {
              if instance_vec[idx] != instance.clone() {
                needs_buf_update = true;
                break;
              }
            }
            instance_vec = instances.clone()
          },
          TransformType::Local => {
            let transformed = self.transform_queue.transform_model(&transform);
            match transformed {
              ModelTransform::Instanced { transform_type: _, instances: instances_t } => {
                for (idx, instance) in instances_t.iter().enumerate() {
                  if instance_vec[idx] != instance.clone() {
                    needs_buf_update = true;
                    break;
                  }
                }
                instance_vec = instances_t.clone()
              }
              _ => {}
            }
          }
        }
      }
    }
    if needs_buf_update {
      let mut render_data = self.models.remove(&model).unwrap();
      render_data.instanced = instanced;
      render_data.global_pos = global_pos;
      render_data.global_rot = global_rot;
      render_data.instances = instance_vec;
      println!("updated render data -> global pos: {:?}, rotation: {:?}, instances: {:?}", render_data.global_pos, render_data.global_rot, render_data.instances);
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
    transform: ModelTransform, 
    queue: &wgpu::Queue,
    device: &wgpu::Device
  ) -> Result<(), EngineError> {
    if !self.models.contains_key(model) {
      return Err(EngineError::ArgumentError { index: 1, name: "model".into() })
    }
    let res = self.update_render_model(model, transform.clone(), queue, device);
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
}
