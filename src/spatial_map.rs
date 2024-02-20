use std::collections::{HashMap, HashSet};
use crate::id::Id;

pub struct SpatialMap {
  by_id: HashMap<Id, (i32, i32)>,
  by_coordinate: HashMap<(i32, i32), HashSet<Id>>,
}

impl Default for SpatialMap {
  fn default() -> Self {
    Self::new()
  }
}


impl SpatialMap {
  pub fn new() -> Self {
    Self {
      by_id: HashMap::new(),
      by_coordinate: HashMap::new(),
    }
  }

  pub fn get(&self, id: &Id) -> Option<&(i32, i32)> {
    self.by_id.get(id)
  }

  pub fn at(&self, coordinate: (i32, i32)) -> Option<&HashSet<Id>> {
    self.by_coordinate.get(&coordinate)
  }

  pub fn insert(&mut self, id: Id, coordinate: (i32, i32)) -> Option<(i32, i32)> {
    let previous = self.remove(&id);
    self.by_id.insert(id, coordinate);
    if let Some(ids) = self.by_coordinate.get_mut(&coordinate) {
      ids.insert(id);
    } else {
      let mut ids = HashSet::new();
      ids.insert(id);
      self.by_coordinate.insert(coordinate, ids);
    }
    self.by_id.insert(id, coordinate);
    previous
  }

  pub fn remove(&mut self, id: &Id) -> Option<(i32, i32)> {
    let coordinate = self.by_id.remove(id);
    if let Some(coordinate) = coordinate {
      if let Some(ids) = self.by_coordinate.get_mut(&coordinate) {
        ids.remove(id);
      }
    }
    coordinate
  }
}
