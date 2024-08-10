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