use std::{any::Any, collections::{hash_map, HashMap}, fmt::format};
use cgmath::Point3;
use std::hash::{
  Hash, Hasher
};

pub struct Point(pub Point3<f32>);

impl PartialEq for Point {
  fn eq(&self, other: &Self) -> bool {
      if self.0.x == other.0.x && self.0.y == other.0.y && self.0.z == other.0.z {
        return true;
      }
      false
  }
}

impl Eq for Point {}

impl Hash for Point {
  fn hash<H: Hasher>(&self, state: &mut H) {
    format!("x: {}, y: {}, z: {}", self.0.x, self.0.y, self.0.z).hash(state)
  }
}

pub struct PointDict<T> {
  map: HashMap<Point, T>,
}

impl<T> PointDict<T> {
  pub fn new() -> PointDict<T> {
    let map: HashMap<Point, T> = HashMap::new();
    PointDict {
      map
    }
  }

  pub fn insert(&mut self, key: Point3<f32>, val: T) -> Option<T> {
    self.map.insert(Point(key), val)
  }

  pub fn remove(&mut self, key: Point3<f32>) -> Option<T> {
    self.map.remove(&Point(key))
  }

  pub fn get(&self, key: Point3<f32>) -> Option<&T> {
    self.map.get(&Point(key))
  }

  pub fn iter(&self) -> hash_map::Iter<Point, T> {
    self.map.iter()
  }
}