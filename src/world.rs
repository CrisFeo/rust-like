use crate::*;
use std::cmp::Reverse;
use std::io;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Layer {
  Map,
  Mob,
}

#[derive(Default)]
pub enum ViewType {
  #[default]
  Normal,
  Navigation,
  Revealed,
}

#[derive(Default)]
pub struct World {
  pub input: Input,
  pub ui: WidgetTree<'static>,
  pub viewport_id: Id,
  pub view_type: ViewType,
  pub tick: usize,
  pub time: usize,
  pub timeline: Timeline<Event>,
  pub auto_step: Option<usize>,
  pub current_event: Option<Event>,
  pub view_target: Id,
  pub name: HasOne<Id, &'static str>,
  pub icon: HasOne<Id, char>,
  pub layer: HasOne<Id, Layer>,
  pub position: ManyToOne<Id, (i32, i32)>,
  pub solidity: Is<Id>,
  pub opacity: Is<Id>,
  pub controls: HasOne<Id, Controls>,
  pub ai: HasOne<Id, Ai>,
  pub navigation: Navigation,
  pub health: HasOne<Id, i32>,
  pub fov: HasOne<Id, FieldOfView>,
  pub held_by: ManyToOne<Id, Id>,
  pub provides_activity: HasMany<Id, Activity>,
}

impl World {
  pub fn remove_entity(&mut self, id: &Id) {
    self.name.remove(id);
    self.icon.remove(id);
    self.layer.remove(id);
    self.position.remove_by_left(id);
    self.solidity.remove(id);
    self.opacity.remove(id);
    self.controls.remove(id);
    self.ai.remove(id);
    self.health.remove(id);
    self.fov.remove(id);
    self.held_by.remove_by_left(id);
    self.held_by.remove_by_right(id);
    self.provides_activity.remove_by_left(id);
  }

  pub fn startup(&mut self) {
    update_fov(self);
    update_navigation(self);
    update_timeline(self);
    update_ui(self);
  }

  pub fn update(&mut self, input: char) {
    self.input = Input::Some(input);
    instrument!("update_view_type", update_view_type(self));
    let last_input_time = self.time;
    loop {
      instrument!("update_timeline", update_timeline(self));
      if self.current_event.is_none() {
        break;
      }
      log!(
        "EVENT",
        "processing current event",
        self.time,
        self.input,
        self.current_event,
      );
      instrument!("update_current_event", update_current_event(self));
      instrument!("update_dead_entities", update_dead_entities(self));
      instrument!("update_fov", update_fov(self));
      instrument!("update_navigation", update_navigation(self));
      if self.input.is_requested() {
        break;
      }
      if let Some(interval) = self.auto_step {
        let elapsed_time = self.time - last_input_time;
        if elapsed_time > interval {
          break;
        }
      }
    }
    instrument!("update_ui", update_ui(self));
  }

  pub fn draw(&mut self, terminal: &mut Terminal) -> io::Result<()> {
    let dimensions = terminal.dimensions()?;
    if self.ui.layout(dimensions) {
      let viewport_position = self.ui.get_global_position(self.viewport_id).unwrap();
      let viewport_geometry = self.ui.get_geometry(self.viewport_id).unwrap();
      self.draw_viewport(terminal, viewport_position, viewport_geometry);
      self.ui.draw(terminal);
    }
    terminal.present()
  }

  fn draw_viewport(&self, terminal: &mut Terminal, offset: (i32, i32), size: (i32, i32)) {
    let view_position = self
      .position
      .get_right(&self.view_target)
      .unwrap_or(&(0, 0));
    let to_screen = ((-size.0 / 2) + view_position.0, (-size.1 / 2) + view_position.1);
    for column in 0..size.0 {
      for row in 0..size.1 {
        let cell_position = (to_screen.0 + column, to_screen.1 + row);
        let char = match self.view_type {
          ViewType::Normal => draw_normal_cell(self, *view_position, cell_position),
          ViewType::Revealed => draw_revealed_cell(self, *view_position, cell_position),
          ViewType::Navigation => draw_navigation_cell(self, *view_position, cell_position),
        };
        let char = char.unwrap_or(' ');
        let screen = (offset.0 + column, offset.1 + row);
        terminal.set(screen, char);
      }
    }
  }
}

