use std::collections::HashMap;
use crate::*;

pub fn arena(world: &mut World) {
  for position in grid::spiral((0, 0), 100) {
    let id = Id::new();
    world.position.insert(id, position);
    world.layer.insert(id, Layer::Map);
    let distance =
      ((position.0.pow(2) + position.1.pow(2)) as f32).sqrt();
    if distance < 20.0 {
      world.name.insert(id, "floor");
      world.icon.insert(id, '.');
      world.navigation.set_value(position, usize::MAX);
    } else {
      world.name.insert(id, "wall");
      world.icon.insert(id, '#');
      world.solidity.insert(id);
      world.opacity.insert(id);
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
    world.navigation.remove_point(p);
    id
  };
  pillar((5, 5));
  pillar((5, 6));
  pillar((5, 7));
  pillar((4, 7));
  pillar((4, 9));
}

pub fn cavern(world: &mut World, seed: u32, walkers: usize, max_steps: usize) {
  let mut random = Random::new(seed);
  let mut cells = HashMap::new();
  for position in grid::spiral((0, 0), 100) {
    cells.insert(position, false);
  }
  let rooms = [
    (0, 0),
    {
      let start_index = random.range(0, cells.len() as i32) as usize;
      cells.keys().nth(start_index).cloned().unwrap()
    },
    {
      let start_index = random.range(0, cells.len() as i32) as usize;
      cells.keys().nth(start_index).cloned().unwrap()
    },
    {
      let start_index = random.range(0, cells.len() as i32) as usize;
      cells.keys().nth(start_index).cloned().unwrap()
    },
  ];
  for center in rooms {
    let radius = random.range(10, 20);
    for position in grid::spiral(center, radius) {
      let delta = (position.0 - center.0, position.1 - center.1);
      let distance = ((delta.0.pow(2) + delta.1.pow(2)) as f32).sqrt();
      if distance < 10.0 {
        cells.insert(position, true);
      }
    }
  }
  for _ in 0..walkers {
    let start_index = random.range(0, cells.len() as i32) as usize;
    let mut position = cells.keys().nth(start_index).cloned().unwrap();
    for _ in 0..max_steps {
      let next_position = match (random.bool(), random.bool()) {
        (true, true)   => (position.0 - 1, position.1    ),
        (true, false)  => (position.0 + 1, position.1    ),
        (false, true)  => (position.0,     position.1 - 1),
        (false, false) => (position.0,     position.1 + 1),
      };
      if cells.get(&next_position).cloned().unwrap_or(false) {
        cells.insert(position, true);
        break;
      }
      position = next_position;
    }
  }
  for (position, is_open) in cells {
    let id = Id::new();
    world.position.insert(id, position);
    world.layer.insert(id, Layer::Map);
    if is_open {
      world.name.insert(id, "floor");
      world.icon.insert(id, '.');
      world.navigation.set_value(position, usize::MAX);
    } else {
      world.name.insert(id, "wall");
      world.icon.insert(id, '#');
      world.solidity.insert(id);
      world.opacity.insert(id);
    }
  }
}

struct Random(u32);

impl Random {
  pub fn new(seed: u32) -> Self {
    Self(seed)
  }

  pub fn bool(&mut self) -> bool {
    self.0 = hash_u32(self.0);
    self.0 % 2 == 0
  }

  pub fn range(&mut self, a: i32, b: i32) -> i32 {
    self.0 = hash_u32(self.0);
    let v = self.0 as f32;
    let t = v / (u32::MAX as f32);
    let d = t * (b as f32 - a as f32);
    a + d as i32
  }
}

pub const fn hash_u32(x: u32) -> u32 {
  let mut v = x;
  v ^= v.wrapping_shr(16);
  v = v.wrapping_mul(0x85ebca6b);
  v ^= v.wrapping_shr(13);
  v = v.wrapping_mul(0xc2b2ae35);
  v ^= v.wrapping_shr(16);
  v
}
