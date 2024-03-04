use crate::*;
use std::io;
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct Controls {
  pub up: char,
  pub down: char,
  pub left: char,
  pub right: char,
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
  pub time: usize,
  pub timeline: Timeline<Event>,
  pub current_event: Option<Event>,
  pub view_target: Id,
  pub exists: HashSet<Id>,
  pub name: HashMap<Id, &'static str>,
  pub icon: HashMap<Id, char>,
  pub layer: HashMap<Id, Layer>,
  pub position: SpatialMap,
  pub solidity: HashSet<Id>,
  pub opacity: HashSet<Id>,
  pub controls: HashMap<Id, Controls>,
  pub ai: HashMap<Id, Ai>,
  pub speed: HashMap<Id, i32>,
  pub health: HashMap<Id, i32>,
  pub weapon: HashMap<Id, i32>,
  pub fov: HashMap<Id, FieldOfView>,
}

impl World {
  pub fn remove_entity(&mut self, id: &Id) {
    self.exists.remove(id);
    self.name.remove(id);
    self.icon.remove(id);
    self.layer.remove(id);
    self.position.remove(id);
    self.solidity.remove(id);
    self.opacity.remove(id);
    self.controls.remove(id);
    self.ai.remove(id);
    self.speed.remove(id);
    self.health.remove(id);
    self.weapon.remove(id);
    self.fov.remove(id);
  }

  pub fn startup(&mut self) {
    update_fov(self);
    update_ui(self);
  }

  pub fn update(&mut self, input: char) {
    self.input = Input::Some(input);
    loop {
      update_current_event(self);
      if self.current_event.is_none() {
        break;
      }
      log!(
        "[EVENT] processing current event",
        self.time,
        self.input,
        self.current_event,
      );
      update_action_event(self);
      update_dead_entities(self);
      update_turn_event(self);
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
      self.render_viewport(terminal, viewport_position, viewport_geometry);
      self.ui.render(terminal);
    }
    terminal.present()
  }

  fn render_viewport(&self, terminal: &mut Terminal, offset: (i32, i32), size: (i32, i32)) {
    let target = self.position.get(&self.view_target).unwrap_or(&(0, 0));
    let to_screen = ((-size.0 / 2) + target.0, (-size.1 / 2) + target.1);
    for column in 0..size.0 {
      for row in 0..size.1 {
        let world = (to_screen.0 + column, to_screen.1 + row);
        let mut is_visible = true;
        if let Some(fov) = self.fov.get(&self.view_target) {
          let vision = (world.0 - target.0, world.1 - target.1);
          is_visible = fov.is_visible(vision);
        }
        let char = match self.position.at(world) {
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

fn update_current_event(world: &mut World) {
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
  let ids = world.health
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
  let Some(position) = world.position.get(&id) else {
    return;
  };
  let id = Id::new();
  world.position.insert(id, *position);
  world.view_target = id;
}

fn update_fov(world: &mut World) {
  for (id, fov) in world.fov.iter_mut() {
    let Some(position) = world.position.get(id) else {
      fov.update(|_| false);
      return;
    };
    fov.update(|p| {
      let position = (position.0 + p.0, position.1 + p.1);
      let Some(ids) = world.position.at(position) else {
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
