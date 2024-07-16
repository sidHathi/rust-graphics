use cgmath::Quaternion;

use crate::engine::{component_store::ComponentKey, errors::EngineError, Scene};

pub enum State {
  Integer ( i32 ),
  Float ( f32 ),
  Bool ( bool ),
  String ( String ),
  // user can define the rest of the types
  Quaternion (Quaternion<f32>)
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