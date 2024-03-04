use crate::*;

#[derive(Debug)]
pub enum Event {
  Turn(Id),
  Action(Id, Action),
}

pub fn update_action_event(world: &mut World) {
  let current_event = world.current_event.take();
  let Some(Event::Action(id, action)) = current_event else {
    world.current_event = current_event;
    return;
  };
  if world.exists.get(&id).is_none() {
    world.current_event = None;
    return;
  }
  update_action(world, action, id);
  world.current_event = Some(Event::Turn(id));
}

pub fn update_turn_event(world: &mut World) {
  let Some(Event::Turn(id)) = world.current_event else {
    return;
  };
  if world.exists.get(&id).is_none() {
    return;
  }
  let turn_handlers = [
    update_turn_controls,
    update_turn_ai,
  ];
  for handler in turn_handlers {
    let completed = handler(world, id);
    if completed {
      world.current_event = None;
      break;
    }
  }
}

fn update_turn_controls(world: &mut World, id: Id) -> bool {
  let Some(controls) = world.controls.get(&id) else {
    return false;
  };
  let direction = match world.input.take_or_request() {
    Some(i) if i == controls.up => (0, -1),
    Some(i) if i == controls.down => (0, 1),
    Some(i) if i == controls.left => (-1, 0),
    Some(i) if i == controls.right => (1, 0),
    _ => return false,
  };
  let Some(position) = world.position.get(&id) else {
    return true;
  };
  let Some(speed) = world.speed.get(&id) else {
    return true;
  };
  let position = (position.0 + direction.0, position.1 + direction.1);
  let mut action = Action::Move(direction);
  if let Some(ids) = world.position.at(position) {
    for target_id in ids.iter() {
      if world.solidity.contains(target_id) {
        action = Action::Attack(direction);
        break;
      }
    }
  }
  world.timeline.push(world.time + *speed as usize, Event::Action(id, action));
  true
}

fn update_turn_ai(world: &mut World, id: Id) -> bool {
  let Some(ai) = world.ai.get(&id) else {
    return false;
  };
  let Some(position) = world.position.get(&id) else {
    return true;
  };
  let Some(speed) = world.speed.get(&id) else {
    return true;
  };
  let Some(target_position) = world.position.get(&ai.target) else {
    return true;
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
  let action = if desired_position == *target_position {
    Action::Attack(vector)
  } else {
    Action::Move(vector)
  };
  world.timeline.push(world.time + *speed as usize, Event::Action(id, action));
  true
}

