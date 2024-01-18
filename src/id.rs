use std::sync::atomic::{
  AtomicUsize,
  Ordering,
};

#[derive(
  Debug,
  Copy,
  Clone,
  Eq,
  PartialEq,
  Ord,
  PartialOrd,
  Hash,
)]
pub struct Id(usize);

static NEXT_ID: AtomicUsize =
  AtomicUsize::new(0);

impl Id {
  pub fn new() -> Self {
    let id = NEXT_ID.fetch_add(
      1,
      Ordering::Relaxed
    );
    Id(id)
  }
}
