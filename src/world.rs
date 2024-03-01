use crate::*;
use std::fmt;
use std::mem;
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

#[derive(Debug, Default)]
pub enum Input {
  #[default]
  None,
  Some(char),
  Requested,
}

impl Input {
  pub fn is_requested(&self) -> bool {
    matches!(self, Self::Requested)
  }

  pub fn take_or_request(&mut self) -> Option<char> {
    match self {
      Self::None => {
        _ = mem::replace(self, Self::Requested);
        None
      }
      Self::Some(_) => {
        let Self::Some(char) = mem::replace(self, Self::None) else {
          unreachable!()
        };
        Some(char)
      },
      Self::Requested => None,
    }
  }
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
    self.update_fov();
    self.update_ui();
  }

  pub fn update(&mut self, input: char) {
    self.input = Input::Some(input);
    loop {
      if self.current_event.is_none() {
        let Some((time, event)) = self.timeline.pop() else {
          break;
        };
        self.time = time;
        self.current_event = Some(event);
      }
      log!("[EVENT] processing event", self.time, self.input, self.current_event);
      self.update_action();
      self.update_dead();
      self.update_turn();
      self.update_fov();
      if self.input.is_requested() {
        break;
      }
    }
    self.update_ui();
  }

  fn update_action(&mut self) {
    let current_event = self.current_event.take();
    let Some(Event::Action(id, action)) = current_event else {
      self.current_event = current_event;
      return;
    };
    if self.exists.get(&id).is_none() {
      self.current_event = None;
      return;
    }
    match action {
      Action::Move(vector) => self.update_move(id, vector),
      Action::Attack(vector) => self.update_attack(id, vector),
    }
    self.current_event = Some(Event::Turn(id));
  }

  fn update_move(&mut self, id: Id, vector: (i32, i32)) {
    let Some(position) = self.position.get(&id) else {
      return;
    };
    let position = (position.0 + vector.0, position.1 + vector.1);
    let mut is_blocked = false;
    if let Some(ids) = self.position.at(position) {
      for target_id in ids.iter() {
        if *target_id == id {
          continue;
        }
        if self.solidity.contains(target_id) {
          is_blocked = true;
          break;
        }
      }
    }
    if !is_blocked {
      self.position.insert(id, position);
    }
  }

  fn update_attack(&mut self, id: Id, vector: (i32, i32)) {
    let Some(position) = self.position.get(&id) else {
      return;
    };
    let position = (position.0 + vector.0, position.1 + vector.1);
    let Some(weapon) = self.weapon.get(&id) else {
      return;
    };
    let Some(ids) = self.position.at(position) else {
      return;
    };
    let mut target_id = None;
    for id in ids.iter() {
      if self.health.contains_key(id) {
        target_id = Some(*id);
        break;
      }
    }
    let Some(target_id) = target_id else {
      return;
    };
    let Some(health) = self.health.get_mut(&target_id) else {
      return;
    };
    *health = (*health - *weapon).max(0);
  }

  fn update_dead(&mut self) {
    let ids = self.health
      .iter()
      .filter(|(_, health)| **health <= 0)
      .map(|(id, _)| *id)
      .collect::<Vec<Id>>();
    for id in ids {
      self.update_dead_view_target(id);
      self.remove_entity(&id);
    }
  }

  fn update_dead_view_target(&mut self, id: Id) {
    if id != self.view_target {
      return;
    }
    let Some(position) = self.position.get(&id) else {
      return;
    };
    let id = Id::new();
    self.position.insert(id, *position);
    self.view_target = id;
  }

  fn update_turn(&mut self) {
    let Some(Event::Turn(id)) = self.current_event else {
      return;
    };
    if self.exists.get(&id).is_none() {
      return;
    }
    let turn_handlers = [
      Self::update_turn_controls,
      Self::update_turn_ai,
    ];
    for handler in turn_handlers {
      let completed = handler(self, id);
      if completed {
        self.current_event = None;
        break;
      }
    }
  }

  fn update_turn_controls(&mut self, id: Id) -> bool {
    let Some(controls) = self.controls.get(&id) else {
      return false;
    };
    let direction = match self.input.take_or_request() {
      Some(i) if i == controls.up => (0, -1),
      Some(i) if i == controls.down => (0, 1),
      Some(i) if i == controls.left => (-1, 0),
      Some(i) if i == controls.right => (1, 0),
      _ => return false,
    };
    let Some(position) = self.position.get(&id) else {
      return true;
    };
    let Some(speed) = self.speed.get(&id) else {
      return true;
    };
    let position = (position.0 + direction.0, position.1 + direction.1);
    let mut action = Action::Move(direction);
    if let Some(ids) = self.position.at(position) {
      for target_id in ids.iter() {
        if self.solidity.contains(target_id) {
          action = Action::Attack(direction);
          break;
        }
      }
    }
    self.timeline.push(self.time + *speed as usize, Event::Action(id, action));
    true
  }

  fn update_turn_ai(&mut self, id: Id) -> bool {
    let Some(ai) = self.ai.get(&id) else {
      return false;
    };
    let Some(position) = self.position.get(&id) else {
      return true;
    };
    let Some(speed) = self.speed.get(&id) else {
      return true;
    };
    let Some(target_position) = self.position.get(&ai.target) else {
      return true;
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
    let action = if desired_position == *target_position {
      Action::Attack(vector)
    } else {
      Action::Move(vector)
    };
    self.timeline.push(self.time + *speed as usize, Event::Action(id, action));
    true
  }

  fn update_fov(&mut self) {
    for (id, fov) in self.fov.iter_mut() {
      let Some(position) = self.position.get(id) else {
        fov.update(|_| false);
        return;
      };
      fov.update(|p| {
        let position = (position.0 + p.0, position.1 + p.1);
        let Some(ids) = self.position.at(position) else {
          return false;
        };
        for id_at in ids {
          if id_at == id {
            return false;
          }
          if self.opacity.contains(id_at) {
            return true;
          }
        }
        false
      });
    }
  }

  fn update_ui(&mut self) {
    let player_stats = {
      let target_id = self.view_target;
      fn stat(name: &'static str, value: Option<impl fmt::Display>) -> WidgetFn<'static> {
        let value = value.map(|v| v.to_string()).unwrap_or_else(|| "-".into());
        row(vec![
          text(name),
          flex(expand_width(text(" "))),
          text(value)
        ])
      }
      column(vec![
        stat("Name:", self.name.get(&target_id)),
        stat("Health:", self.health.get(&target_id)),
        stat("Speed:", self.speed.get(&target_id)),
      ])
    };
    let world_stats = {
      column(vec![
        row(vec![
            text("Time:"),
            flex(expand_width(text(" "))),
            text(self.time.to_string()),
        ]),
      ])
    };
    let timeline = {
      let format_event = |time: usize, event: &Event| {
        let (icon, description) = match event {
          Event::Turn(id) => {
            let Some(icon) = self.icon.get(id) else {
              return None;
            };
            (icon, "turn".to_string())
          },
          Event::Action(id, action) => {
            let Some(icon) = self.icon.get(id) else {
              return None;
            };
            let description = match action {
              Action::Move(v) => format!("move {},{}", v.0, v.1),
              Action::Attack(v) => format!("attack {},{}", v.0, v.1),
            };
            (icon, description)
          },
        };
        let time = time - self.time;
        Some((time.to_string(), icon.to_string(), description))
      };
      let current_event = match &self.current_event {
        Some(event) => format_event(self.time, event),
        None => None,
      };
      let mut events = vec![current_event];
      for (time, event) in self.timeline.iter() {
        events.push(format_event(time, event))
      }
      let entries = events
        .into_iter()
        .flatten()
        .map(|e| {
          row(vec![
            text(e.0),
            text(" "),
            text(e.1),
            text(" "),
            text(e.2),
          ])
        })
        .collect::<Vec<_>>();
      column(vec![
        text("Events:"),
        column(entries),
      ])
    };
    fn border(width: (i32, i32, i32, i32), child: WidgetFn) -> WidgetFn {
      fill(
        '┃',
        padding(
          (width.0, width.1, 0, 0),
          fill(
            '━',
            padding(
              (0, 0, width.2, width.3),
              fill(' ', child),
            ),
          ),
        )
      )
    }
    self.ui.update(row(vec![
      border(
        (0, 1, 0, 0),
        fixed_width(
          20,
          column(vec![
            player_stats,
            text(" "),
            world_stats,
            text(" "),
            expand_height(timeline),
          ]),
        ),
      ),
      expand_height(expand_width(viewport(self.viewport_id))),
    ]));
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
