use std::{collections::{HashMap, HashSet}, os::macos::raw::stat};

use crate::engine::{component::Component, component_store::ComponentKey, errors::EngineError, Scene};

use super::state::State;

pub struct Store {
  state_map: HashMap<String, State>,
  single_state_listeners: HashMap<ComponentKey, HashMap<String, fn(&mut Component, &State) -> ()>>,
  multi_state_listeners: HashMap<ComponentKey, HashMap<String, fn(&mut Component, &HashMap<String, State>) -> ()>>,
  triggered_single_functions: HashMap<ComponentKey, Vec<(String, fn(&mut Component, &State) -> ())>>,
  triggered_multi_functions: HashMap<ComponentKey, Vec<fn(&mut Component, &HashMap<String, State>) -> ()>>
}

impl Store {
  pub fn create(app_state: Vec<(String, State)>) -> Self {
    let state_map = app_state.into_iter().collect();
    Self {
      state_map,
      single_state_listeners: HashMap::new(),
      multi_state_listeners: HashMap::new(),
      triggered_multi_functions: HashMap::new(),
      triggered_single_functions: HashMap::new()
    }
  }

  pub fn add_state_value(&mut self, key: String, state: State) -> Option<State> {
    self.state_map.insert(key, state)
  }

  pub fn remove_state_key(&mut self, key: &String) -> Option<State> {
    self.state_map.remove(key)
  }

  pub fn set_state(&mut self, key: String, val: State) -> Result<State, EngineError> {
    if !self.state_map.contains_key(&key) {
      return Err(EngineError::ArgumentError { index: 1, name: "key".into() })
    }
    if let Some(inserted) = self.state_map.insert(key.clone(), val) {
      self.handle_state_change(key.clone());
      return Ok(inserted)
    }
    Err(EngineError::Custom("State set failed for unknown reason".into()))
  }

  pub fn get_state(&self, key: &String) -> Option<&State> {
    self.state_map.get(key)
  }

  pub fn listen(&mut self, component_key: ComponentKey, state_key: String, callback: fn(&mut Component, &State) -> ()) -> Result<(), EngineError> {
    if !self.state_map.contains_key(&state_key) {
      return Err(EngineError::ArgumentError { index: 2, name: "state_key".into() })
    }
    if !self.single_state_listeners.contains_key(&component_key) {
      self.single_state_listeners.insert(component_key.clone(), HashMap::new());
    }
    let mut listener_map = self.single_state_listeners.remove(&component_key).unwrap();
    if let Some(_) = listener_map.insert(state_key, callback) {
      return Ok(())
    }
    self.single_state_listeners.insert(component_key, listener_map);
    Err(EngineError::Custom("Listener set failed for unknown reason".into()))
  }


  pub fn listen_vec(&mut self, component_key: ComponentKey, state_keys: Vec<String>, callback: fn(&mut Component, &State) -> ()) -> Result<(), EngineError> {
    for state_key in &state_keys {
      if !self.state_map.contains_key(state_key) {
        return Err(EngineError::ArgumentError { index: 2, name: "state_key".into() })
      }
    }

    if !self.multi_state_listeners.contains_key(&component_key) {
      self.single_state_listeners.insert(component_key.clone(), HashMap::new());
    }
    let mut listener_map = self.single_state_listeners.remove(&component_key).unwrap();
    
    let mut res = Ok(());
    for state_key in state_keys {
      if listener_map.insert(state_key, callback).is_none() {
        res = Err(EngineError::Custom("Listener set failed for unknown reason".into()))
      }
    }
    self.single_state_listeners.insert(component_key, listener_map);
    return res
  }

  pub fn trigger_callbacks(&mut self, scene: &mut Scene) -> Result<(), EngineError> {
    for (key, callback_tuples) in self.triggered_single_functions.iter() {
      let mut component = scene.components.remove(key).unwrap();
      let mut used_keys: HashSet<String> = HashSet::new();
      for (state_key, cb) in callback_tuples {
        if used_keys.contains(state_key) {
          continue;
        }
        used_keys.insert(state_key.clone());
        let val_opt = self.state_map.get(state_key);
        if let Some(val) = val_opt {
          (*cb)(&mut component, val);
        } else {
          return Err(EngineError::StateAccessError { state_key: state_key.clone() });
        }
      }
      let _ = scene.components.insert(component);
    }
    self.triggered_single_functions.clear();


    for (key, callback_tuples) in self.triggered_multi_functions.iter() {
      let mut component = scene.components.remove(key).unwrap();
      for cb in callback_tuples {
        (*cb)(&mut component, &self.state_map);
      }
      let _ = scene.components.insert(component);
    }
    self.triggered_multi_functions.clear();
    Ok(())
  }

  pub fn handle_state_change(&mut self, state_key: String) {
    for (comp, cb_map) in self.single_state_listeners.iter_mut() {
      if cb_map.contains_key(&state_key) {
        let func_opt = cb_map.remove(&state_key);
        if let Some(func) = func_opt {
          if !self.triggered_single_functions.contains_key(comp) {
            self.triggered_single_functions.insert(comp.clone(), Vec::new());
          }
          self.triggered_single_functions.get_mut(comp).unwrap().push((state_key.clone(), func))
        }
      }
    }

    for (comp, cb_map) in self.multi_state_listeners.iter_mut() {
      if cb_map.contains_key(&state_key) {
        let func_opt = cb_map.remove(&state_key);
        if let Some(func) = func_opt {
          if !self.triggered_multi_functions.contains_key(comp) {
            self.triggered_multi_functions.insert(comp.clone(), Vec::new());
          }
          self.triggered_multi_functions.get_mut(comp).unwrap().push(func);
        }
      }
    }
  }
}