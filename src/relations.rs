use std::cmp::Eq;
use std::collections::*;
use std::hash::Hash;

pub struct Is<V> {
  set: HashSet<V>,
}

impl<V> Default for Is<V> {
  fn default() -> Self {
    Self {
      set: HashSet::new(),
    }
  }
}

impl<V> Is<V>
where
  V: Hash + Eq + Copy,
{
  pub fn contains(&self, value: &V) -> bool {
    self.set.contains(value)
  }

  pub fn get(&self, value: &V) -> Option<&V> {
    self.set.get(value)
  }

  pub fn iter(&self) -> hash_set::Iter<'_, V> {
    self.set.iter()
  }

  pub fn insert(&mut self, value: V) -> bool {
    self.set.insert(value)
  }

  pub fn remove(&mut self, value: &V) -> bool {
    self.set.remove(value)
  }
}

pub struct HasOne<L, R> {
  map: HashMap<L, R>,
}

impl<L, R> Default for HasOne<L, R> {
  fn default() -> Self {
    Self {
      map: HashMap::new(),
    }
  }
}

impl<L, R> HasOne<L, R>
where
  L: Hash + Eq + Copy,
{
  pub fn get(&self, left: &L) -> Option<&R> {
    self.map.get(left)
  }

  pub fn get_mut(&mut self, left: &L) -> Option<&mut R> {
    self.map.get_mut(left)
  }

  pub fn iter(&self) -> hash_map::Iter<'_, L, R> {
    self.map.iter()
  }

  pub fn iter_mut(&mut self) -> hash_map::IterMut<'_, L, R> {
    self.map.iter_mut()
  }

  pub fn contains_key(&self, left: &L) -> bool {
    self.map.contains_key(left)
  }

  pub fn insert(&mut self, left: L, right: R) -> Option<R> {
    self.map.insert(left, right)
  }

  pub fn remove(&mut self, left: &L) -> Option<R> {
    self.map.remove(left)
  }
}

pub struct HasMany<L, R> {
  map: HashMap<L, HashSet<R>>,
}

impl<L, R> Default for HasMany<L, R> {
  fn default() -> Self {
    Self {
      map: HashMap::new(),
    }
  }
}

impl<L, R> HasMany<L, R>
where
  L: Hash + Eq + Copy,
  R: Hash + Eq + Copy,
{
  pub fn get(&self, left: &L) -> Option<&HashSet<R>> {
    self.map.get(left)
  }

  pub fn iter(&self) -> hash_map::Iter<'_, L, HashSet<R>> {
    self.map.iter()
  }

  pub fn iter_mut(&mut self) -> hash_map::IterMut<'_, L, HashSet<R>> {
    self.map.iter_mut()
  }

  pub fn contains_key(&self, left: &L) -> bool {
    self.map.contains_key(left)
  }

  pub fn insert(&mut self, left: L, right: R) -> bool {
    self
      .map
      .entry(left)
      .or_insert_with(|| HashSet::new())
      .insert(right)
  }

  pub fn remove_by_left(&mut self, left: &L) -> Option<HashSet<R>> {
    self.map.remove(left)
  }

  pub fn remove_by_right(&mut self, left: &L, right: &R) -> bool {
    let Some(rights) = self.map.get_mut(left) else {
      return false;
    };
    let was_removed = rights.remove(right);
    if was_removed && rights.is_empty() {
      self.map.remove(left);
    }
    was_removed
  }
}

#[derive(Default)]
pub struct ManyToOne<L, R> {
  by_left: HashMap<L, R>,
  by_right: HashMap<R, HashSet<L>>,
}

