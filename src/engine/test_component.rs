use std::sync::{Arc, Mutex};

use super::{component::{Component, ComponentFunctions}, component_store::ComponentKey, transforms::{ComponentTransform, ModelTransform}, errors::EngineError, model_renderer::{ModelRenderer, RenderableModel}, Scene};
use cgmath::{Point3, Quaternion, Vector3};
use async_trait::async_trait;
use super::test_child_component::TestChildComponent;

pub struct TestComponent {
  key: ComponentKey,
  parent: Option<ComponentKey>,
  local_position: Point3<f32>,
  model: Option<RenderableModel>,
  child: Option<Component>,
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
    // println!("model loaded");

    // load a child of same type
    let child_underlying = TestChildComponent::new();
    let child = Component::new(child_underlying, scene, Some(self.key)).await;
    self.child = child;
  }

  fn update(&mut self, scene: &mut Scene, dt: instant::Duration) {
    todo!()
  }

  fn render(&self, scene: &mut Scene) -> Result<(), EngineError> {
    if self.model.is_none() {
      // println!("No model to render");
      return Ok(());
    }
    let res: Result<(), EngineError> = scene.render_model(&self.model.as_ref().unwrap(), ModelTransform::default());
    if let Err(e) = res {
        return Err(e);
    }
    if let Some(child_safe) = self.child.clone() {
      return child_safe.render(scene, Some(ComponentTransform::local(
        Vector3::new(20., 0., -50.), 
        Quaternion::new(0.5, 0., 0., 0.)
      )));
    }
    Ok(())
  }
}

impl TestComponent {
  pub fn new() -> TestComponent {
    Self {
      key: ComponentKey::zero(),
      parent: None,
      model: None,
      child: None,
      local_position: Point3 { x: 0., y: 0., z: 0. },
      active: false
    }
  }
}