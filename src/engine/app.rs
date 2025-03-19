use std::sync::Arc;

use async_trait::async_trait;
use parking_lot::Mutex;

use super::{component::{Component, ComponentFunctions}, component_store::ComponentKey, components::{LightComponent, TestComponent}, errors::EngineError, events::{Event, EventListener}, state::StateListener, Scene};

pub struct App {
  key: ComponentKey,
  pub mem: Option<Arc<Mutex<Self>>>,
  children: Option<Vec<Component>>,
} 

impl App {
  pub fn new() -> Arc<Mutex<Self>> {
   let new_self = Self { key: ComponentKey::zero(), mem: None, children: None };
   let mem = Arc::new(Mutex::new(new_self));
   mem.lock().mem = Some(mem.clone());
   mem
  }
}

#[async_trait(?Send)]
impl ComponentFunctions for App {
  async fn init(&mut self, scene: &mut Scene, key: ComponentKey, _parent: Option<ComponentKey>) {
    self.key = key;
    let mut children = Vec::new();

    /* init children */
    let test_component_underlying = TestComponent::new();
    let test_component_optional = Component::new(test_component_underlying, scene, Some(key)).await;
    if let Some(test_component) = test_component_optional {
      children.push(test_component);
    }
    
    let light_sphere_underlying = LightComponent::new(scene.point_light.clone());
    let light_sphere_optional = Component::new(light_sphere_underlying, scene, Some(key)).await;
    if let Some(light_sphere) = light_sphere_optional {
      children.push(light_sphere);
    }
    /* end init children */

    self.children = Some(children);
  }
  
  fn update(&mut self, scene: &mut Scene, dt:instant::Duration) {
    if let Some(children) = &mut self.children {
      for child in children.iter_mut() {
        child.update(scene, dt);
      }
    }
  }
  
  fn render(&self, scene: &mut Scene) -> Result<(),EngineError>{
    if let Some(children) = &self.children {
      for child in children.iter() {
        let _ = child.render(scene, None);
      }
    }
    Ok(())
  }
}

impl EventListener for App {
  fn handle_event(&mut self, _event: Event) {
  }
}

impl StateListener for App {
  fn handle_state_change(&mut self, _key: String, _state: &super::state::State) {
  }
} 
