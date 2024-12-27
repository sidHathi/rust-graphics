use std::{any::Any, sync::{Arc, Mutex, RwLock}};

use crate::sdf::{CubeSdf, SdfShape, Shape};

use super::{super::{collisions::{Collider, Collision, SdfBoundary}, component::{AsyncCallbackHandler, Component, ComponentFunctions}, component_store::ComponentKey, errors::EngineError, events::{Event, EventData, EventKey, EventListener}, model_renderer::ModelRenderer, renderable_model::{ModelDims, RenderableModel}, scene, state::{State, StateListener}, text::{CGText, TextAlignment, TextWrapStyle, VerticalTextAlignment}, transforms::{ColliderTransform, ComponentTransform, ModelTransform}, util::random_quaternion, Scene}, debug_spheres::DebugSpheres};
use cgmath::{InnerSpace, Point3, Quaternion, Rad, Rotation, Rotation3, Vector3};
use async_trait::async_trait;
use wgpu::Color;
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
  debug_net: Option<Component>,
  collider: Option<Arc<RwLock<Collider>>>,
  active: bool,
  mem: Option<Arc<Mutex<Self>>>,
  rotating: bool,
  jumping: bool,
  opacity: f32
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
      Vector3::new(0., -5., -40.), 
      Quaternion::new(5., 0., 0., 0.)
    );

    let debug_net_underlying = DebugSpheres::new();
    let debug_net = Component::new(debug_net_underlying, scene, Some(self.key)).await;
    self.debug_net = debug_net;

    let collision_sdf = SdfShape::new(Shape::Cube { center: Point3::new(0., 0., 0.), width: 20., height: 20., depth: 20.}, CubeSdf);
    let collision_boundary = SdfBoundary::new(Point3::new(0., 0., 0.), collision_sdf);
    self.collider = Some(scene.collision_manager.add_component_collider(collision_boundary, key, None));
    
    let _ = self.add_event_listener(scene, &key, &EventKey::KeyboardEvent);
    let _ = self.add_event_listener(scene, &key, &EventKey::CollisionStartEvent(self.key.clone()));
    let _ = self.add_event_listener(scene, &key, &EventKey::MouseHoverStartEvent(self.key.clone()));
    let _ = self.add_event_listener(scene, &key, &EventKey::MouseHoverEndEvent(self.key.clone()));
    let _ = self.add_event_listener(scene, &key, &EventKey::MouseHoveringEvent(self.key.clone()));
    let _ = self.add_event_listener(scene, &key, &EventKey::MouseSelectEvent(self.key.clone()));
    let _ = self.add_state_listener(scene, &key, "parent_rotation".into());
    let _ = scene.app_state.set_state("parent_y_offset", State::Float(0.));

    if let Some(mem_safe) = self.mem.clone() {
      Component::exec_async(mem_safe.clone(), Self::set_rotation_after_wait, ());
    }
  }

  fn update(&mut self, scene: &mut Scene, dt: instant::Duration) {
    if self.rotating {
      let axis = Vector3::<f32>::unit_y();
      let angle = Rad(0.01);
      if let Some(model_pos) = self.model_pos.as_mut() {
        model_pos.apply_rot(axis, angle);
        if let Some(collider) = self.collider.clone() {
          collider.write().unwrap().update_rot(Quaternion::from_axis_angle(axis, angle));
        }
      } else {
        self.model_pos = Some(ModelTransform::default());
        self.model_pos.as_mut().unwrap().set_rot(Quaternion::new(1., 0., 0., 0.));
      }
    }

    if self.jumping && !scene.app_state.get_interpolating_keys().contains("parent_y_offset") && scene.app_state.get_state("parent_y_offset").unwrap().get_float().unwrap() < 5. {
      scene.app_state.interpolate("parent_y_offset", State::Float(5.), 1.);
    }
    
    if self.jumping && !scene.app_state.get_interpolating_keys().contains("parent_y_offset") && scene.app_state.contains_key("parent_y_offset") && scene.app_state.get_state("parent_y_offset").unwrap().get_float().unwrap() >= 5. {
      scene.app_state.interpolate("parent_y_offset", State::Float(0.), 1.);
    }

    if scene.app_state.contains_key("parent_y_offset") && self.jumping {
      if let Some(y_offset) = scene.app_state.get_state("parent_y_offset").unwrap().get_float() {
        if self.model_pos.is_none() {
          self.model_pos = Some(ModelTransform::default());
        }
        self.model_pos.as_mut().unwrap().set_pos(Vector3::new(0., y_offset, 0.));
      }
    }
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

    if let Some(model) = self.model.as_ref() {
      let res = model
        .transform(self.model_pos.clone().unwrap_or(ModelTransform::default()))
        .dims(ModelDims::new(20., 20., 20.))
        .opacity(self.opacity)
        .render(scene);

      if let Err(e) = res {
          return Err(e);
      }
    }

    CGText::new("this is text")
      .transform(ModelTransform::local(Vector3 { x: -5., y: 20., z: -10. }, Quaternion::new(0., 0., 0., 0.)))
      .font_size(10.)
      .wrap(TextWrapStyle::Wrap)
      .max_width(20.)
      .align_vertical(VerticalTextAlignment::Top)
      .align(TextAlignment::Left)
      .opacity(0.5)
      .color(Color::RED)
      .render(scene);

    if let Some(child_safe) = self.child.clone() {
      let _ = child_safe.render(scene, Some(self.child_pos.clone()));
    }

    if let Some(debug_net_safe) = self.debug_net.clone() {
      let _ = debug_net_safe.render(scene, Some(self.child_pos.clone()));
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
      },
      EventData::MouseHoverStartEvent { component, collider_idx, intersect_loc } => {
        println!("Handling mouse hover start event!");
        self.opacity = 0.5;
      },
      EventData::MouseHoveringEvent { component, collider_idx, intersect_loc } => {
        // println!("Handling mouse hovering event!");
      },
      EventData::MouseHoverEndEvent { component, collider_idx } => {
        println!("Handling mouse hover end event!");
        self.opacity = 1.;
      },
      EventData::MouseSelectEvent { .. } => {
        println!("Handling mouse hover select event!");
        self.jumping = !self.jumping;
      }
      _ => ()
    }
  }
}

impl StateListener for TestComponent {
  fn handle_state_change(&mut self, key: String, state: &super::super::state::State) {
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
      rotating: false,
      opacity: 1.,
      jumping: false,
      debug_net: None,
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

  pub async fn set_rotation_after_wait(mem: Arc<Mutex<Self>>, args: ()) {
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    mem.lock().unwrap().rotating = true;
  }
}

impl AsyncCallbackHandler<()> for TestComponent {
  fn handle_async_res(&mut self, data: ()) -> () {
    println!("Async callback triggered");
  }
}