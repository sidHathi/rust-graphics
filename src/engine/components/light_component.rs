use std::sync::Arc;

use async_trait::async_trait;
use cgmath::{EuclideanSpace, Point3, Quaternion, Vector3};
use parking_lot::Mutex;

use crate::engine::{component::ComponentFunctions, component_store::ComponentKey, debug::DebugLine, errors::EngineError, events::{Event, EventListener}, renderable_model::{ModelDims, RenderableModel}, state::{State, StateListener}, transforms::ModelTransform, utils::Positioned, Scene };
pub struct LightComponent {
  key: ComponentKey,
  parent: Option<ComponentKey>,
  active: bool,
  mem: Option<Arc<Mutex<Self>>>,
  position_ref: Option<Arc<dyn Positioned>>,
  model: Option<RenderableModel>,
}

#[async_trait(?Send)]
impl ComponentFunctions for LightComponent {
  async fn init(&mut self, scene: &mut Scene, key: ComponentKey, parent: Option<ComponentKey>) {
    self.key = key;
    self.parent = parent;
    self.active = true;

    if let Ok(model) = scene.load_model("sphere.obj", None, key).await {
      self.model = Some(model);
    } else {
      self.model = None;
    }
  }

  fn update(&mut self, _scene: &mut Scene, _dt: instant::Duration) {
    // Light component update logic can be implemented here
  }

  fn render(&self, scene: &mut Scene) -> Result<(), EngineError> {
    if let Some(position_ref) = self.position_ref.clone() {
      let position = position_ref.get_position();
      if let Some(position) = position {
        if let Some(model) = &self.model {
          return model
            .transform(ModelTransform::local(position.to_vec(), Quaternion::new(0., 0., 0., 0.)))
            .dims(ModelDims::new(5., 5., 5.))
            .render(scene);
        }
      }
    }
    Ok(())
  }
}

impl EventListener for LightComponent {
  fn handle_event(&mut self, _event: Event) {
    // Light component event handling logic can be implemented here
  }
}

impl StateListener for LightComponent {
  fn handle_state_change(&mut self, _key: String, _state: &State) {
    // Light component state change handling logic can be implemented here
  }
}

impl LightComponent {
  pub fn new(position_ref: Arc<dyn Positioned>) -> Arc<Mutex<Self>> {
    let new_self = Self {
      key: ComponentKey::zero(),
      parent: None,
      active: false,
      mem: None,
      position_ref: Some(position_ref),
      model: None,
    };
    let mem = Arc::new(Mutex::new(new_self));
    mem.lock().mem = Some(mem.clone());
    mem
  }
}