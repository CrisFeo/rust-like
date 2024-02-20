use std::collections::{
  BinaryHeap,
  HashMap,
  HashSet,
};
use crate::*;

#[derive(Debug)]
pub struct Ai {
  pub target: Id,
}

#[derive(Debug)]
pub struct Controls {
  pub up: char,
  pub down: char,
  pub left: char,
  pub right: char,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Layer {
  Map,
  Mob,
  //Ui,
}

pub struct World {
  pub time: usize,
  pub timeline: BinaryHeap<Event>,
  pub pending_action: Option<Box<dyn Action>>,
  pub input: Option<char>,
  pub view_target: Option<Id>,
  pub name: HashMap<Id, &'static str>,
  pub icon: HashMap<Id, char>,
  pub layer: HashMap<Id, Layer>,
  pub position: SpatialMap,
  pub solidity: HashSet<Id>,
  pub opacity: HashSet<Id>,
  pub ai: HashMap<Id, Ai>,
  pub controls: HashMap<Id, Controls>,
  pub speed: HashMap<Id, i32>,
  pub health: HashMap<Id, i32>,
  pub weapon: HashMap<Id, i32>,
  pub fov: HashMap<Id, FieldOfView>,
}

impl Default for World {
  fn default() -> Self {
    Self::new()
  }
}

impl World {
  pub fn new() -> Self {
    World {
      time: 0,
      timeline: BinaryHeap::new(),
      pending_action: None,
      input: None,
      view_target: None,
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
      fov: HashMap::new(),
    }
  }

  pub fn remove_entity(&mut self, id: &Id) {
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
    self.fov.remove(id);
  }

  pub fn update(&mut self, input: char) {
    self.input = Some(input);
    loop {
      if let Some(mut pending_action) = self.pending_action.take() {
        if !pending_action.run(self) {
          self.pending_action = Some(pending_action);
          self.update_fov();
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
  }

  pub fn update_fov(&mut self) {
    for (id, fov) in self.fov.iter_mut() {
      let Some(position) = self.position.get(id) else {
        fov.update(|_| { false });
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

  pub fn draw(&self, t: &mut Terminal) -> std::io::Result<()> {
    let mut order = self.layer
      .iter()
      .collect::<Vec<_>>();
    order.sort_by_key(|l| l.1);
    let (vision_origin, vision_fov) = if let Some(id) = self.view_target {
      (self.position.get(&id), self.fov.get(&id))
    } else {
      (None, None)
    };

    for (id, _) in order {
      let position = or_continue!(
        self.position.get(id)
      );
      let is_visible = if let Some(vision_origin) = vision_origin {
        let position = (position.0 - vision_origin.0, position.1 - vision_origin.1);
        if let Some(fov) = vision_fov {
          fov.is_visible(position)
        } else {
          true
        }
      } else {
        true
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
