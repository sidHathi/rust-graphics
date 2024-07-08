use std::sync::{Arc, Mutex};

use super::{component::{Component, ComponentFunctions}, component_store::ComponentKey, errors::EngineError, model_renderer::{ModelRenderer, RenderableModel}, Scene};
use cgmath::Point3;
use async_trait::async_trait;

pub struct TestComponent {
  key: ComponentKey,
  parent: Option<ComponentKey>,
  local_position: Point3<f32>,
  model: Option<RenderableModel>,
  active: bool,
}

#[async_trait(?Send)]
impl ComponentFunctions for TestComponent {
  async fn init(
    &mut self,
    scene: &mut Scene,
    key: ComponentKey,
    parent: Option<ComponentKey>
  ) {
    self.key = key;
    self.active = true;
    self.parent = parent;
    // could be made safer
    if let Ok(model) = scene.load_model("dice.obj", None, key).await {
      self.model = Some(model);
    } else {
      self.model = None;
    }
  }

  fn update(&mut self, scene: &mut Scene, dt: instant::Duration) {
    todo!()
  }

  fn render(&self, scene: &mut Scene) -> Result<(), EngineError> {
    if self.model.is_none() {
      // println!("No model to render");
      return Ok(());
    }
    scene.render_model(&self.model.as_ref().unwrap())
  }
}

impl TestComponent {
  pub fn new() -> TestComponent {
    Self {
      key: ComponentKey::zero(),
      parent: None,
      model: None,
      local_position: Point3 { x: 0., y: 0., z: 0. },
      active: false
    }
  }
}