use std::{ops::Deref, rc::Rc, sync::{Arc, Mutex, MutexGuard}};

use cgmath::Point3;

use crate::graphics::{DrawModel, Model};

use super::{component_store::ComponentKey, errors::EngineError, model_renderer::ModelRenderer, Scene};
use async_trait::async_trait;

#[async_trait(?Send)]
pub trait ComponentFunctions {
  // initialize the component
  async fn init(
    &mut self,
    scene: &mut Scene,
    key: ComponentKey,
    parent: Option<ComponentKey>,
  );

  // update is called every frame
  fn update(&mut self, scene: &mut Scene, dt: instant::Duration) {
    return;
  }

  // get models to be rendered when this component is rendered
  fn render(&self, scene: &mut Scene) -> Result<(), EngineError> {
    Ok(())
  }
}

#[derive(Clone)]
pub struct Component {
  pub key: ComponentKey,
  underlying: Arc<Mutex<dyn ComponentFunctions>>,
}

impl Component {
  pub async fn new(
    underlying: impl ComponentFunctions + 'static,
    scene: &mut Scene,
    parent: Option<ComponentKey>
  ) -> Option<Component> {
    let mut component = Self {
      key: ComponentKey::zero(),
      underlying: Arc::new(Mutex::new(underlying))
    };
    let key_res = scene.components.insert(component.clone());
    if let Ok(key) = key_res {
      component.key = key;
      component.clone().init(scene, key.clone(), parent).await;
      return Some(component);
    }
    None
  }

  pub async fn init(
    &mut self, 
    scene: &mut Scene,
    key: ComponentKey,
    parent: Option<ComponentKey>,
  ) {
    self.underlying.lock().unwrap().init(scene, key, parent).await;
  }

  pub fn update(&mut self, scene: &mut Scene, dt: instant::Duration) {
    self.underlying.lock().unwrap().update(scene, dt);
  }

  pub fn render(&self, scene: &mut Scene) -> Result<(), EngineError> {
    self.underlying.lock().unwrap().render(scene)
  }
}