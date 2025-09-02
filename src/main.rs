use rust_like::*;
use std::rc::Rc;

fn main() {
  let mut terminal = Terminal::new().unwrap();
  let visibility_cache = Rc::new(VisibilityCache::new(100));
  let mut world = World::default();


  terminal.set_str((0, 0),"generating dungeon...");
  terminal.present().unwrap();

  instrument!("mapping", {
    //mapping::arena(&mut world);
    mapping::cavern(&mut world, 41, 10000, 10000);
  });

  let player = {
    let id = world.view_target;
    world.name.insert(id, "Player");
    world.icon.insert(id, '@');
    world.layer.insert(id, Layer::Mob);
    world.position.insert(id, (0, 0));
    world.solidity.insert(id);
    world.controls.insert(
      id,
      Controls {
        act_up: 'k',
        act_down: 'j',
        act_left: 'h',
        act_right: 'l',
        act_center: 'g',
        activity_previous: 'y',
        activity_next: 'u',
      },
    );
    world.health.insert(id, 3);
    world
      .fov
      .insert(id, FieldOfView::new(visibility_cache.clone()));
    world.timeline.push(0, turn::Player::new_turn(id));
    world.provides_activity.insert(
      id,
      Activity {
        name: "Walk",
        speed: 5,
        activity_type: ActivityType::Step(),
      },
    );
    world.provides_activity.insert(
      id,
      Activity {
        name: "Hold",
        speed: 3,
        activity_type: ActivityType::Wait(),
      },
    );
    let sword = {
      let id = Id::new();
      world.name.insert(id, "Arming Sword");
      world.icon.insert(id, '/');
      world.layer.insert(id, Layer::Mob);
      world.provides_activity.insert(
        id,
        Activity {
          name: "Stab",
          speed: 5,
          activity_type: ActivityType::MeleeAttack(1),
        },
      );
      id
    };
    world.held_by.insert(sword, id);
    id
  };

  let mut goblin = |i, p| {
    let id = Id::new();
    world.name.insert(id, "Goblin");
    world.icon.insert(id, i);
    world.layer.insert(id, Layer::Mob);
    world.position.insert(id, p);
    world.solidity.insert(id);
    world.ai.insert(id, Ai { target: player });
    world.health.insert(id, 1);
    world.timeline.push(0, turn::Ai::new_turn(id));
    world.provides_activity.insert(
      id,
      Activity {
        name: "Walk",
        speed: 10,
        activity_type: ActivityType::Step(),
      },
    );
    let club = {
      let id = Id::new();
      world.name.insert(id, "Crude Club");
      world.icon.insert(id, '!');
      world.layer.insert(id, Layer::Mob);
      world.provides_activity.insert(
        id,
        Activity {
          name: "Wallop",
          speed: 10,
          activity_type: ActivityType::MeleeAttack(1),
        },
      );
      id
    };
    world.held_by.insert(club, id);
    id
  };
  _ = goblin('G', (8, 3));
  _ = goblin('N', (9, 2));
  _ = goblin('T', (8, 5));

  world.startup();

  loop {
    instrument!("draw", world.draw(&mut terminal).unwrap());
    use terminal::Event::*;
    match terminal.poll() {
      Tick(_) => {
        world.tick += 1;
      }
      Input(char) => {
        if char == 'q' {
          break;
        }
        instrument!("world update", world.update(char));
      }
    }
  }
}
