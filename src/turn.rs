use crate::*;

#[derive(Debug)]
pub enum TurnType {
  Player(Player),
  Ai(Ai),
}

impl TurnType {
  pub fn update(self, world: &mut World, id: Id) {
    match self {
      Self::Player(player) => player.update(world, id),
      Self::Ai(ai) => ai.update(world, id),
    }
  }
}

#[derive(Debug)]
pub struct Player {
  pub selected_activity_index: usize,
}

impl Player {
  pub fn new_turn(id: Id) -> Event {
    let turn = Self { selected_activity_index: 0 };
    Event::Turn(id, TurnType::Player(turn))
  }

  pub fn next_turn(self, id: Id) -> Event {
    let turn = Self {
      selected_activity_index: self.selected_activity_index
    };
    Event::Turn(id, TurnType::Player(turn))
  }

  fn update(self, world: &mut World, id: Id) {
    let Some(controls) = world.controls.get(&id) else {
      return;
    };
    let result = match world.input.take_or_request() {
      Some(i) if i == controls.act_up => self.act(world, id, (0, -1)),
      Some(i) if i == controls.act_down => self.act(world, id, (0, 1)),
      Some(i) if i == controls.act_left => self.act(world, id, (-1, 0)),
      Some(i) if i == controls.act_right => self.act(world, id, (1, 0)),
      Some(i) if i == controls.act_center => self.act(world, id, (0, 0)),
      Some(i) if i == controls.activity_previous => self.select_activity(world, id, -1),
      Some(i) if i == controls.activity_next => self.select_activity(world, id, 1),
      _ => Some(self),
    };
    world.current_event = result.map(|r| Event::Turn(id, TurnType::Player(r)));
  }

  fn select_activity(self, world: &mut World, id: Id, delta: i32) -> Option<Self> {
    let total = collect_activities(world, id).count();
    let magnitude = delta.unsigned_abs() as usize;
    let index = if delta < 0 {
      if magnitude > self.selected_activity_index {
        let from_end = magnitude.saturating_sub(self.selected_activity_index);
        total.saturating_sub(from_end)
      } else {
        self.selected_activity_index.saturating_sub(magnitude)
      }
    } else {
        self.selected_activity_index.saturating_add(magnitude) % total
    };
    Some(Self{ selected_activity_index: index })
  }

  fn act(self, world: &mut World, id: Id, direction: (i32, i32)) -> Option<Self> {
    let activity = collect_activities(world, id).nth(self.selected_activity_index);
    let (_, activity) = activity?;
    let activity = *activity;
    match activity.activity_type {
      ActivityType::Wait() => {
        world.timeline.push(world.time + activity.speed, self.next_turn(id));
      }
      ActivityType::Step() => {
        let action = Action::Move(direction);
        update_action(world, id, action);
        world.timeline.push(world.time + activity.speed, self.next_turn(id));
      },
      ActivityType::MeleeAttack(damage) => {
        let action = Action::Attack(direction, damage);
        update_action(world, id, action);
        world.timeline.push(world.time + activity.speed, self.next_turn(id));
      },
    }
    None
  }


}

#[derive(Debug)]
pub struct Ai {
  pub pending_action: Option<Action>,
}

impl Ai {
  pub fn new_turn(id: Id, action: Option<Action>) -> Event {
    let turn = Self {
      pending_action: action,
    };
    Event::Turn(id, TurnType::Ai(turn))
  }

  fn update(self, world: &mut World, id: Id) {
    if let Some(pending_action) = self.pending_action {
      update_action(world, id, pending_action);
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
    world.timeline.push(
      world.time + speed,
      Event::Turn(
        id,
        TurnType::Ai(Self {
          pending_action: action,
        }),
      ),
    );
  }
}

fn pick_step(world: &World, id: Id) -> Option<usize> {
  collect_activities(world, id)
    .find(|(_, activity)| matches!(activity.activity_type, ActivityType::Step()))
    .map(|(_, activity)| activity.speed)
}

fn pick_melee_attack(world: &World, id: Id) -> Option<(usize, i32)> {
  collect_activities(world, id)
    .filter_map(|(_, activity)| {
      if let ActivityType::MeleeAttack(damage) = activity.activity_type {
        Some((activity.speed, damage))
      } else {
        None
      }
    })
    .next()
}
