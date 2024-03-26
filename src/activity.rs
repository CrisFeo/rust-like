#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Activity {
  pub speed: usize,
  pub activity_type: ActivityType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActivityType {
  Step(),
  MeleeAttack(i32),
  RangeAttack(i32),
}
