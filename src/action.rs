use crate::*;

#[derive(Debug)]
pub enum Action {
  Move((i32, i32)),
  Attack((i32, i32)),
}

pub fn update_action(world: &mut World, action: Action, id: Id) {
  match action {
    Action::Move(vector) => update_move(world, id, vector),
    Action::Attack(vector) => update_attack(world, id, vector),
  }
}

fn update_move(world: &mut World, id: Id, vector: (i32, i32)) {
  let Some(position) = world.position.get(&id) else {
    return;
  };
  let position = (position.0 + vector.0, position.1 + vector.1);
  let mut is_blocked = false;
  if let Some(ids) = world.position.at(position) {
    for target_id in ids.iter() {
      if *target_id == id {
        continue;
      }
      if world.solidity.contains(target_id) {
        is_blocked = true;
        break;
      }
    }
  }
  if !is_blocked {
    world.position.insert(id, position);
  }
}

fn update_attack(world: &mut World, id: Id, vector: (i32, i32)) {
  let Some(position) = world.position.get(&id) else {
    return;
  };
  let position = (position.0 + vector.0, position.1 + vector.1);
  let Some(weapon) = world.weapon.get(&id) else {
    return;
  };
  let Some(ids) = world.position.at(position) else {
    return;
  };
  let mut target_id = None;
  for id in ids.iter() {
    if world.health.contains_key(id) {
      target_id = Some(*id);
      break;
    }
  }
  let Some(target_id) = target_id else {
    return;
  };
  let Some(health) = world.health.get_mut(&target_id) else {
    return;
  };
  *health = (*health - *weapon).max(0);
}

