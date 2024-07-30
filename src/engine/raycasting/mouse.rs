use cgmath::{EuclideanSpace, InnerSpace, MetricSpace, Vector2, Vector3};

use crate::{engine::{collisions::CollisionManager, events::{Event, EventData, EventKey, EventManager}}, graphics::{Camera, Projection}};

use super::{raycast_manager::RayIntersect, Ray};

pub struct Mouse {
  ray: Option<Ray>,
  max_dist: f32,
  pressed: bool,
  closest_intersect: Option<RayIntersect>
}

impl Mouse {
  pub fn new(max_raycast_dist: f32) -> Self {
    Self {
      ray: None,
      pressed: false,
      max_dist: max_raycast_dist,
      closest_intersect: None
    }
  }

  pub fn update_mouse_state(
    &mut self, 
    new_pos: Vector2<f32>, 
    pressed: bool,
    camera: &Camera,
  ) {
    let (sin_pitch, cos_pitch) = camera.pitch.0.sin_cos();
    let (sin_yaw, cos_yaw) = camera.yaw.0.sin_cos();
    let eye = camera.position.to_vec();
    let dir = Vector3::new(
        cos_pitch * cos_yaw,
        sin_pitch,
        cos_pitch * sin_yaw
    ).normalize();
    let up: Vector3<f32> = Vector3::unit_y();
    let u = up.cross(dir).normalize();
    let w = (-1. * dir).normalize();
    let v = up.normalize();

    self.ray = Some(Ray::gen_ortho(new_pos, eye, u, v, w));
    self.pressed = pressed;
  }

  pub fn intersect_colliders(&mut self, collision_manager: &CollisionManager) {
    if let Some(ray_unwrapped) = self.ray {
      let mut intersections = collision_manager.intersect_ray(&ray_unwrapped, self.max_dist);
      intersections.sort_by(|a, b| b.loc.distance(ray_unwrapped.origin).partial_cmp(&a.loc.distance(ray_unwrapped.origin)).unwrap_or(std::cmp::Ordering::Equal));
      self.closest_intersect = intersections.pop();
    }
  }

  pub fn trigger_mouse_events(&self, event_manager: &mut EventManager) {
    if let Some(intersect) = self.closest_intersect {
      if self.pressed {
        event_manager.handle_event(Event {
          key: EventKey::MouseSelectEvent(intersect.component),
          data: EventData::MouseSelectEvent {
            component: intersect.component.clone(),
            collider_idx: intersect.collider_idx,
            intersect_loc: intersect.loc.clone()
          }
        });
      } else {
        event_manager.handle_event(Event {
          key: EventKey::MouseHoverEvent(intersect.component),
          data: EventData::MouseHoverEvent {
            component: intersect.component.clone(),
            collider_idx: intersect.collider_idx,
            intersect_loc: intersect.loc.clone()
          }
        });
      }
    }
  }
}