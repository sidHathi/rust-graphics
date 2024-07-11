use std::{any, collections::HashMap, future::Future};

use super::{async_closure::run_component_closure, component::{self, Component}, errors::EngineError};


#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub struct ComponentKey {
  pub index: u32,
}

impl ComponentKey {
  pub fn zero() -> ComponentKey {
    Self {
      index: 0
    }
  }
}

pub struct ComponentStore {
  next_idx: u32,
  components: HashMap<ComponentKey, Component>,
}

impl ComponentStore {
  pub fn new() -> ComponentStore {
    Self {
      next_idx: 1,
      components: HashMap::new()
    }
  }

  pub fn insert(&mut self, component: Component) -> Result<ComponentKey, EngineError> {
    if self.next_idx >= u32::MAX {
      return Err(EngineError::MaxComponentsError { insertion_loc: "ComponentStore::insert".into() })
    }

    let key = ComponentKey { index: self.next_idx };
    self.next_idx += 1;
    self.components.insert(key.clone(), component);
    Ok(key)
  }

  pub fn insert_with_key(&mut self, component: Component, key_override: ComponentKey) -> Option<Component> {
    self.components.insert(key_override, component)
  }

  pub fn modify<F>(&mut self, key: ComponentKey, modfunc: F) -> Option<&Component>
    where F: Fn(Component) -> Component {
    if !self.components.contains_key(&key) {
      return None;
    }

    let component = self.components.remove(&key).unwrap();
    let modified = (modfunc)(component);
    self.components.insert(key, modified);
    self.components.get(&key)
  }

  pub async fn modify_async<F, Fut>(&mut self, key: ComponentKey, modfunc: F) -> Option<&Component>
    where
      F: FnOnce(&mut Component) -> Fut,
      Fut: std::future::Future<Output = ()>,
  {
    if !self.components.contains_key(&key) {
      return None;
    }

    let mut component = self.components.remove(&key).unwrap();
    run_component_closure(modfunc, &mut component).await;
    self.components.insert(key, component);
    self.components.get(&key)
  }

  pub fn get(&self, key: &ComponentKey) -> Option<&Component> {
    self.components.get(key)
  }

  pub fn get_mut(&mut self, key: &ComponentKey) -> Option<&mut Component> {
    self.components.get_mut(key)
  }

  pub fn remove(&mut self, key: &ComponentKey) -> Option<Component> {
    self.components.remove(key)
  }

  pub fn keys(&self) -> Vec<&ComponentKey> {
    self.components.keys().into_iter().collect::<Vec<&ComponentKey>>()
  }
}