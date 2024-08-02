use cgmath::{Angle, EuclideanSpace, InnerSpace, MetricSpace, Vector2, Vector3, Vector4};
use wgpu::SurfaceConfiguration;

use crate::{engine::{collisions::CollisionManager, events::{Event, EventData, EventKey, EventManager}}, graphics::{Camera, Projection}};

use super::raycasting::{RayIntersect, Ray};

pub struct Mouse {
  ray: Option<Ray>,
  max_dist: f32,
  pressed: bool,
  last_intersect: Option<RayIntersect>,
  closest_intersect: Option<RayIntersect>
}

impl Mouse {
  pub fn new(max_raycast_dist: f32) -> Self {
    Self {
      ray: None,
      pressed: false,
      max_dist: max_raycast_dist,
      last_intersect: None,
      closest_intersect: None
    }
  }

  pub fn update_mouse_state(
    &mut self, 
    new_pos: Option<Vector2<f32>>, 
    pressed: bool,
    camera: &Camera,
    proj: &Projection,
    config: &SurfaceConfiguration
  ) {
    if new_pos.is_none() {
      self.ray = None;
      self.pressed = pressed;
      return
    }
    let focal_len = 1. / (proj.get_fovy()/2.).tan();
    let scaled_pos = Vector2::new(2. * new_pos.unwrap().x / config.width as f32, 2. * new_pos.unwrap().y/config.height as f32);
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

    self.ray = Some(Ray::gen_perspective(scaled_pos, eye, u, v, w, focal_len));
    // println!("Updating mouse state with new ray: {:?}", self.ray);
    self.pressed = pressed;
  }

  pub fn intersect_colliders(&mut self, collision_manager: &CollisionManager) {
    if let Some(ray_unwrapped) = self.ray {
      let mut intersections = collision_manager.intersect_ray(&ray_unwrapped, self.max_dist);
      intersections.sort_by(|a, b| b.loc.distance(ray_unwrapped.origin).partial_cmp(&a.loc.distance(ray_unwrapped.origin)).unwrap_or(std::cmp::Ordering::Equal));
      let next_intersect = intersections.pop();
      self.last_intersect = self.closest_intersect.clone();
      self.closest_intersect = next_intersect;
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
        if let Some(last) = self.last_intersect {
          if last.collider_idx != intersect.collider_idx {
            event_manager.handle_event(Event {
              key: EventKey::MouseHoverEndEvent(last.component),
              data: EventData::MouseHoverEndEvent {
                component: last.component.clone(),
                collider_idx: last.collider_idx,
              }
            });
          }
        }
      } else {
        if self.last_intersect.is_none() || intersect.collider_idx != self.last_intersect.unwrap().collider_idx {
          event_manager.handle_event(Event {
            key: EventKey::MouseHoverStartEvent(intersect.component),
            data: EventData::MouseHoverStartEvent {
              component: intersect.component.clone(),
              collider_idx: intersect.collider_idx,
              intersect_loc: intersect.loc.clone()
            }
          });
          if let Some(last) = self.last_intersect {
            event_manager.handle_event(Event {
              key: EventKey::MouseHoverEndEvent(last.component),
              data: EventData::MouseHoverEndEvent {
                component: last.component.clone(),
                collider_idx: last.collider_idx,
              }
            });
          }
        }
        event_manager.handle_event(Event {
          key: EventKey::MouseHoveringEvent(intersect.component),
          data: EventData::MouseHoveringEvent {
            component: intersect.component.clone(),
            collider_idx: intersect.collider_idx,
            intersect_loc: intersect.loc.clone()
          }
        });
      }
    } else {
      if let Some(last) = self.last_intersect {
        event_manager.handle_event(Event {
          key: EventKey::MouseHoverEndEvent(last.component),
          data: EventData::MouseHoverEndEvent {
            component: last.component.clone(),
            collider_idx: last.collider_idx,
          }
        });
      }
    }
  }
}