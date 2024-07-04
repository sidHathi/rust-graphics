use std::sync::{Arc, Mutex};

use super::{component::{Component, ComponentTrait}, component_models::ComponentModels};
use cgmath::Point3;
use async_trait::async_trait;

pub struct TestComponent<'a> {
  parent: Option<&'a Component<'a>>,
  local_position: Point3<f32>,
  model_key: String,
  active: bool,
}

#[async_trait(?Send)]
impl<'a> ComponentTrait<'a> for TestComponent<'a> {
  async fn init<'b>(
    &mut self,
    parent: Option<&'a Component<'a>>, 
    model_store: Arc<Mutex<ComponentModels>>,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    tex_layout: &wgpu::BindGroupLayout
  ) where 'a: 'b{
    self.active = true;
    self.parent = parent;
    self.model_key = model_store.lock().unwrap().load_model("dice.obj", device, queue, tex_layout).await;
  }

  fn update(&mut self, dt: instant::Duration) {
    todo!()
  }

  fn position(&mut self, pos: cgmath::Point3<f32>) {
    self.local_position = pos;
  }

  fn children(&self) -> Vec<&Component> {
      todo!()
  }

  fn model_keys(&self) -> Vec<String> {
    [self.model_key.clone()].into()
  }
}

impl<'a> TestComponent<'a> {
  pub fn new() -> TestComponent<'a> {
    Self {
      parent: None,
      model_key: "".into(),
      local_position: Point3 { x: 0., y: 0., z: 0. },
      active: false
    }
  }
}