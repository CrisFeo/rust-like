use crate::*;
use std::cmp::Reverse;
use std::io;

#[derive(Debug)]
pub struct Controls {
  pub up: char,
  pub down: char,
  pub left: char,
  pub right: char,
  pub wait: char,
}

#[derive(Debug)]
pub struct Ai {
  pub target: Id,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Layer {
  Map,
  Mob,
}

#[derive(Default)]
pub struct World {
  pub input: Input,
  pub ui: WidgetTree<'static>,
  pub viewport_id: Id,
  pub tick: usize,
  pub time: usize,
  pub timeline: Timeline<Event>,
  pub current_event: Option<Event>,
  pub view_target: Id,
  pub exists: Is<Id>,
  pub name: HasOne<Id, &'static str>,
  pub icon: HasOne<Id, char>,
  pub layer: HasOne<Id, Layer>,
  pub position: ManyToOne<Id, (i32, i32)>,
  pub solidity: Is<Id>,
  pub opacity: Is<Id>,
  pub controls: HasOne<Id, Controls>,
  pub ai: HasOne<Id, Ai>,
  pub health: HasOne<Id, i32>,
  pub fov: HasOne<Id, FieldOfView>,
  pub held_by: ManyToOne<Id, Id>,
  pub provides_activity: HasMany<Id, Activity>,
}

impl World {
  pub fn remove_entity(&mut self, id: &Id) {
    self.exists.remove(id);
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
    update_ui(self);
  }

  pub fn update(&mut self, input: char) {
    self.input = Input::Some(input);
    loop {
      update_timeline(self);
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
      update_current_event(self);
      update_dead_entities(self);
      update_fov(self);
      if self.input.is_requested() {
        break;
      }
    }
    update_ui(self);
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
    let target = self
      .position
      .get_right(&self.view_target)
      .unwrap_or(&(0, 0));
    let to_screen = ((-size.0 / 2) + target.0, (-size.1 / 2) + target.1);
    for column in 0..size.0 {
      for row in 0..size.1 {
        let world = (to_screen.0 + column, to_screen.1 + row);
        let mut is_visible = true;
        if let Some(fov) = self.fov.get(&self.view_target) {
          let vision = (world.0 - target.0, world.1 - target.1);
          is_visible = fov.is_visible(vision);
        }
        let char = match self.position.get_lefts(&world) {
          None => ' ',
          Some(ids) => {
            if is_visible {
              let mut ids = ids.iter().collect::<Vec<_>>();
              ids.sort_by_key(|id| Reverse(self.layer.get(id)));
              *ids.first().and_then(|id| self.icon.get(id)).unwrap_or(&' ')
            } else {
              '~'
            }
          }
        };
        let screen = (offset.0 + column, offset.1 + row);
        terminal.set(screen, char);
      }
    }
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
