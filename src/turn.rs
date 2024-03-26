use crate::*;

#[derive(Debug)]
pub enum Turn {
  Player,
  Ai(Option<Action>),
}

pub fn update_turn(world: &mut World, id: Id, turn: Turn) {
  match turn {
    Turn::Player => update_player(world, id),
    Turn::Ai(action) => update_ai(world, id, action),
  }
}

fn update_player(world: &mut World, id: Id) {
  let Some(controls) = world.controls.get(&id) else {
    return;
  };
  let Some(position) = world.position.get_right(&id) else {
    return;
  };
  let direction = match world.input.take_or_request() {
    Some(i) if i == controls.up => (0, -1),
    Some(i) if i == controls.down => (0, 1),
    Some(i) if i == controls.left => (-1, 0),
    Some(i) if i == controls.right => (1, 0),
    Some(i) if i == controls.wait => (0, 0),
    _ => {
      world.current_event = Some(Event::Turn(id, Turn::Player));
      return;
    }
  };
  let mut speed = 1;
  if direction != (0, 0) {
    let position = (position.0 + direction.0, position.1 + direction.1);
    if let Some(move_speed) = pick_step(world, id) {
      speed = move_speed;
      let mut action = Action::Move(direction);
      if let Some((attack_speed, attack_damage)) = pick_melee_attack(world, id) {
        if let Some(ids) = world.position.get_lefts(&position) {
          for target_id in ids.iter() {
            if world.solidity.contains(target_id) {
              speed = attack_speed;
              action = Action::Attack(direction, attack_damage);
              break;
            }
          }
        }
      }
      update_action(world, id, action);
    }
  }
  world
    .timeline
    .push(world.time + speed, Event::Turn(id, Turn::Player));
}

fn update_ai(world: &mut World, id: Id, action: Option<Action>) {
  if let Some(action) = action {
    update_action(world, id, action);
  }
  let Some(ai) = world.ai.get(&id) else {
    return;
  };
  let Some(position) = world.position.get_right(&id) else {
    return;
  };
  let Some(target_position) = world.position.get_right(&ai.target) else {
    return;
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
  let mut speed = 1;
  let mut action = None;
  if let Some(move_speed) = pick_step(world, id) {
    speed = move_speed;
    action = Some(Action::Move(vector));
    if let Some((attack_speed, attack_damage)) = pick_melee_attack(world, id) {
      if desired_position == *target_position {
        speed = attack_speed;
        action = Some(Action::Attack(vector, attack_damage));
      }
    }
  }
  world
    .timeline
    .push(world.time + speed, Event::Turn(id, Turn::Ai(action)));
}

fn collect_activities(world: &World, id: Id) -> impl Iterator<Item = &Activity> {
  std::iter::once(id)
    .chain(
      world
        .held_by
        .get_lefts(&id)
        .into_iter()
        .flat_map(|ids| ids.iter())
        .copied(),
    )
    .filter_map(|item_id| world.provides_activity.get(&item_id))
    .flatten()
}

fn pick_step(world: &World, id: Id) -> Option<usize> {
  collect_activities(world, id)
    .find(|activity| matches!(activity.activity_type, ActivityType::Step()))
    .map(|a| a.speed)
}

fn pick_melee_attack(world: &World, id: Id) -> Option<(usize, i32)> {
  collect_activities(world, id)
    .filter_map(|activity| {
      if let ActivityType::MeleeAttack(damage) = activity.activity_type {
        Some((activity.speed, damage))
      } else {
        None
      }
    })
    .next()
}
