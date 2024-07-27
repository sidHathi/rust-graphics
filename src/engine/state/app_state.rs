use crate::engine::util::random_quaternion;

use super::{State, Store};

pub fn create_app_state() -> Store {
  let fields: Vec<(String, State)> = Vec::from([
    ("parent_rotation".into(), State::Quaternion(random_quaternion())),
    ("child_rotation".into(), State::Quaternion(random_quaternion())),
  ]);
  Store::create(fields)
}