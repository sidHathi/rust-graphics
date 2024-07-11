use std::collections::{HashMap, HashSet};

use crate::engine::{component::{self, Component}, component_store::ComponentKey, errors::EngineError, Scene};

use super::event::{Event, EventKey};

pub struct EventManager<'a> {
  new_events: HashMap<EventKey, Vec<Event>>,
  event_listeners: HashMap<ComponentKey, HashMap<EventKey, fn(&mut Component, Event) -> ()>>,
  triggered_events: HashMap<ComponentKey, Vec<(EventKey, &'a fn(&mut Component, Event) -> ())>>
}

impl<'a> EventManager<'a> {
  pub fn new() -> EventManager<'a> {
    Self {
      new_events: HashMap::new(),
      event_listeners: HashMap::new(),
      triggered_events: HashMap::new()
    }
  }

  pub fn handle_event(&'a mut self, event: Event) -> bool {
    for (comp, map) in self.event_listeners.iter() {
      if map.contains_key(&event.key) {
        if !self.triggered_events.contains_key(comp) {
          self.triggered_events.insert(comp.clone(), Vec::new());
        }
        let trigger_vec = self.triggered_events.get_mut(comp).unwrap();
        trigger_vec.push((event.key.clone(), map.get(&event.key).unwrap()));
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
    function: fn(&mut Component, Event) -> ()
  ) -> Result<(), EngineError> {
    if !self.event_listeners.contains_key(&component) {
      self.event_listeners.insert(component.clone(), HashMap::new());
    }
    if !self.event_listeners.get_mut(&component).unwrap().insert(event, function).is_none() {
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
    scene: &mut Scene,
  ) {
    for (comp, events) in self.triggered_events.iter() {
      if let Some(component) = scene.components.get_mut(comp) {
        let mut triggered_events: HashSet<EventKey> = HashSet::new();
        for (key, callback) in events {
          if triggered_events.contains(&key) || !self.new_events.contains_key(&key) {
            continue;
          }
          triggered_events.insert(key.clone());
          for event in self.new_events.remove(&key).unwrap() {
            (**callback)(component, event);
          }
          
        }
      }
    }

    self.new_events.clear();
  }
}