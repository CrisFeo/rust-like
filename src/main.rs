use std::collections::{
  BinaryHeap,
  HashMap,
  HashSet,
};
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

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
struct Wait(Id);

impl Action for Wait {
  fn run(&mut self, world: &mut World) -> bool {
    think(world, self.0)
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
    think(world, self.0)
  }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
struct Attack(Id, (i32, i32));

impl Action for Attack {
  fn run(&mut self, world: &mut World) -> bool {
    let mut target_id = None;
    if let Some(ids) = world.position.at(self.1) {
      for id in ids.iter() {
        if world.health.contains_key(id) {
          target_id = Some(*id);
          break;
        }
      }
    }
    if let Some(target_id) = target_id {
  		if let Some(health) = world.health.get_mut(&target_id) {
    		if let Some(weapon) = world.weapon.get(&self.0) {
      		*health -= *weapon;
      		if *health <= 0 {
        		world.remove_entity(&target_id);
      		}
    		}
  		}
    }
    think(world, self.0)
  }
}

fn think(world: &mut World, id: Id) -> bool {
  let mut action: Box<dyn Action> = Box::new(Wait(id));
	if let Some(controls) = world.controls.get(&id) {
		// Look to see if one of this
		// entity's controls was pressed
		let direction = match world.input {
      Some(i) if i == controls.up =>    (0, -1),
      Some(i) if i == controls.down =>  (0, 1),
      Some(i) if i == controls.left =>  (-1, 0),
      Some(i) if i == controls.right => (1, 0),
      _ => return false,
		};
		// Consume the input
		world.input = None;
    if let Some(position) = world.position.get(&id) {
      let position = (position.0 + direction.0, position.1 + direction.1);
      action = Box::new(Move(id, position));
      if let Some(ids) = world.position.at(position) {
        for target_id in ids.iter() {
          if world.solidity.contains(target_id) {
            action = Box::new(Attack(id, position));
            break;
          }
        }
      }
    }
	} else if let Some(ai) = world.ai.get(&id) {
		// Let the AI make a decision
    if let Some(position) = world.position.get(&id) {
      let mut position = *position;
      if let Some(target_position) = world.position.get(&ai.target) {
        let dx = target_position.0 - position.0;
        let dy = target_position.1 - position.1;
        if dx.abs() > dy.abs() {
          position.0 += dx.signum();
        } else {
          position.1 += dy.signum();
        }
        action = if position == *target_position {
          Box::new(Attack(id, position))
        } else {
          Box::new(Move(id, position))
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
		action,
	});
	return true;
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
  ai: HashMap<Id, Ai>,
  controls: HashMap<Id, Controls>,
  speed: HashMap<Id, i32>,
  health: HashMap<Id, i32>,
  weapon: HashMap<Id, i32>,
}

impl<'a> World<'a> {
  fn new() -> Self {
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
      ai: HashMap::new(),
      controls: HashMap::new(),
      speed: HashMap::new(),
      health: HashMap::new(),
      weapon: HashMap::new(),
    }
  }

  fn remove_entity(&mut self, id: &Id) {
    self.name.remove(id);
    self.icon.remove(id);
    self.layer.remove(id);
    self.position.remove(id);
    self.solidity.remove(id);
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
  		let event = match self.timeline.pop() {
    		Some(event) => event,
    		None => break,
  		};
  		self.time = event.time;
  		self.pending_action = Some(event.action);
		}
  }

  fn draw(&self, t: &mut Terminal) -> std::io::Result<()> {
    t.clear()?;
    let mut order = self.layer
      .iter()
      .collect::<Vec<_>>();
    order.sort_by_key(|l| l.1);
    for (id, _) in order {
      let icon = or_continue!(
        self.icon.get(id)
      );
      let (x, y) = or_continue!(
        self.position.get(id)
      );
      t.go(*x, *y)?;
      t.char(*icon)?;
    }
    t.flush()?;
    Ok(())
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
    world.timeline.push(Event{time: 0, action: Box::new(Wait(id))});
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
    world.timeline.push(Event{time: 1, action: Box::new(Wait(id))});
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
      if is_wall {
        world.name.insert(id, "wall");
        world.icon.insert(id, '#');
        world.layer.insert(id, Layer::Map);
        world.position.insert(id, position);
        world.solidity.insert(id);
      } else {
        world.name.insert(id, "floor");
        world.icon.insert(id, '.');
        world.layer.insert(id, Layer::Map);
        world.position.insert(id, position);
      }
    }
  }

	let mut t = Terminal::new().unwrap();
  loop {
    let c = t.read().unwrap();
    if let Some(c) = c {
      if c == 'q' {
        break;
      }
      world.update(c);
    }
    world.draw(&mut t).unwrap();
  }
}
