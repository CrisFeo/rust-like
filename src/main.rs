use std::collections::{
  BinaryHeap,
  HashMap,
  HashSet,
};
use std::rc::Rc;
use rust_like::*;

#[derive(Debug)]
struct Ai {
  target: Id,
}

#[derive(Debug)]
struct Controls {
  up: char,
  down: char,
  left: char,
  right: char,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
enum Layer {
  Map,
  Mob,
  Ui,
}

trait Action {
  fn run(&mut self, world: &mut World) -> bool;
}

struct Turn(Id, Option<Box<dyn Action>>);

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
struct Move(Id, (i32, i32));

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
struct Attack(Id, (i32, i32));

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

struct Event {
  time: usize,
  action: Box<dyn Action>,
}

impl Eq for Event { }

impl PartialEq for Event {
  fn eq(&self, other: &Self) -> bool {
    self.time == other.time
  }
}

impl Ord for Event {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    other.time.cmp(&self.time)
  }
}

impl PartialOrd for Event {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(other.time.cmp(&self.time))
  }
}

struct World<'a> {
  time: usize,
  timeline: BinaryHeap<Event>,
  pending_action: Option<Box<dyn Action>>,
  input: Option<char>,
  name: HashMap<Id, &'a str>,
  icon: HashMap<Id, char>,
  layer: HashMap<Id, Layer>,
  position: SpatialMap,
  solidity: HashSet<Id>,
  opacity: HashSet<Id>,
  ai: HashMap<Id, Ai>,
  controls: HashMap<Id, Controls>,
  speed: HashMap<Id, i32>,
  health: HashMap<Id, i32>,
  weapon: HashMap<Id, i32>,
  fov: FieldOfView,
}

impl<'a> World<'a> {
  fn new() -> Self {
    let visibility_cache = Rc::new(VisibilityCache::new(100));
    World {
      time: 0,
      timeline: BinaryHeap::new(),
      pending_action: None,
      input: None,
      name: HashMap::new(),
      icon: HashMap::new(),
      layer: HashMap::new(),
      position: SpatialMap::new(),
      solidity: HashSet::new(),
      opacity: HashSet::new(),
      ai: HashMap::new(),
      controls: HashMap::new(),
      speed: HashMap::new(),
      health: HashMap::new(),
      weapon: HashMap::new(),
      fov: FieldOfView::new(visibility_cache.clone()),
    }
  }

  fn remove_entity(&mut self, id: &Id) {
    self.name.remove(id);
    self.icon.remove(id);
    self.layer.remove(id);
    self.position.remove(id);
    self.solidity.remove(id);
    self.opacity.remove(id);
    self.ai.remove(id);
    self.controls.remove(id);
    self.speed.remove(id);
    self.health.remove(id);
    self.weapon.remove(id);
  }

  fn update(&mut self, input: char) {
    self.input = Some(input);
    loop {
      if let Some(mut pending_action) = self.pending_action.take() {
        if !pending_action.run(self) {
          self.pending_action = Some(pending_action);
          break;
        }
      }
      let Some(event) = self.timeline.pop() else {
        break;
      };
      self.time = event.time;
      self.pending_action = Some(event.action);
      self.update_fov();
    }
    self.update_fov();
  }

  fn update_fov(&mut self) {
    let Some((player_id, _)) = self.controls.iter().next() else {
      self.fov.update(|p| { false });
      return;
    };
    let Some(player_position) = self.position.get(player_id) else {
      self.fov.update(|p| { false });
      return;
    };
    self.fov.update(|p| {
      let position = (player_position.0 + p.0, player_position.1 + p.1);
      let Some(ids) = self.position.at(position) else {
        return false;
      };
      for id in ids {
        if id == player_id {
          return false;
        }
        if self.opacity.contains(id) {
          return true;
        }
      }
      false
    });
  }

  fn draw(&self, t: &mut Terminal) -> std::io::Result<()> {
    let mut order = self.layer
      .iter()
      .collect::<Vec<_>>();
    order.sort_by_key(|l| l.1);
    let vision_origin = if let Some((player_id, _)) = self.controls.iter().next() {
      self.position.get(player_id)
    } else {
      None
    };

    for (id, _) in order {
      let position = or_continue!(
        self.position.get(id)
      );
      let is_visible = if let Some(vision_origin) = vision_origin {
        let position = (position.0 - vision_origin.0, position.1 - vision_origin.1);
        self.fov.is_visible(position)
      } else {
        false
      };
      let icon = if is_visible {
        let Some(icon) = self.icon.get(id) else {
          continue;
        };
        *icon
      } else {
        '~'
      };
      t.set(*position, icon);
    }
    t.present()
  }
}

// Main

fn main() {
  let mut world = World::new();

  let player = {
    let id = Id::new();
    world.name.insert(id, "Player");
    world.icon.insert(id, '@');
    world.layer.insert(id, Layer::Mob);
    world.position.insert(id, (4, 4));
    world.solidity.insert(id);
    world.controls.insert(id, Controls {
      up: 'k',
      down: 'j',
      left: 'h',
      right: 'l',
    });
    world.health.insert(id, 3);
    world.weapon.insert(id, 1);
    world.speed.insert(id, 5);
    world.timeline.push(Event{time: 0, action: Box::new(Turn(id, None))});
    id
  };

  let mut goblin = |p| {
    let id = Id::new();
    world.name.insert(id, "Goblin");
    world.icon.insert(id, 'G');
    world.layer.insert(id, Layer::Mob);
    world.position.insert(id, p);
    world.solidity.insert(id);
    world.ai.insert(id, Ai {
      target: player,
    });
    world.health.insert(id, 1);
    world.weapon.insert(id, 1);
    world.speed.insert(id, 10);
    world.timeline.push(Event{time: 0, action: Box::new(Turn(id, None))});
    id
  };
  _ = goblin((8, 3));
  _ = goblin((9, 2));
  _ = goblin((8, 5));

  let size = 20;
  for y in 0..size {
    for x in 0..size {
      let position = (x, y);
      let is_wall = match position {
        (x, _) if x == 0 => true,
        (x, _) if x == size - 1 => true,
        (_, y) if y == 0 => true,
        (_, y) if y == size - 1 => true,
        _ => false,
      };
      let id = Id::new();
      world.position.insert(id, position);
      world.layer.insert(id, Layer::Map);
      if is_wall {
        world.name.insert(id, "wall");
        world.icon.insert(id, '#');
        world.solidity.insert(id);
        world.opacity.insert(id);
      } else {
        world.name.insert(id, "floor");
        world.icon.insert(id, '.');
      }
    }
  }

  let mut pillar = |p| {
    let id = Id::new();
    world.name.insert(id, "pillar");
    world.icon.insert(id, 'o');
    world.layer.insert(id, Layer::Mob);
    world.position.insert(id, p);
    world.solidity.insert(id);
    world.opacity.insert(id);
    id
  };
  pillar((5, 5));
  pillar((5, 6));
  pillar((5, 7));
  pillar((4, 7));
  pillar((4, 9));

  world.update_fov();

  let mut t = Terminal::new().unwrap();
  loop {
    world.draw(&mut t).unwrap();
    let c = t.read().unwrap();
    if let Some(c) = c {
      if c == 'q' {
        break;
      }
      world.update(c);
    }
  }
}
