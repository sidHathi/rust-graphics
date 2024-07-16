use std::{any::Any, ops::Deref, rc::Rc, sync::{Arc, Mutex, MutexGuard}};

use cgmath::Point3;

use crate::graphics::{DrawModel, Model};

use super::{component_store::ComponentKey, errors::EngineError, events::EventListener, model_renderer::ModelRenderer, state::StateListener, transforms::ComponentTransform, Scene};
use async_trait::async_trait;

#[async_trait(?Send)]
pub trait ComponentFunctions: Any + Send + EventListener + StateListener {
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
  underlying: Arc<Mutex<Box<dyn ComponentFunctions>>>,
}

impl Component {
  pub async fn new(
    underlying: impl ComponentFunctions + 'static,
    scene: &mut Scene,
    parent: Option<ComponentKey>
  ) -> Option<Component> {
    let mut component = Self {
      key: ComponentKey::zero(),
      underlying: Arc::new(Mutex::new(Box::new(underlying)))
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

  pub fn update(&self, scene: &mut Scene, dt: instant::Duration) {
    self.underlying.lock().unwrap().update(scene, dt);
  }

  pub fn render(&self, scene: &mut Scene, transform: Option<ComponentTransform>) -> Result<(), EngineError> {
    scene.model_renderer.start_component_render(transform);
    let res = self.underlying.lock().unwrap().render(scene);
    scene.model_renderer.end_component_render();
    res
  }
}

impl EventListener for Component {
  fn handle_event(&mut self, event: super::events::Event) {
    // Check if the trait object also implements AnotherTrait
    self.underlying.lock().unwrap().handle_event(event)
  }
}

impl StateListener for Component {
  fn handle_state_change(&mut self, key: String, state: &super::state::State) {
      self.underlying.lock().unwrap().handle_state_change(key, state)
  }
}