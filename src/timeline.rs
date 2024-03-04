use std::collections::BinaryHeap;
use std::cmp::Ordering;

pub struct Timeline<T>(BinaryHeap<Scheduled<T>>);

impl<T> Default for Timeline<T> {
  fn default() -> Self {
    Timeline(BinaryHeap::default())
  }
}

impl<T> Timeline<T> {

  pub fn push(&mut self, time: usize, item: T) {
    self.0.push(Scheduled(time, item))
  }

  pub fn pop(&mut self) -> Option<(usize, T)> {
    self.0.pop().map(|s| (s.0, s.1))
  }

  pub fn iter(&self) -> impl Iterator<Item = (usize, &T)> + '_ {
    self.0.iter().map(|s| (s.0, &s.1))
  }
}

struct Scheduled<T>(usize, T);

impl<T> Eq for Scheduled<T> {}

impl<T> PartialEq for Scheduled<T> {
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0
  }
}

impl<T> Ord for Scheduled<T> {
  fn cmp(&self, other: &Self) -> Ordering {
    other.0.cmp(&self.0)
  }
}

impl<T> PartialOrd for Scheduled<T> {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(other.0.cmp(&self.0))
  }
}
