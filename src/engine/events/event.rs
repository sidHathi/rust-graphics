use cgmath::{Point3, Vector3};
use winit::event::{KeyboardInput, WindowEvent};

use crate::engine::{collisions::Collision, component_store::ComponentKey, errors::EngineError, Scene};

#[derive(Clone)]
pub struct Event {
  pub key: EventKey,
  pub data: EventData
}

#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub enum EventKey {
  KeyboardEvent,
  MouseHoverStartEvent(ComponentKey),
  MouseHoverEndEvent(ComponentKey),
  MouseHoveringEvent(ComponentKey),
  MouseSelectEvent(ComponentKey),
  CollisionOngoingEvent(ComponentKey),
  CollisionStartEvent(ComponentKey),
  CollisionEndEvent(ComponentKey),
  RaycastIntersectEvent(ComponentKey),
}

#[derive(Clone)]
pub enum EventData {
  KeyboardEvent (KeyboardInput),
  MouseHoverStartEvent {
    component: ComponentKey,
    collider_idx: u32,
    intersect_loc: Point3<f32>
  },
  MouseHoveringEvent {
    component: ComponentKey,
    collider_idx: u32,
    intersect_loc: Point3<f32>
  },
  MouseHoverEndEvent {
    component: ComponentKey,
    collider_idx: u32,
  },
  MouseSelectEvent {
    component: ComponentKey,
    collider_idx: u32,
    intersect_loc: Point3<f32>
  },
  CollisionOngoingEvent {
    c1: ComponentKey,
    c2: ComponentKey,
    collision: Collision
  },
  CollisionStartEvent {
    c1: ComponentKey,
    c2: ComponentKey,
    collision: Collision
  },
  CollisionEndEvent {
    c1: ComponentKey,
    c2: ComponentKey,
    collider_keys: (u32, u32)
  },
  RaycastIntersectEvent {
    component: ComponentKey,
    collider_idx: u32,
    intersect_loc: Point3<f32>
  },
}

impl Event {
  pub fn from(event: &WindowEvent) -> Option<Self> {
    match event {
      WindowEvent::KeyboardInput {
        input,
        ..
      } => Some(Event {
        key: EventKey::KeyboardEvent,
        data: EventData::KeyboardEvent(input.clone())
      }),
      _ => None
    }
  } 
}

pub trait EventListener {
  fn handle_event(&mut self, _: Event) {
    ()
  }

  fn add_event_listener(&mut self, scene: &mut Scene, component_key: &ComponentKey, event_key: &EventKey) -> Result<(), EngineError> {
    let listener: fn(&mut dyn EventListener, Event) = |component: &mut dyn EventListener, event: Event| {
      component.handle_event(event);
    };
    scene.event_manager.add_listener(component_key.clone(), event_key.clone(), listener)
  }
}
