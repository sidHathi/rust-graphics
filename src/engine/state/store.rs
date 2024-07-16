use std::{collections::{HashMap, HashSet}, os::macos::raw::stat};

use crate::engine::{component::Component, component_store::{ComponentKey, ComponentStore}, errors::EngineError, Scene};

use super::state::{State, StateListener};

pub struct Store {
  state_map: HashMap<String, State>,
  state_listeners: HashMap<ComponentKey, HashMap<String, fn(&mut dyn StateListener, String, &State) -> ()>>,
  triggered_functions: HashMap<ComponentKey, Vec<(String, fn(&mut dyn StateListener, String, &State) -> ())>>,
}

impl Store {
  pub fn create(app_state: Vec<(String, State)>) -> Self {
    let state_map = app_state.into_iter().collect();
    Self {
      state_map,
      state_listeners: HashMap::new(),
      triggered_functions: HashMap::new(),
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

  pub fn listen(&mut self, component_key: ComponentKey, state_key: String, callback: fn(&mut dyn StateListener, String, &State) -> ()) -> Result<(), EngineError> {
    if !self.state_map.contains_key(&state_key) {
      return Err(EngineError::ArgumentError { index: 2, name: "state_key".into() })
    }
    if !self.state_listeners.contains_key(&component_key) {
      self.state_listeners.insert(component_key.clone(), HashMap::new());
    }
    let listener_map = self.state_listeners.get_mut(&component_key).unwrap();
    let _ = listener_map.insert(state_key.clone(), callback);
    return Ok(())
  }


  pub fn trigger_callbacks(&mut self, components: &mut ComponentStore) -> Result<(), EngineError> {
    for (key, callback_tuples) in self.triggered_functions.iter() {
      let component: &mut dyn StateListener = components.get_mut(key).unwrap();
      let mut used_keys: HashSet<String> = HashSet::new();
      for (state_key, cb) in callback_tuples {
        if used_keys.contains(state_key) {
          continue;
        }
        used_keys.insert(state_key.clone());
        let val_opt = self.state_map.get(state_key);
        if let Some(val) = val_opt {
          (*cb)(component, state_key.clone(), val);
        } else {
          return Err(EngineError::StateAccessError { state_key: state_key.clone() });
        }
      }
    }
    self.triggered_functions.clear();

    Ok(())
  }

  pub fn handle_state_change(&mut self, state_key: String) {
    for (comp, cb_map) in self.state_listeners.iter_mut() {
      if cb_map.contains_key(&state_key) {
        let func_opt = cb_map.get(&state_key);
        if let Some(func) = func_opt {
          if !self.triggered_functions.contains_key(comp) {
            self.triggered_functions.insert(comp.clone(), Vec::new());
          }
          self.triggered_functions.get_mut(comp).unwrap().push((state_key.clone(), func.clone()))
        }
      }
    }
  }
}
