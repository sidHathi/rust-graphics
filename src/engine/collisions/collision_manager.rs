use std::{borrow::Borrow, collections::{HashMap, HashSet}, hash::Hash, ops::Index, sync::{Arc, Mutex, RwLock}};

use cgmath::Matrix4;

use crate::engine::{component::Component, component_store::ComponentKey, events::{Event, EventData, EventKey, EventManager}, raycasting::{Ray, RayIntersect, Raycast}, transform_queue::{apply_quaternion_transform, to_point, to_vec}, transforms::{ColliderTransform, ComponentTransform}, Scene};

use super::collider::{Collider, ColliderBoundary, Collision};
use cgmath::Transform;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct IndexPair(u32, u32);

impl Hash for IndexPair {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    let IndexPair(x, y) = *self;
    let (min, max) = if x < y { (x, y) } else { (y, x) };
    min.hash(state);
    max.hash(state);
  }
}

pub struct CollisionManager {
  index_collider_map: HashMap<u32, Arc<RwLock<Collider>>>,
  comp_collider_map: HashMap<ComponentKey, Vec<Arc<RwLock<Collider>>>>,
  index_comp_map: HashMap<u32, ComponentKey>,
  colliding_pairs: HashSet<IndexPair>,
  collisions: Vec<Collision>,
  next_key: u32,
}

impl CollisionManager {
  pub fn new() -> CollisionManager {
    Self {
      index_collider_map: HashMap::new(),
      comp_collider_map: HashMap::new(),
      index_comp_map: HashMap::new(),
      colliding_pairs: HashSet::new(),
      collisions: Vec::new(),
      next_key: 0
    }
  }

  pub fn add_component_collider(
    &mut self, 
    boundary: impl ColliderBoundary + 'static, 
    parent: ComponentKey,
    transform: Option<ColliderTransform>
  ) -> Arc<RwLock<Collider>> {
    let collider_idx = self.next_key;
    self.next_key += 1;

    let collider = Collider::new(collider_idx, boundary, parent.clone(), transform);
    let collider_rc = Arc::new(RwLock::new(collider));
    if !self.comp_collider_map.contains_key(&parent) {
      self.comp_collider_map.insert(parent.clone(), Vec::new());
    }
    self.comp_collider_map.get_mut(&parent).unwrap().push(collider_rc.clone());
    self.index_collider_map.insert(collider_idx, collider_rc.clone());
    self.index_comp_map.insert(collider_idx, parent.clone());
    collider_rc
  }

  pub fn remove_component_colliders(&mut self, comp: ComponentKey) -> Option<Vec<Arc<RwLock<Collider>>>> {
    if let Some(colliders) = self.comp_collider_map.remove(&comp) {
      for col in colliders.iter() {
        let idx = col.read().unwrap().index;
        self.index_collider_map.remove(&idx);
        self.index_comp_map.remove(&idx);
      }
      return Some(colliders)
    }
    None
  }

  pub fn update_collider_positions(&mut self, position_cache: &HashMap<ComponentKey, Matrix4<f32>>) {
    for (key, colliders) in self.comp_collider_map.iter_mut() {
      if position_cache.contains_key(key) {
        let mat = position_cache.get(key).unwrap();
        for collider in colliders {
          let mut mutex_guard = collider.write().unwrap();
          let curr_transform = mutex_guard.transform.clone();
          let new_pos = to_vec(mat.transform_point(to_point(curr_transform.relative_pos)));
          let new_rot = apply_quaternion_transform(mat, curr_transform.relative_rot);
          mutex_guard.transform.cache_global_pos(new_pos);
          mutex_guard.transform.cache_global_rot(new_rot);
        }
      }
    }
  }

