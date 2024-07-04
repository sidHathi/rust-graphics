use std::{ops::Deref, rc::Rc, sync::{Arc, Mutex, MutexGuard}};

use cgmath::Point3;

use crate::graphics::{DrawModel, Model};

use super::component_models::ComponentModels;
use async_trait::async_trait;

#[async_trait(?Send)]
pub trait ComponentTrait<'a> {
  // initialize the component
  async fn init<'b>(
    &mut self,
    parent: Option<&'a Component<'a>>,
    model_store: Arc<Mutex<ComponentModels>>,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    tex_layout: &wgpu::BindGroupLayout
  ) where 'a: 'b;

  // update is called every frame
  fn update(&mut self, dt: instant::Duration);

  // position the component with repsect to its parent
  fn position(&mut self, pos: Point3<f32>);

  // get all children
  fn children(&self) -> Vec<&Component>;

  // get models to be rendered when this component is rendered
  fn model_keys(&self) -> Vec<String>;
}

pub struct Component<'a> {
  underlying: Arc<Mutex<dyn ComponentTrait<'a>>>,
}

impl<'a> Component<'a> {
  pub fn new(underlying: impl ComponentTrait<'a> + 'static) -> Component<'a> {
    Self {
      underlying: Arc::new(Mutex::new(underlying))
    }
  }

  pub fn init<'b>(
    &mut self, 
    parent: Option<&'a Component<'a>>, 
    model_store:  Arc<Mutex<ComponentModels>>,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    tex_layout: &wgpu::BindGroupLayout
  ) where 'a: 'b {
    self.underlying.lock().unwrap().init(parent, model_store, device, queue, tex_layout);
  }

  pub fn update(&mut self, dt: instant::Duration) {
    self.underlying.lock().unwrap().update(dt);
  }

  pub fn position(&mut self, pos: Point3<f32>) {
    self.underlying.lock().unwrap().position(pos);
  }

  pub fn get_model_keys(&self) -> Vec<String> {
    let underlying_guard = self.underlying.lock().unwrap();
    let mut keys = underlying_guard.model_keys().clone();
    let mut child_keys: Vec<String> = underlying_guard
      .children()
      .iter()
      .map(|child| child.get_model_keys().clone())
      .collect::<Vec<Vec<String>>>()
      .concat()
      .clone();
    keys.append(&mut child_keys);
    keys
  }
}