use std::collections::HashMap;

use wgpu::util::DeviceExt;

use crate::graphics::{load_model, Model};

use super::component::Component;

pub struct ComponentModels {
  models: HashMap<String, Model>,
}

impl ComponentModels {
  pub fn new() -> ComponentModels {
    Self {
      models: HashMap::new()
    }
  }

  // loads a model into gpu memory adds and indexes it for retreival
  // returns key
  pub async fn load_model(
    &mut self,
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    tex_layout: &wgpu::BindGroupLayout,
  ) -> String {
    let model = load_model(file_name, device, queue, tex_layout).await.unwrap();
    self.models.insert(file_name.into(), model);
    file_name.into()
  }

  pub fn get_component_models(&self, components: Vec<Component>) -> Vec<&Model> {
    let keys = components.iter()
      .map(|comp| comp.get_model_keys())
      .collect::<Vec<Vec<String>>>()
      .concat();
    let mut model_refs: Vec<&Model> = Vec::new();
    for key in keys {
      if self.models.contains_key(&key) {
        model_refs.push(self.models.get(&key).unwrap());
      }
    }
    model_refs
  }
}
