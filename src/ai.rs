use crate::*;

#[derive(Debug)]
pub struct Ai {
  // TODO switch ai to use a "target nav map" instead of a specific target id
  pub target: Id,
}

pub fn pick_ai_action(world: &World, id: Id) -> (usize, Option<Action>) {
  let Some(ai) = world.ai.get(&id) else {
    return (1, None);
  };
  let Some(position) = world.position.get_right(&id) else {
    return (1, None);
  };
  let Some(navigation) = world.navigation.best_neighbor(*position) else {
    return (1, None);
  };
  let (desired_position, remaining_steps) = navigation;
  let activities = collect_activities(world, id).collect::<Vec<_>>();
  let mut speed = 1;
  let mut action = None;
  if let Some(move_speed) = pick_step(&activities) {
    let move_vector = (desired_position.0 - position.0, desired_position.1 - position.1);
    speed = move_speed;
    action = Some(Action::Move(move_vector));
    if remaining_steps == 0 {
      if let Some((attack_speed, attack_damage)) = pick_melee_attack(&activities) {
        speed = attack_speed;
        action = Some(Action::Attack(move_vector, attack_damage));
      }
    }
  }
  (speed, action)
}

fn pick_step(activities: &[(Id, &Activity)]) -> Option<usize> {
  activities
    .iter()
    .find(|(_, activity)| matches!(activity.activity_type, ActivityType::Step()))
    .map(|(_, activity)| activity.speed)
}

fn pick_melee_attack(activities: &[(Id, &Activity)]) -> Option<(usize, i32)> {
  activities
    .iter()
    .filter_map(|(_, activity)| {
      if let ActivityType::MeleeAttack(damage) = activity.activity_type {
        Some((activity.speed, damage))
      } else {
        None
      }
    })
    .next()
}
