use std::sync::{Arc, Mutex, RwLock};

use crate::sdf::{CubeSdf, SdfShape, Shape};

use super::{collisions::{Collider, SdfBoundary}, component::{Component, ComponentFunctions}, component_store::ComponentKey, errors::EngineError, events::{EventData, EventKey, EventListener}, model_renderer::{ModelRenderer, RenderableModel}, state::{State, StateListener}, transforms::ModelTransform, util::random_quaternion, Scene};
use cgmath::{Point3, Quaternion, Vector3};
use async_trait::async_trait;
use winit::event::{ElementState, KeyboardInput};

pub struct TestChildComponent {
  key: ComponentKey,
  parent: Option<ComponentKey>,
  local_position: Point3<f32>,
  model: Option<RenderableModel>,
  active: bool,
  should_set_state: bool,
  collider: Option<Arc<RwLock<Collider>>>,
}

#[async_trait(?Send)]
impl ComponentFunctions for TestChildComponent {
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

    let collision_sdf = SdfShape::new(Shape::Cube { center: Point3::new(0., 0., 0.), half_bounds:  Vector3::new(20., 20., 20.)}, CubeSdf);
    let collision_boundary = SdfBoundary::new(Point3::new(0., 0., 0.), collision_sdf);
    self.collider = Some(scene.collision_manager.add_component_collider(collision_boundary, key, None));

    let _ = self.add_event_listener(scene, &key, &EventKey::KeyboardEvent);
  }

  fn update(&mut self, scene: &mut Scene, dt: instant::Duration) {
    if self.should_set_state {
      let quaternion = random_quaternion();
      println!("setting new state: {:?}", quaternion);
      let _ = scene.app_state.set_state("parent_rotation".into(), State::Quaternion(quaternion));
      self.should_set_state = false;
    }
  }

  fn render(&self, scene: &mut Scene) -> Result<(), EngineError> {
    // Ok(())
    if self.model.is_none() {
      // println!("No model to render");
      return Ok(());
    }

    scene.render_model(&self.model.as_ref().unwrap(), ModelTransform::default())
  }
}

impl TestChildComponent {
  pub fn new() -> TestChildComponent {
    Self {
      key: ComponentKey::zero(),
      parent: None,
      model: None,
      local_position: Point3 { x: 0., y: 0., z: 0. },
      active: false,
      should_set_state: false,
      collider: None,
    }
  }
}

impl EventListener for TestChildComponent {
  fn handle_event(&mut self, event: super::events::Event) {
      match event.data {
        EventData::KeyboardEvent (KeyboardInput {
          virtual_keycode: Some(key),
          state,
          ..
        }) => {
          if state == ElementState::Pressed {
            self.should_set_state = true;
          }
        },
        _ => {}
      }
  }
}

impl StateListener for TestChildComponent {}
