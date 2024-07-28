use cgmath::Vector3;

use super::State;

pub struct StateInterpolator {
  pub key: String,
  pub start_val: State,
  pub end_val: State,
  pub current_val: State,
  pub time: f64,
  pub time_elapsed: f64
}

pub trait Interpolates {
  fn interpolate(start: Self, end: Self, t: f32) -> Self;
}

impl Interpolates for State {
  fn interpolate(start: Self, end: Self, t: f32) -> Self {
    match (start.clone(), end.clone()) {
      (State::Integer(sv), State::Integer(ev)) => {
        State::Integer((ev - sv) * t as i32)
      },
      (State::Float(sv), State::Float(ev)) => {
        State::Float((ev - sv) * t)
      },
      (State::Quaternion(sv), State::Quaternion(ev)) => {
        State::Quaternion(sv.slerp(ev, t))
      },
      (State::Vector3(sv), State::Vector3(ev)) => {
        State::Vector3(Vector3::new(
          (ev.x - sv.x) * t, 
          (ev.y - sv.y) * t, 
          (ev.z - sv.z) * t
        ))
      },
      _ => {
        if t >= 0.5 {
          start
        } else {
          end
        }
      }
    }
  }
}

impl StateInterpolator {
  pub fn new(key: String, start: State, end: State, time: f64) -> Option<Self> {
    if !start.same_type(&end) { return None }
    Some(Self {
      key,
      start_val: start.clone(),
      current_val: start,
      end_val: end,
      time_elapsed: 0.,
      time,
    })
  }

  pub fn update(&mut self, dt: instant::Duration) {
    self.time_elapsed += dt.as_secs_f64();
    if self.time_elapsed >= self.time {
      self.current_val = self.end_val.clone()
    }
    self.current_val = State::interpolate(self.start_val.clone(), self.end_val.clone(), (self.time_elapsed/self.time) as f32);
  }

  pub fn get_current(&self) -> State {
    self.current_val.clone()
  }

  pub fn complete(&self) -> bool {
    return self.time_elapsed >= self.time
  }
}