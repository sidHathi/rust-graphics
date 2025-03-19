use std::{sync::Arc};

use crate::sdf::{CubeSdf, SdfShape, Shape};

use super::super::{collisions::{Collider, SdfBoundary}, component::{AsyncCallbackHandler, Component, ComponentFunctions}, component_store::ComponentKey, errors::EngineError, events::{EventData, EventKey, EventListener}, renderable_model::{ModelDims, RenderableModel}, state::{State, StateListener}, transforms::ModelTransform, utils::random_quaternion, Scene};
use cgmath::{Point3, Quaternion};
use async_trait::async_trait;
use parking_lot::Mutex;
use winit::event::{ElementState, KeyboardInput};

pub struct TestChildComponent {
  key: ComponentKey,
  parent: Option<ComponentKey>,
  local_position: Point3<f32>,
  model: Option<RenderableModel>,
  active: bool,
  should_set_state: bool,
  collider: Option<Arc<Mutex<Collider>>>,
  mem: Option<Arc<Mutex<Self>>>,
  opacity: f32,
  pub should_interp_state: bool,
}

#[async_trait(?Send)]
impl ComponentFunctions for TestChildComponent {
  async fn init(
    &mut self,
    scene: &mut Scene,
    key: ComponentKey,
    parent: Option<ComponentKey>,
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

    let collision_sdf = SdfShape::new(Shape::Cube { center: Point3::new(0., 0., 0.), width: 10., height: 10., depth: 10.}, CubeSdf);
    let collision_boundary = SdfBoundary::new(Point3::new(0., 0., 0.), collision_sdf);
    self.collider = Some(scene.collision_manager.add_component_collider(collision_boundary, key, None));

    let _ = self.add_event_listener(scene, &key, &EventKey::KeyboardEvent);
    let _ = self.add_event_listener(scene, &key, &EventKey::MouseHoverStartEvent(self.key));
    let _ = self.add_event_listener(scene, &key, &EventKey::MouseHoverEndEvent(self.key));
    if let Some(mem_safe) = self.mem.clone() {
      Component::exec_async(mem_safe, Self::wait_then_interpolate, ());
    }
  }

  fn update(&mut self, scene: &mut Scene, _dt: instant::Duration) {
    if self.should_set_state {
      let quaternion = random_quaternion();
      // println!("setting new state: {:?}", quaternion);
      let _ = scene.app_state.set_state("parent_rotation", State::Quaternion(quaternion));
      self.should_set_state = false;
    }

    if self.should_interp_state {
      scene.app_state.interpolate("child_rotation", State::Quaternion(Quaternion::new(1., 0., 0., 0.)), 5.);
      self.should_interp_state = false;
    }
  }

  fn render(&self, scene: &mut Scene) -> Result<(), EngineError> {
    // Ok(())
    if self.model.is_none() {
      // println!("No model to render");
      return Ok(());
    }

    let mut model_transform = ModelTransform::default();
    if let Some(rotation_state) = scene.app_state.get_state("child_rotation") {
      let rotation = rotation_state.get_quat().unwrap();
      model_transform.set_rot(rotation);
      if let Some(collider) = self.collider.clone() {
        collider.lock().update_rot(rotation);
      }
    }
    
    self.model.as_ref().unwrap()
      .transform(model_transform)
      .opacity(self.opacity)
      .dims(ModelDims::new(10., 10., 10.))
      .render(scene)
  }
}

impl TestChildComponent {
  pub fn new() -> Arc<Mutex<Self>> {
    let new_self = Self {
      key: ComponentKey::zero(),
      parent: None,
      model: None,
      local_position: Point3 { x: 0., y: 0., z: 0. },
      active: false,
      should_set_state: false,
      collider: None,
      mem: None,
      opacity: 0.5,
      should_interp_state: false
    };
    let mem = Arc::new(Mutex::new(new_self));
    mem.lock().mem = Some(mem.clone());
    mem
  }

  pub async fn wait_then_interpolate(mem: Arc<Mutex<Self>>, _args: ()) {
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    mem.lock().should_interp_state = true;
  }
}

impl EventListener for TestChildComponent {
  fn handle_event(&mut self, event: super::super::events::Event) {
      match event.data {
        EventData::KeyboardEvent (KeyboardInput {
          virtual_keycode: Some(_key),
          state,
          ..
        }) => {
          if state == ElementState::Pressed {
            self.should_set_state = true;
          }
        },
        EventData::MouseHoverStartEvent { .. } => {
          self.opacity = 1.;
        },
        EventData::MouseHoverEndEvent { .. } => {
          self.opacity = 0.5;
        },
        _ => {}
      }
  }
}

impl AsyncCallbackHandler<()> for TestChildComponent {
  fn handle_async_res(&mut self, _data: ()) {
    println!("callback triggered");
  }
}

impl StateListener for TestChildComponent {}
