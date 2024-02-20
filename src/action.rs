use crate::*;

pub trait Action {
  fn run(&mut self, world: &mut World) -> bool;
}

pub struct Turn(pub Id, pub Option<Box<dyn Action>>);

impl Action for Turn {
  fn run(&mut self, world: &mut World) -> bool {
    if let Some(action) = &mut self.1 {
      let inner_action_done = action.run(world);
      if inner_action_done {
        self.1 = None;
      } else {
        return false;
      }
    }
    let id = self.0;
    let mut action: Option<Box<dyn Action>> = None;
    if let Some(controls) = world.controls.get(&id) {
      let direction = match world.input.take() {
        Some(i) if i == controls.up =>    ( 0, -1),
        Some(i) if i == controls.down =>  ( 0,  1),
        Some(i) if i == controls.left =>  (-1,  0),
        Some(i) if i == controls.right => ( 1,  0),
        _ => return false,
      };
      if let Some(position) = world.position.get(&id) {
        let position = (position.0 + direction.0, position.1 + direction.1);
        action = Some(Box::new(Move(id, position)));
        if let Some(ids) = world.position.at(position) {
          for target_id in ids.iter() {
            if world.solidity.contains(target_id) {
              action = Some(Box::new(Attack(id, position)));
              break;
            }
          }
        }
      }
    } else if let Some(ai) = world.ai.get(&id) {
      if let Some(&position) = world.position.get(&id) {
        if let Some(&target_position) = world.position.get(&ai.target) {
          let dx = target_position.0 - position.0;
          let dy = target_position.1 - position.1;
          let mut position = position;
          if dx.abs() > dy.abs() {
            position.0 += dx.signum();
          } else {
            position.1 += dy.signum();
          }
          action = if position == target_position {
            Some(Box::new(Attack(id, position)))
          } else {
            Some(Box::new(Move(id, position)))
          };
        }
      }
    }
    // Schedule next turn based on speed
    let speed = match world.speed.get(&id) {
      Some(speed) => *speed,
      None => 10,
    };
    world.timeline.push(Event {
      time: world.time + speed as usize,
      action: Box::new(Self(id, action)),
    });
    true
  }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Move(Id, (i32, i32));

impl Action for Move {
  fn run(&mut self, world: &mut World) -> bool {
    let mut is_blocked = false;
    if let Some(ids) = world.position.at(self.1) {
      for target_id in ids.iter() {
        if *target_id == self.0 {
          continue;
        }
        if world.solidity.contains(target_id) {
          is_blocked = true;
          break;
        }
      }
    }
    if !is_blocked {
      world.position.insert(self.0, self.1);
    }
    true
  }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Attack(Id, (i32, i32));

impl Action for Attack {
  fn run(&mut self, world: &mut World) -> bool {
    let Some(ids) = world.position.at(self.1) else {
      return true;
    };
    let mut target_id = None;
    for id in ids.iter() {
      if world.health.contains_key(id) {
        target_id = Some(*id);
        break;
      }
    }
    let Some(target_id) = target_id else {
      return true;
    };
    let Some(health) = world.health.get_mut(&target_id) else {
      return true;
    };
    let Some(weapon) = world.weapon.get(&self.0) else {
      return true;
    };
    *health -= *weapon;
    if *health <= 0 {
      world.remove_entity(&target_id);
    }
    true
  }
}
