use std::collections::{HashMap, HashSet};

use crate::engine::{component::{self, Component, ComponentFunctions}, component_store::{ComponentKey, ComponentStore}, errors::EngineError, Scene};

use super::event::{Event, EventKey, EventListener};

pub struct EventManager {
  new_events: HashMap<EventKey, Vec<Event>>,
  event_listeners: HashMap<ComponentKey, HashMap<EventKey, fn(&mut dyn EventListener, Event) -> ()>>,
  triggered_events: HashMap<ComponentKey, Vec<(EventKey, fn(&mut dyn EventListener, Event) -> ())>>
}

impl EventManager {
  pub fn new() -> EventManager {
    Self {
      new_events: HashMap::new(),
      event_listeners: HashMap::new(),
      triggered_events: HashMap::new()
    }
  }

  pub fn handle_event(&mut self, event: Event) -> bool {
    for (comp, map) in self.event_listeners.iter() {
      if map.contains_key(&event.key) {
        if !self.triggered_events.contains_key(comp) {
          self.triggered_events.insert(comp.clone(), Vec::new());
        }
        let trigger_vec = self.triggered_events.get_mut(comp).unwrap();
        trigger_vec.push((event.key.clone(), map.get(&event.key).unwrap().clone()));
      }
    }

    if !self.new_events.contains_key(&event.key) {
      self.new_events.insert(event.key.clone(), Vec::new());
    }
    self.new_events.get_mut(&event.key).unwrap().push(event);
    true
  }

  pub fn add_listener(
    &mut self, 
    component: ComponentKey, 
    event: EventKey,
    function: fn(&mut dyn EventListener, Event) -> ()
  ) -> Result<(), EngineError> {
    if !self.event_listeners.contains_key(&component) {
      self.event_listeners.insert(component.clone(), HashMap::new());
    }
    if !self.event_listeners.get_mut(&component).unwrap().insert(event, function).is_none() {
      println!("Event listener successfully added");
      return Ok(())
    }

    Err(EngineError::Custom("Hashmap insertion failure".into()))
  }

  pub fn remove_listener(
    &mut self,
    component: &ComponentKey,
    event: &EventKey,
  ) -> Result<(), EngineError> {
    if !self.event_listeners.contains_key(component) {
      return Err(EngineError::ArgumentError { index: 1, name: "component".into() })
    }

    let event_map = self.event_listeners.get_mut(component).unwrap();
    if !event_map.remove(event).is_none() {
      return Err(EngineError::ArgumentError { index: 2, name: "event".into() });
    }
    Ok(())
  }

  pub fn trigger_callbacks(
    &mut self,
    components: &mut ComponentStore,
  ) {
    let mut callbacks_to_trigger: HashMap<EventKey, Vec<(Component, fn(&mut dyn EventListener, Event) -> ())>> = HashMap::new();
    for (comp, events) in self.triggered_events.iter() {
      if let Some(component) = components.get_mut(comp) {
        let mut triggered_events: HashSet<EventKey> = HashSet::new();
        for (key, callback) in events {
          if triggered_events.contains(&key) || !self.new_events.contains_key(&key) {
            continue;
          }
          triggered_events.insert(key.clone());
          if !callbacks_to_trigger.contains_key(key) {
            callbacks_to_trigger.insert(key.clone(), Vec::new());
          }
          let cloned = component.clone();
          callbacks_to_trigger.get_mut(key).unwrap().push((cloned, callback.clone()));
        }
      }
    }

    for (key, callbacks) in callbacks_to_trigger.iter_mut() {
      for event in self.new_events.remove(&key).unwrap_or(Vec::new()) {
        for (component, callback) in callbacks.iter_mut() {
          (*callback)(component, event.clone());
        }
      }
    }
    
    self.new_events.clear();
  }
}
