use cgmath::{Point3, Vector3};
use winit::event::KeyboardInput;

use crate::engine::component_store::ComponentKey;

use super::component_event::ComponentEvent;

pub struct Event {
  pub key: EventKey,
  pub data: EventData
}

#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub enum EventKey {
  KeyboardEvent,
  MouseRaycastEvent,
  ComponentEvent,
  CollisionEvent,
  CustomEvent,
}

pub enum EventData {
  KeyboardEvent (KeyboardInput),
  MouseRaycastEvent {
    origin: Point3<f32>,
    dir: Vector3<f32>
  },
  ComponentEvent (ComponentEvent),
  CollisionEvent {
    c1: ComponentKey,
    c2: ComponentKey,
    loc: Point3<f32>
  },
  CustomEvent (String)
}