  pub fn trigger_collision_events(&mut self, event_manager: &mut EventManager) {
    let mut collisions: HashMap<IndexPair, Collision> = HashMap::new();
    for (key_i, collider_i) in self.index_collider_map.iter() {
      for (key_j, collider_j) in self.index_collider_map.iter() {
        if key_i == key_j {
          continue;
        }

        let pot_collision = collider_i.read().unwrap().collide(&collider_j.read().unwrap());
        let index_pair = IndexPair(key_i.clone(), key_j.clone());
        if let Some(collision) = pot_collision {
          if !collisions.contains_key(&index_pair) {
            collisions.insert(index_pair, collision);
            // println!("Collision detected: {:?} -> comp 1: {:?}, comp2: {:?}", collision.clone(), self.index_collider_map.get(&collision.colliders.0).unwrap().read().unwrap().parent, self.index_collider_map.get(&collision.colliders.1).unwrap().read().unwrap().parent);
          }
        }
      }
    }

    // for each collision -> want to trigger an event for each pair of colliders that are intersecting with the detected collision
    // this event is registered for each pair of components involved in the collision -> this means we need to know which collider index corresponds with which component on registration
    // want to know which collisions are already ongoing, and which ongoing collisions are no longer happening
    let mut new_colliding_pairs: HashSet<IndexPair> = HashSet::new();
    for (index_pair, collision) in collisions {
      if let Some(c1) = self.index_comp_map.get(&index_pair.0) {
        if let Some(c2) = self.index_comp_map.get(&index_pair.1) {
          if c1 == c2 {
            continue;
          }

          let co_event_data = EventData::CollisionOngoingEvent { 
            c1: c1.clone(), 
            c2: c2.clone(), 
            collision: collision.clone()
          };
          event_manager.handle_event(Event {
            key: EventKey::CollisionOngoingEvent(c1.clone()),
            data: co_event_data.clone()
          });
          event_manager.handle_event(Event {
            key: EventKey::CollisionOngoingEvent(c2.clone()),
            data: co_event_data
          });

          new_colliding_pairs.insert(index_pair.clone());
          if !self.colliding_pairs.contains(&index_pair) {
            let cs_event_data = EventData::CollisionStartEvent { 
              c1: c1.clone(), 
              c2: c2.clone(), 
              collision: collision.clone()
            };
            event_manager.handle_event(Event {
              key: EventKey::CollisionStartEvent(c1.clone()),
              data: cs_event_data.clone()
            });
            event_manager.handle_event(Event {
              key: EventKey::CollisionStartEvent(c2.clone()),
              data: cs_event_data
            });
          }
        }
      }
    }

    for index_pair in self.colliding_pairs.iter() {
      if !new_colliding_pairs.contains(&index_pair) {
        if !self.index_comp_map.contains_key(&index_pair.0) || !self.index_comp_map.contains_key(&index_pair.1) {
          continue;
        }

        let c1 = self.index_comp_map.get(&index_pair.0).unwrap().clone();
        let c2 = self.index_comp_map.get(&index_pair.1).unwrap().clone();
        let collider_keys = (index_pair.0, index_pair.1);
        let ce_event_data = EventData::CollisionEndEvent { c1, c2, collider_keys };
        event_manager.handle_event(Event {
          key: EventKey::CollisionEndEvent(c1),
          data: ce_event_data.clone()
        });
        event_manager.handle_event(Event {
          key: EventKey::CollisionEndEvent(c2),
          data: ce_event_data
        });
      }
    }

    self.colliding_pairs = new_colliding_pairs;
  }

  pub fn intersect_raycasts(&self, raycasts: Vec<&mut Raycast>) {
    // for each ray
    // figure out if intersects any of the colliders -> that's basicaly it
    for raycast in raycasts {
      raycast.intersections.clear();
      for collider in self.index_collider_map.values() {
        if let Some(collision_loc) = collider.read().unwrap().intersects_ray(&raycast.ray, raycast.max_dist) {
          raycast.intersections.push(RayIntersect {
            component: collider.read().unwrap().parent,
            loc: collision_loc,
            collider_idx: collider.read().unwrap().index
          });
        }
      }
    }
  }

  pub fn intersect_ray(&self, ray: &Ray, max_dist: f32) -> Vec<RayIntersect> {
    // for each ray
    // figure out if intersects any of the colliders -> that's basicaly it
    let mut intersections: Vec<RayIntersect> = Vec::new();
    for collider in self.index_collider_map.values() {
      if let Some(collision_loc) = collider.read().unwrap().intersects_ray(ray, max_dist) {
        intersections.push(RayIntersect {
          component: collider.read().unwrap().parent,
          loc: collision_loc,
          collider_idx: collider.read().unwrap().index
        });
      }
    }

    intersections
  }
}


pub fn try_collide(col1: &Arc<RwLock<Collider>>, col2: &Arc<RwLock<Collider>>) -> Option<Collision> {
  let col1_unwrapped = col1.read().unwrap();
  let col2_unwrapped = col2.read().unwrap();

  if let Some(collision) = col1_unwrapped.collide(&col2_unwrapped) {
    return Some(collision)
  }
  None
}