impl<L, R> ManyToOne<L, R>
where
  L: Hash + Eq + Copy,
  R: Hash + Eq + Copy,
{
  pub fn get_right(&self, left: &L) -> Option<&R> {
    self.by_left.get(left)
  }

  pub fn get_lefts(&self, right: &R) -> Option<&HashSet<L>> {
    self.by_right.get(right)
  }

  pub fn insert(&mut self, left: L, right: R) -> Option<R> {
    let previous_right = self.remove_by_left(&left);
    self.by_left.insert(left, right);
    if let Some(lefts) = self.by_right.get_mut(&right) {
      lefts.insert(left);
    } else {
      let mut lefts = HashSet::new();
      lefts.insert(left);
      self.by_right.insert(right, lefts);
    }
    previous_right
  }

  pub fn remove_by_left(&mut self, left: &L) -> Option<R> {
    let right = self.by_left.remove(left);
    if let Some(right) = right {
      if let Some(lefts) = self.by_right.get_mut(&right) {
        lefts.remove(left);
      }
    }
    right
  }

  pub fn remove_by_right(&mut self, right: &R) -> Option<HashSet<L>> {
    let previous_lefts = self.by_right.remove(right);
    if let Some(ref previous_lefts) = previous_lefts {
      for left in previous_lefts.iter() {
        self.by_left.remove(left);
      }
    }
    previous_lefts
  }
}

/*
#[derive(Default)]
pub struct OneToOne<L, R> {
  by_left: HashMap<L, R>,
  by_right: HashMap<R, L>,
}

impl<L, R> OneToOne<L, R>
where
  L: Hash + Eq + Copy,
  R: Hash + Eq + Copy,
{
  pub fn get_right(&self, left: &L) -> Option<&R> {
    self.by_left.get(left)
  }

  pub fn get_left(&self, right: &R) -> Option<&L> {
    self.by_right.get(right)
  }

  pub fn insert(&mut self, left: L, right: R) -> Option<R> {
    let previous_right = self.remove_by_left(&left);
    self.by_left.insert(left, right);
    self.by_right.insert(right, left);
    previous_right
  }

  pub fn remove_by_left(&mut self, left: &L) -> Option<R> {
    let previous_right = self.by_left.remove(left);
    if let Some(previous_right) = previous_right {
      self.by_right.remove(&previous_right);
    }
    previous_right
  }

  pub fn remove_by_right(&mut self, right: &R) -> Option<L> {
    let previous_left = self.by_right.remove(right);
    if let Some(previous_left) = previous_left {
      self.by_left.remove(&previous_left);
    }
    previous_left
  }
}

#[derive(Default)]
pub struct ManyToMany<L, R> {
  by_left: HashMap<L, HashSet<R>>,
  by_right: HashMap<R, HashSet<L>>,
}

impl<L, R> ManyToMany<L, R>
where
  L: Hash + Eq + Copy,
  R: Hash + Eq + Copy,
{
  pub fn get_rights(&self, left: &L) -> Option<&HashSet<R>> {
    self.by_left.get(left)
  }

  pub fn get_lefts(&self, right: &R) -> Option<&HashSet<L>> {
    self.by_right.get(right)
  }

  pub fn insert(&mut self, left: L, right: R) {
    self
      .by_left
      .entry(left)
      .or_insert_with(|| HashSet::new())
      .insert(right);
    self
      .by_right
      .entry(right)
      .or_insert_with(|| HashSet::new())
      .insert(left);
  }

  pub fn remove_by_left(&mut self, left: &L) -> Option<HashSet<R>> {
    let previous_rights = self.by_left.remove(left);
    if let Some(ref previous_rights) = previous_rights {
      for right in previous_rights.iter() {
        if let Some(lefts) = self.by_right.get_mut(right) {
          if lefts.len() == 1 {
            self.by_right.remove(right);
          } else {
            lefts.remove(left);
          }
        }
      }
    }
    previous_rights
  }

  pub fn remove_by_right(&mut self, right: &R) -> Option<HashSet<L>> {
    let previous_lefts = self.by_right.remove(right);
    if let Some(ref previous_lefts) = previous_lefts {
      for left in previous_lefts.iter() {
        if let Some(rights) = self.by_left.get_mut(left) {
          if rights.len() == 1 {
            self.by_left.remove(left);
          } else {
            rights.remove(right);
          }
        }
      }
    }
    previous_lefts
  }
}
*/