fn draw_normal_cell(
  world: &World,
  view_position: (i32, i32),
  cell_position: (i32, i32),
) -> Option<char> {
  if let Some(fov) = world.fov.get(&world.view_target) {
    let vision = (cell_position.0 - view_position.0, cell_position.1 - view_position.1);
    if !fov.is_visible(vision) {
      return Some('~');
    }
  }
  let ids = world.position.get_lefts(&cell_position)?;
  let mut ids = ids.iter().collect::<Vec<_>>();
  ids.sort_by_key(|id| Reverse(world.layer.get(id)));
  let id = ids.first()?;
  world.icon.get(id).copied()
}

fn draw_revealed_cell(
  world: &World,
  view_position: (i32, i32),
  cell_position: (i32, i32),
) -> Option<char> {
  let ids = world.position.get_lefts(&cell_position)?;
  let mut ids = ids.iter().collect::<Vec<_>>();
  ids.sort_by_key(|id| Reverse(world.layer.get(id)));
  let id = ids.first()?;
  world.icon.get(id).copied()
}

fn draw_navigation_cell(
  world: &World,
  _view_position: (i32, i32),
  cell_position: (i32, i32),
) -> Option<char> {
  let value = world.navigation.get_value(cell_position)?;
  let char = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().nth(value);
  match char {
    Some(char) => Some(char),
    None => Some('!'),
  }
}

fn update_timeline(world: &mut World) {
  if world.current_event.is_some() {
    return;
  }
  let Some((time, event)) = world.timeline.pop() else {
    return;
  };
  world.time = time;
  world.current_event = Some(event);
}

fn update_view_type(world: &mut World) {
  if world.input.try_consume('1') {
    world.view_type = ViewType::Normal;
  } else if world.input.try_consume('2') {
    world.view_type = ViewType::Navigation;
  } else if world.input.try_consume('3') {
    world.view_type = ViewType::Revealed;
  }
}

fn update_dead_entities(world: &mut World) {
  let ids = world
    .health
    .iter()
    .filter(|(_, health)| **health <= 0)
    .map(|(id, _)| *id)
    .collect::<Vec<Id>>();
  for id in ids {
    update_dead_view_target(world, id);
    world.remove_entity(&id);
  }
}

fn update_dead_view_target(world: &mut World, id: Id) {
  if id != world.view_target {
    return;
  }
  let Some(position) = world.position.get_right(&id) else {
    return;
  };
  let id = Id::new();
  world.position.insert(id, *position);
  world.auto_step = Some(10);
  world.view_target = id;
}

fn update_fov(world: &mut World) {
  for (id, fov) in world.fov.iter_mut() {
    let Some(position) = world.position.get_right(id) else {
      fov.update(|_| false);
      return;
    };
    fov.update(|p| {
      let position = (position.0 + p.0, position.1 + p.1);
      let Some(ids) = world.position.get_lefts(&position) else {
        return false;
      };
      for id_at in ids {
        if id_at == id {
          return false;
        }
        if world.opacity.contains(id_at) {
          return true;
        }
      }
      false
    });
  }
}

fn update_navigation(world: &mut World) {
  world.navigation.reset();
  if let Some(position) = world.position.get_right(&world.view_target) {
    if world.health.contains_key(&world.view_target) {
      world.navigation.set_value(*position, 0);
    }
  }
  world.navigation.calculate();
}

pub fn can_see(world: &World, a: Id, b: Id) -> Option<bool> {
  let fov = world.fov.get(&a)?;
  let a = world.position.get_right(&a)?;
  let b = world.position.get_right(&b)?;
  Some(fov.is_visible((b.0 - a.0, b.1 - a.1)))
}
