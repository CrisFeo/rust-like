use rust_like::*;
use std::rc::Rc;

fn main() {
  let mut world = World::default();

  let visibility_cache = Rc::new(VisibilityCache::new(100));

  let player = {
    let id = world.view_target;
    world.exists.insert(id);
    world.name.insert(id, "Player");
    world.icon.insert(id, '@');
    world.layer.insert(id, Layer::Mob);
    world.position.insert(id, (4, 4));
    world.solidity.insert(id);
    world.controls.insert(
      id,
      Controls {
        up: 'k',
        down: 'j',
        left: 'h',
        right: 'l',
      },
    );
    world.health.insert(id, 3);
    world.weapon.insert(id, 1);
    world.speed.insert(id, 5);
    world
      .fov
      .insert(id, FieldOfView::new(visibility_cache.clone()));
    world.timeline.push(0, Event::Turn(id));
    id
  };

  let mut goblin = |i, p| {
    let id = Id::new();
    world.exists.insert(id);
    world.name.insert(id, "Goblin");
    world.icon.insert(id, i);
    world.layer.insert(id, Layer::Mob);
    world.position.insert(id, p);
    world.solidity.insert(id);
    world.ai.insert(id, Ai { target: player });
    world.health.insert(id, 1);
    world.weapon.insert(id, 1);
    world.speed.insert(id, 10);
    world.timeline.push(0, Event::Turn(id));
    id
  };
  _ = goblin('G', (8, 3));
  _ = goblin('N', (9, 2));
  _ = goblin('T', (8, 5));

  let size = 20;
  for y in 0..size {
    for x in 0..size {
      let position = (x, y);
      let is_wall = match position {
        (0, _) => true,
        (x, _) if x == size - 1 => true,
        (_, 0) => true,
        (_, y) if y == size - 1 => true,
        _ => false,
      };
      let id = Id::new();
      world.exists.insert(id);
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
    world.exists.insert(id);
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

  world.startup();

  let mut t = Terminal::new().unwrap();
  loop {
    world.draw(&mut t).unwrap();
    use terminal::Event::*;
    match t.poll() {
      Resize() => {}
      Input(char) => {
        if char == 'q' {
          break;
        }
        world.update(char);
      }
    }
  }
}
