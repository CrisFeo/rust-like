use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Activity {
  pub name: &'static str,
  pub speed: usize,
  pub activity_type: ActivityType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActivityType {
  Wait(),
  Step(),
  MeleeAttack(i32),
}

pub fn held_items(world: &World, id: Id) -> impl Iterator<Item = &Id> {
  world
    .held_by
    .get_lefts(&id)
    .into_iter()
    .flat_map(|ids| ids.iter())
}

pub fn collect_activities(world: &World, id: Id) -> impl Iterator<Item = (Id, &Activity)> {
  std::iter::once(id)
    .chain(held_items(world, id).copied())
    .filter_map(|id| world.provides_activity.get(&id).map(|a| (id, a)))
    .flat_map(|(id, activities)| activities.iter().map(move |a| (id, a)))
}
