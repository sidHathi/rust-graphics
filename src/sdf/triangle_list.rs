use super::triangle::Triangle;
use std::hash::{
  Hash, Hasher
};
use std::collections::HashSet;

impl<'a> Hash for Triangle<'a> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.hash_str().hash(state)
  }
}

pub struct TriangleSet<'a> {
  set: HashSet<Triangle<'a>>
}

impl<'a> TriangleSet<'a> {
  pub fn new() -> TriangleSet<'a> {
    TriangleSet {
      set: HashSet::new()
    }
  }

  pub fn from(&self, vals: &'a [Triangle]) -> TriangleSet<'a> {
    TriangleSet {
      set: HashSet::from_iter(vals.iter().cloned())
    }
  }

  pub fn insert(&mut self, val: Triangle<'a>) {
    self.set.insert(val);
  }

  pub fn remove(&mut self, val: &'a Triangle) -> bool {
    self.set.remove(val)
  }
}
