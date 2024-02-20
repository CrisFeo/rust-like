use std::cmp::Ordering;
use crate::action::Action;

pub struct Event {
  pub time: usize,
  pub action: Box<dyn Action>,
}

impl Eq for Event { }

impl PartialEq for Event {
  fn eq(&self, other: &Self) -> bool {
    self.time == other.time
  }
}

impl Ord for Event {
  fn cmp(&self, other: &Self) -> Ordering {
    other.time.cmp(&self.time)
  }
}

impl PartialOrd for Event {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(other.time.cmp(&self.time))
  }
}

