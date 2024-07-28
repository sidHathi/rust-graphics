use std::time::{Duration, SystemTime};

use super::Event;

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct ScheduledEventId(pub u32);

pub struct ScheduledEvent {
  pub id: ScheduledEventId,
  pub event: Event,
  pub recurrent: bool,
  pub time_to_trigger: f64,
  pub time_elapsed: f64 
}

impl ScheduledEvent {
  pub fn seconds_from_now(event: Event, seconds: f64, id: ScheduledEventId) -> Self {
    Self {
      id,
      event,
      recurrent: false,
      time_elapsed: 0.,
      time_to_trigger: seconds
    }
  }

  pub fn at_time(event: Event, time: SystemTime, id: ScheduledEventId) -> Option<Self> {
    if let Ok(duration) = time.duration_since(SystemTime::now()) {
      let time_to_trigger = duration.as_secs_f64();
      return Some(Self {
        id,
        event,
        recurrent: false,
        time_to_trigger,
        time_elapsed: 0.
      })
    }
    None
  }

  pub fn recurrent(event: Event, seconds: f64, offset: Option<f64>, id: ScheduledEventId) -> Self {
    Self {
      id,
      event,
      recurrent: true,
      time_to_trigger: seconds,
      time_elapsed: seconds - offset.unwrap_or(seconds)
    }
  }

  pub fn update_time(&mut self, dt: instant::Duration) {
    self.time_elapsed += dt.as_secs_f64();
  }

  pub fn should_trigger(&self) -> bool {
    self.time_elapsed >= self.time_to_trigger
  }

  pub fn reset(&mut self) {
    if self.recurrent {
      self.time_elapsed = 0.;
    }
  }
}