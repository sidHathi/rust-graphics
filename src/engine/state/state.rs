use std::clone;

use cgmath::{Quaternion, Vector3};

use crate::engine::{component_store::ComponentKey, errors::EngineError, Scene};

#[derive(Clone, Debug)]
pub enum State {
  Integer ( i32 ),
  Float ( f32 ),
  Bool ( bool ),
  String ( String ),
  // user can define the rest of the types
  Quaternion (Quaternion<f32>),
  Vector3 (Vector3<f32>)
}

impl State {
  pub fn same_type(&self, other: &Self) -> bool {
    matches!((self, other),
      (State::Integer(_), State::Integer(_)) |
      (State::Float(_), State::Float(_)) |
      (State::Bool(_), State::Bool(_)) |
      (State::String(_), State::String(_)) |
      (State::Quaternion(_), State::Quaternion(_)) |
      (State::Vector3(_), State::Vector3(_)))
  }

  pub fn get_int(&self) -> Option<i32> {
    match self {
      &State::Integer(val) => Some(val),
      &State::Float(val) => Some(val as i32),
      &State::Bool(val) => Some(val as i32),
      _ => None
    }
  }

  pub fn get_float(&self) -> Option<f32> {
    match self {
      &State::Float(val) => Some(val),
      &State::Integer(val) => Some(val as f32),
      _ => None
    }
  }

  pub fn get_bool(&self) -> Option<bool> {
    match self {
      &State::Bool(val) => Some(val),
      &State::Integer(val) => Some(val != 0),
      _ => None
    }
  }

  pub fn get_string(&self) -> Option<String> {
    match self {
      State::String(val) => Some(val.clone()),
      State::Float(val) => Some(format!("{:?}", val)),
      State::Integer(val) => Some(format!("{:?}", val)),
      State::Bool(val) => Some(format!("{:?}", val)),
      State::Quaternion(val) => Some(format!("{:?}", val)),
      State::Vector3(val) => Some(format!("{:?}", val)),
      _ => None
    }
  }

  pub fn get_quat(&self) -> Option<Quaternion<f32>> {
    match self {
      State::Quaternion(val) => Some(val.clone()),
      _ => None
    }
  }

  pub fn get_vec3(&self) -> Option<Vector3<f32>> {
    match self {
      State::Vector3(val) => Some(val.clone()),
      _ => None
    }
  }
}

pub trait StateListener {
  fn handle_state_change(&mut self, key: String, state: &State) {
    println!("Warning: Component listens for state change without handler");
    ()
  }

  fn add_state_listener(&mut self, scene: &mut Scene, component_key: &ComponentKey, state_key: String) -> Result<(), EngineError> {
    let listener: fn(&mut dyn StateListener, key: String, state: &State) = |component: &mut dyn StateListener, key: String, state: &State| {
      component.handle_state_change(key, state);
    };
    scene.app_state.listen(component_key.clone(), state_key, listener)
  }
}