use std::collections::HashMap;

use cgmath::Point3;

use crate::engine::{collisions::{Collider, CollisionManager}, component_store::ComponentKey, events::{Event, EventData, EventKey, EventManager}};

use super::{ray, Ray};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct RayIntersect {
  pub component: ComponentKey,
  pub collider_idx: u32,
  pub loc: Point3<f32>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Raycast {
  pub id: u32,
  pub ray: Ray,
  pub max_dist: f32,
  pub parent: ComponentKey,
  pub intersections: Vec<RayIntersect>
}


pub struct RaycastManager {
  next_raycast_idx: u32,
  pub raycasts: HashMap<u32, Raycast>
}

impl RaycastManager {
  // new
  pub fn new() -> Self {
    Self {
      next_raycast_idx: 1,
      raycasts: HashMap::new()
    }
  }

  // add_raycast
  pub fn new_raycast(
    &mut self, 
    ray: Ray, 
    max_dist: f32,
    parent: ComponentKey
  ) -> Option<Raycast> {
    if self.next_raycast_idx == u32::MAX {
      return None
    }

    let raycast = Raycast {
      id: self.next_raycast_idx,
      ray,
      max_dist,
      parent,
      intersections: Vec::new()
    };
    self.raycasts.insert(self.next_raycast_idx, raycast.clone());
    self.next_raycast_idx += 1;

    Some(raycast)
  }

  // remove_raycast
  pub fn remove_raycast(
    &mut self,
    raycast_id: &u32
  ) -> Option<Raycast> {
    self.raycasts.remove(raycast_id)
  }

  // update_intersections
  pub fn intersect_colliders(&mut self, collision_manager: &CollisionManager) {
    collision_manager.intersect_raycasts(self.raycasts.values_mut().collect());
  }

  // trigger_events
  pub fn trigger_raycast_events(&self, event_manager: &mut EventManager) {
    for raycast in self.raycasts.values() {
      for intersect in raycast.intersections.iter() {
        event_manager.handle_event(Event {
          key: EventKey::RaycastIntersectEvent(intersect.component.clone()),
          data: EventData::RaycastIntersectEvent {
            component: intersect.component.clone(), 
            intersect_loc: intersect.loc.clone() ,
            collider_idx: intersect.collider_idx
          }
        });
      }
    }
  }
}
