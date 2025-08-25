use crate::*;

#[derive(Debug)]
pub struct Ai {
  pub target: Id,
}

pub fn pick_ai_action(world: &World, id: Id) -> (usize, Option<Action>) {
  let Some(ai) = world.ai.get(&id) else {
    return (1, None);
  };
  let Some(position) = world.position.get_right(&id) else {
    return (1, None);
  };
  let Some(target_position) = world.position.get_right(&ai.target) else {
    return (1, None);
  };
  let target_vector = (
    target_position.0 - position.0,
    target_position.1 - position.1,
  );
  let vector = if target_vector.0.abs() > target_vector.1.abs() {
    (target_vector.0.signum(), 0)
  } else {
    (0, target_vector.1.signum())
  };
  let desired_position = (position.0 + vector.0, position.1 + vector.1);
  let activities = collect_activities(world, id).collect::<Vec<_>>();
  let mut speed = 1;
  let mut action = None;
  if let Some(move_speed) = pick_step(&activities) {
    speed = move_speed;
    action = Some(Action::Move(vector));
    if let Some((attack_speed, attack_damage)) = pick_melee_attack(&activities) {
      if desired_position == *target_position {
        speed = attack_speed;
        action = Some(Action::Attack(vector, attack_damage));
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
