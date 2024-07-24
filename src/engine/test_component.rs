use std::{any::Any, sync::{Arc, Mutex, RwLock}};

use crate::sdf::{CubeSdf, SdfShape, Shape};

use super::{collisions::{Collider, Collision, SdfBoundary}, component::{Component, ComponentFunctions}, component_store::ComponentKey, errors::EngineError, events::{Event, EventData, EventKey, EventListener}, model_renderer::{ModelRenderer, RenderableModel}, state::{State, StateListener}, transforms::{ColliderTransform, ComponentTransform, ModelTransform}, util::random_quaternion, Scene};
use cgmath::{InnerSpace, Point3, Quaternion, Rotation, Vector3};
use async_trait::async_trait;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};
use super::test_child_component::TestChildComponent;
use rand::Rng;

pub struct TestComponent {
  key: ComponentKey,
  parent: Option<ComponentKey>,
  local_position: Point3<f32>,
  model: Option<RenderableModel>,
  model_pos: Option<ModelTransform>,
  child: Option<Component>,
  child_pos: ComponentTransform,
  collider: Option<Arc<RwLock<Collider>>>,
  active: bool,
  mem: Option<Arc<Mutex<Self>>>
}

#[async_trait(?Send)]
impl ComponentFunctions for TestComponent {
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

    // load a child of same type
    let child_underlying = TestChildComponent::new();
    let child = Component::new(child_underlying, scene, Some(self.key)).await;
    self.child = child;
    self.child_pos = ComponentTransform::local(
      Vector3::new(20., 0., 0.), 
      Quaternion::new(5., 0., 0., 0.)
    );

    let collision_sdf = SdfShape::new(Shape::Cube { center: Point3::new(0., 0., 0.), half_bounds:  Vector3::new(20., 20., 20.)}, CubeSdf);
    let collision_boundary = SdfBoundary::new(Point3::new(0., 0., 0.), collision_sdf);
    self.collider = Some(scene.collision_manager.add_component_collider(collision_boundary, key, None));
    
    let _ = self.add_event_listener(scene, &key, &EventKey::KeyboardEvent);
    let _ = self.add_event_listener(scene, &key, &EventKey::CollisionStartEvent);
    let _ = self.add_state_listener(scene, &key, "parent_rotation".into());
  }

  fn update(&mut self, scene: &mut Scene, dt: instant::Duration) {
    ()
  }

  fn render(&self, scene: &mut Scene) -> Result<(), EngineError> {
    if self.model.is_none() {
      // println!("No model to render");
      return Ok(());
    }
    if let Some(collider) = self.collider.clone() {
      if let Some(transform) = self.model_pos.clone() {
        collider.write().unwrap().update_transform(transform.pos, transform.rot)
      }
    }

    let res: Result<(), EngineError> = scene.render_model(&self.model.as_ref().unwrap(), self.model_pos.clone().unwrap_or(ModelTransform::default()));
    if let Err(e) = res {
        return Err(e);
    }
    if let Some(child_safe) = self.child.clone() {
      return child_safe.render(scene, Some(self.child_pos.clone()));
    }
    Ok(())
  }
}

impl EventListener for TestComponent {
  fn handle_event(&mut self, event: Event) {
    match event.data {
      EventData::KeyboardEvent(KeyboardInput {
        virtual_keycode: Some(key),
        state,
        ..
      }) => {
        if state == ElementState::Pressed {
          // randomize child position in spherical orbit around origin
          let mut radius: f32 = 40.;
          if key == VirtualKeyCode::K {
            radius = 10.
          }
          let quaternion = random_quaternion();
          let dir = quaternion.rotate_vector(Vector3::new(1., 0., 0.)).normalize();
          let new_pos = radius * dir;
          let new_rot = self.child_pos.rot;
          self.child_pos = ComponentTransform::local(new_pos, new_rot);
        }
      },
      EventData::CollisionStartEvent { c1, c2, collision } => {
        if c1 == self.key {
          self.handle_collision(c2, collision);
        } else {
          self.handle_collision(c1, collision);
        }
      }
      _ => ()
    }
  }
}

impl StateListener for TestComponent {
  fn handle_state_change(&mut self, key: String, state: &super::state::State) {
      match key {
        s if s.eq("parent_rotation") => {
          self.handle_new_rotation_state(state);
        },
        _ => {}
      }
  }
}

impl TestComponent {
  pub fn new() -> Arc<Mutex<Self>> {
    let new_self = Self {
      key: ComponentKey::zero(),
      parent: None,
      model: None,
      child: None,
      local_position: Point3 { x: 0., y: 0., z: 0. },
      active: false,
      child_pos: ComponentTransform::default(),
      model_pos: None,
      collider: None,
      mem: None,
    };
    let mem = Arc::new(Mutex::new(new_self));
    mem.lock().unwrap().mem = Some(mem.clone());
    mem
  }

  pub fn handle_new_rotation_state(&mut self, new_state: &State) {
    match new_state {
      State::Quaternion(q) => {
        println!("handling new state: {:?}", q);
        let old_pos = self.model_pos.clone().unwrap_or(ModelTransform::default()).get_pos();
        self.model_pos = Some(ModelTransform::local(old_pos, q.clone()));
      },
      _ => {}
    }
  }

  pub fn handle_collision(&mut self, component: ComponentKey, collision: Collision) {
    println!("Collision event with component {:?} detected and handled!", component);
    return
  }
}