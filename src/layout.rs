use crate::log;
use crate::Id;
use crate::Terminal;
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Debug, Copy, Clone)]
struct Position(i32, i32);

#[derive(Debug, Copy, Clone)]
struct Constraints {
  width: (i32, i32),
  height: (i32, i32),
}

impl Constraints {
  fn clamped_geometry(&self, width: i32, height: i32) -> Geometry {
    Geometry {
      width: width.max(self.width.0).min(self.width.1),
      height: height.max(self.height.0).min(self.height.1),
    }
  }
}

#[derive(Debug, Copy, Clone)]
struct Geometry {
  width: i32,
  height: i32,
}

#[derive(Debug)]
enum Widget<'a> {
  Fill(char),
  Padding(i32, i32, i32, i32),
  Row(),
  Column(),
  Flex(),
  ExpandWidth(),
  ExpandHeight(),
  FixedWidth(i32),
  FixedHeight(i32),
  Text(Cow<'a, str>),
  Viewport(),
}

#[derive(Default)]
pub struct WidgetTree<'a> {
  root_id: Option<Id>,
  widget: HashMap<Id, Widget<'a>>,
  parent: HashMap<Id, Id>,
  children: HashMap<Id, Vec<Id>>,
  position: HashMap<Id, Position>,
  geometry: HashMap<Id, Geometry>,
}

impl<'a> WidgetTree<'a> {
  pub fn update(&mut self, widgets: WidgetFn<'a>) {
    self.widget.clear();
    self.parent.clear();
    self.children.clear();
    self.position.clear();
    self.geometry.clear();
    self.root_id = Some(widgets(self));
  }

  pub fn layout(&mut self, dimensions: (i32, i32)) -> bool {
    let Some(root_id) = self.root_id else {
      return false;
    };
    self.layout_widget(
      root_id,
      Constraints {
        width: (dimensions.0, dimensions.0),
        height: (dimensions.1, dimensions.1),
      },
    );
    true
  }

  pub fn draw(&mut self, terminal: &mut Terminal) {
    let root_id = self.root_id.expect("root id should be set during draw");
    self.draw_widget(terminal, Position(0, 0), root_id);
  }

  pub fn get_global_position(&self, id: Id) -> Option<(i32, i32)> {
    let mut global = (0, 0);
    let mut parent_id = Some(&id);
    while let Some(id) = parent_id {
      let local = self.position.get(id).map(|p| (p.0, p.1))?;
      global.0 += local.0;
      global.1 += local.1;
      parent_id = self.parent.get(id);
    }
    Some(global)
  }

  pub fn get_geometry(&self, id: Id) -> Option<(i32, i32)> {
    self.geometry.get(&id).map(|g| (g.width, g.height))
  }

  fn layout_widget(&mut self, id: Id, constraints: Constraints) -> Geometry {
    let widget = self
      .widget
      .get(&id)
      .expect("widget with id should exist during layout");
    let geometry = match widget {
      Widget::Fill(_) => {
        let child_id = *self.get_single_child_id(id);
        let child_geometry = self.layout_widget(child_id, constraints);
        constraints.clamped_geometry(child_geometry.width, child_geometry.height)
      }
      Widget::Padding(left, right, top, bottom) => {
        let child_id = *self.get_single_child_id(id);
        self.position.insert(child_id, Position(*left, *top));
        let width_total = left + right;
        let height_total = top + bottom;
        let child_geometry = self.layout_widget(
          child_id,
          Constraints {
            width: (
              constraints.width.0,
              (constraints.width.1 - width_total).max(0),
            ),
            height: (
              constraints.height.0,
              (constraints.height.1 - height_total).max(0),
            ),
          },
        );
        let child_geometry = constraints.clamped_geometry(
          child_geometry.width + width_total,
          child_geometry.height + height_total,
        );
        constraints.clamped_geometry(child_geometry.width, child_geometry.height)
      }
      Widget::Row() => {
        let mut height = constraints.height.0;
        let mut width = 0;
        let mut flexible_child_ids = vec![];
        let children = self.get_multi_child_ids(id);
        for child_id in children.iter() {
          let child_widget = self
            .widget
            .get(child_id)
            .expect("child widget with id should exist during layout");
          if let Widget::Flex() = child_widget {
            flexible_child_ids.push(child_id);
            continue;
          }
          let child_geometry = self.layout_widget(
            *child_id,
            Constraints {
              width: (0, constraints.width.1 - width),
              height: constraints.height,
            },
          );
          height = height.max(child_geometry.height);
          width += child_geometry.width;
        }
        if !flexible_child_ids.is_empty() {
          let remaining_size = constraints.width.1 - width;
          let flex_child_count = flexible_child_ids.len();
          let size = remaining_size / flex_child_count as i32;
          for child_id in flexible_child_ids.into_iter() {
            let child_geometry = self.layout_widget(
              *child_id,
              Constraints {
                width: (0, size),
                height: constraints.height,
              },
            );
            height = height.max(child_geometry.height);
          }
        }
        width = 0;
        for child_id in children.iter() {
          let child_geometry = self
            .geometry
            .get(child_id)
            .expect("child geometry should exist during layout");
          self.position.insert(*child_id, Position(width, 0));
          width += child_geometry.width;
        }
        constraints.clamped_geometry(width, height)
      }
      Widget::Column() => {
        let mut width = constraints.width.0;
        let mut height = 0;
        let mut flexible_child_ids = vec![];
        let children = self.get_multi_child_ids(id);
        for child_id in children.iter() {
          let child_widget = self
            .widget
            .get(child_id)
            .expect("child widget with id should exist during layout");
          if let Widget::Flex() = child_widget {
            flexible_child_ids.push(child_id);
            continue;
          }
          let child_geometry = self.layout_widget(
            *child_id,
            Constraints {
              width: constraints.width,
              height: (0, constraints.height.1 - height),
            },
          );
          width = width.max(child_geometry.width);
          height += child_geometry.height;
        }
        if !flexible_child_ids.is_empty() {
          let remaining_size = constraints.height.1 - height;
          let flex_child_count = flexible_child_ids.len();
          let size = remaining_size / flex_child_count as i32;
          for child_id in flexible_child_ids.into_iter() {
            let child_geometry = self.layout_widget(
              *child_id,
              Constraints {
                width: constraints.width,
                height: (0, size),
              },
            );
            width = width.max(child_geometry.width);
          }
        }
        height = 0;
        for child_id in children.iter() {
          let child_geometry = self
            .geometry
            .get(child_id)
            .expect("child geometry should exist during layout");
          self.position.insert(*child_id, Position(0, height));
          height += child_geometry.height;
        }
        constraints.clamped_geometry(width, height)
      }
      Widget::Flex() => {
        let child_id = *self.get_single_child_id(id);
        self.position.insert(child_id, Position(0, 0));
        let child_geometry = self.layout_widget(child_id, constraints);
        constraints.clamped_geometry(child_geometry.width, child_geometry.height)
      }
      Widget::ExpandWidth() => {
        let child_id = *self.get_single_child_id(id);
        self.position.insert(child_id, Position(0, 0));
        let child_geometry = self.layout_widget(
          child_id,
          Constraints {
            width: (constraints.width.1, constraints.width.1),
            height: constraints.height,
          },
        );
        constraints.clamped_geometry(child_geometry.width, child_geometry.height)
      }
      Widget::ExpandHeight() => {
        let child_id = *self.get_single_child_id(id);
        self.position.insert(child_id, Position(0, 0));
        let child_geometry = self.layout_widget(
          child_id,
          Constraints {
            width: constraints.width,
            height: (constraints.height.1, constraints.height.1),
          },
        );
        constraints.clamped_geometry(child_geometry.width, child_geometry.height)
      }
      Widget::FixedWidth(size) => {
        let size = (*size).max(constraints.width.0).min(constraints.width.1);
        let child_id = *self.get_single_child_id(id);
        self.position.insert(child_id, Position(0, 0));
        let child_geometry = self.layout_widget(
          child_id,
          Constraints {
            width: (size, size),
            height: constraints.height,
          },
        );
        constraints.clamped_geometry(child_geometry.width, child_geometry.height)
      }
      Widget::FixedHeight(size) => {
        let size = (*size).max(constraints.height.0).min(constraints.height.1);
        let child_id = *self.get_single_child_id(id);
        self.position.insert(child_id, Position(0, 0));
        let child_geometry = self.layout_widget(
          child_id,
          Constraints {
            width: constraints.width,
            height: (size, size),
          },
        );
        constraints.clamped_geometry(child_geometry.width, child_geometry.height)
      }
      Widget::Text(value) => {
        fn div_ceil(a: i32, b: i32) -> i32 {
          if b == 0 {
            return 0;
          }
          (a + (b - 1)) / b
        }
        let characters = value.len() as i32;
        let lines = div_ceil(characters, constraints.width.1);
        constraints.clamped_geometry(characters, lines)
      }
      Widget::Viewport() => Geometry {
        width: constraints.width.0,
        height: constraints.height.0,
      },
    };
    self.geometry.insert(id, geometry);
    geometry
  }

  fn get_single_child_id(&self, id: Id) -> &Id {
    let children = self
      .children
      .get(&id)
      .expect("single child widget had no children");
    if children.len() > 1 {
      panic!("single child widget had more than one child");
    }
    children
      .first()
      .expect("single child widget had zero children")
  }

  fn get_multi_child_ids(&self, id: Id) -> Vec<Id> {
    self.children.get(&id).cloned().unwrap_or_else(Vec::new)
  }

  fn draw_widget(&self, terminal: &mut Terminal, parent_position: Position, id: Id) {
    let position = {
      let p = self
        .position
        .get(&id)
        .expect("widget with id should have a position during draw");
      Position(p.0 + parent_position.0, p.1 + parent_position.1)
    };
    let widget = self
      .widget
      .get(&id)
      .expect("widget with id should exist during draw");
    let geometry = self
      .geometry
      .get(&id)
      .expect("widget should have a geometry during draw");
    log!("LAYOUT", "drawing widget", id, widget, position, geometry);
    match widget {
      Widget::Fill(char) => {
        for column in 0..geometry.width {
          for row in 0..geometry.height {
            terminal.set((position.0 + column, position.1 + row), *char);
          }
        }
        self.draw_child(terminal, position, id);
      }
      Widget::Padding(_, _, _, _) => self.draw_child(terminal, position, id),
      Widget::ExpandWidth() => self.draw_child(terminal, position, id),
      Widget::ExpandHeight() => self.draw_child(terminal, position, id),
      Widget::FixedWidth(_) => self.draw_child(terminal, position, id),
      Widget::FixedHeight(_) => self.draw_child(terminal, position, id),
      Widget::Row() => self.draw_children(terminal, position, id),
      Widget::Column() => self.draw_children(terminal, position, id),
      Widget::Flex() => self.draw_child(terminal, position, id),
      Widget::Text(value) => {
        let mut chars = value.chars();
        for row in 0..geometry.height {
          for column in 0..geometry.width {
            let char = chars.next().unwrap_or(' ');
            terminal.set((position.0 + column, position.1 + row), char);
          }
        }
      }
      Widget::Viewport() => {}
    };
  }

  fn draw_child(&self, terminal: &mut Terminal, position: Position, id: Id) {
    let child_id = *self.get_single_child_id(id);
    self.draw_widget(terminal, position, child_id);
  }

  fn draw_children(&self, terminal: &mut Terminal, position: Position, id: Id) {
    let children = self.get_multi_child_ids(id);
    for child_id in children.iter() {
      self.draw_widget(terminal, position, *child_id);
    }
  }
}

pub type WidgetFn<'a> = Box<dyn FnOnce(&mut WidgetTree<'a>) -> Id + 'a>;

pub fn fill(char: char, child: WidgetFn) -> WidgetFn {
  Box::new(move |tree: &mut WidgetTree| {
    let id = Id::new();
    tree.position.insert(id, Position(0, 0));
    tree.widget.insert(id, Widget::Fill(char));
    let child_id = child(tree);
    tree.parent.insert(child_id, id);
    tree
      .children
      .entry(id)
      .or_insert_with(|| Vec::with_capacity(1))
      .push(child_id);
    id
  })
}

pub fn padding(values: (i32, i32, i32, i32), child: WidgetFn) -> WidgetFn {
  Box::new(move |tree: &mut WidgetTree| {
    let id = Id::new();
    tree.position.insert(id, Position(0, 0));
    tree
      .widget
      .insert(id, Widget::Padding(values.0, values.1, values.2, values.3));
    let child_id = child(tree);
    tree.parent.insert(child_id, id);
    tree
      .children
      .entry(id)
      .or_insert_with(|| Vec::with_capacity(1))
      .push(child_id);
    id
  })
}

pub fn expand_width(child: WidgetFn) -> WidgetFn {
  Box::new(move |tree: &mut WidgetTree| {
    let id = Id::new();
    tree.position.insert(id, Position(0, 0));
    tree.widget.insert(id, Widget::ExpandWidth());
    let child_id = child(tree);
    tree.parent.insert(child_id, id);
    tree
      .children
      .entry(id)
      .or_insert_with(|| Vec::with_capacity(1))
      .push(child_id);
    id
  })
}

pub fn expand_height(child: WidgetFn) -> WidgetFn {
  Box::new(move |tree: &mut WidgetTree| {
    let id = Id::new();
    tree.position.insert(id, Position(0, 0));
    tree.widget.insert(id, Widget::ExpandHeight());
    let child_id = child(tree);
    tree.parent.insert(child_id, id);
    tree
      .children
      .entry(id)
      .or_insert_with(|| Vec::with_capacity(1))
      .push(child_id);
    id
  })
}

pub fn fixed_width(size: i32, child: WidgetFn) -> WidgetFn {
  Box::new(move |tree: &mut WidgetTree| {
    let id = Id::new();
    tree.position.insert(id, Position(0, 0));
    tree.widget.insert(id, Widget::FixedWidth(size));
    let child_id = child(tree);
    tree.parent.insert(child_id, id);
    tree
      .children
      .entry(id)
      .or_insert_with(|| Vec::with_capacity(1))
      .push(child_id);
    id
  })
}

pub fn fixed_height(size: i32, child: WidgetFn) -> WidgetFn {
  Box::new(move |tree: &mut WidgetTree| {
    let id = Id::new();
    tree.position.insert(id, Position(0, 0));
    tree.widget.insert(id, Widget::FixedHeight(size));
    let child_id = child(tree);
    tree.parent.insert(child_id, id);
    tree
      .children
      .entry(id)
      .or_insert_with(|| Vec::with_capacity(1))
      .push(child_id);
    id
  })
}

pub fn row(children: Vec<WidgetFn>) -> WidgetFn {
  Box::new(move |tree: &mut WidgetTree| {
    let id = Id::new();
    tree.position.insert(id, Position(0, 0));
    tree.widget.insert(id, Widget::Row());
    let child_ids = children.into_iter().map(|f| f(tree)).collect::<Vec<Id>>();
    let child_count = child_ids.len();
    for child_id in child_ids {
      tree.parent.insert(child_id, id);
      tree
        .children
        .entry(id)
        .or_insert_with(|| Vec::with_capacity(child_count))
        .push(child_id);
    }
    id
  })
}

pub fn column(children: Vec<WidgetFn>) -> WidgetFn {
  Box::new(move |tree: &mut WidgetTree| {
    let id = Id::new();
    tree.position.insert(id, Position(0, 0));
    tree.widget.insert(id, Widget::Column());
    let child_ids = children.into_iter().map(|f| f(tree)).collect::<Vec<Id>>();
    let child_count = child_ids.len();
    for child_id in child_ids {
      tree.parent.insert(child_id, id);
      tree
        .children
        .entry(id)
        .or_insert_with(|| Vec::with_capacity(child_count))
        .push(child_id);
    }
    id
  })
}

pub fn flex(child: WidgetFn) -> WidgetFn {
  Box::new(move |tree: &mut WidgetTree| {
    let id = Id::new();
    tree.position.insert(id, Position(0, 0));
    tree.widget.insert(id, Widget::Flex());
    let child_id = child(tree);
    tree.parent.insert(child_id, id);
    tree
      .children
      .entry(id)
      .or_insert_with(|| Vec::with_capacity(1))
      .push(child_id);
    id
  })
}

pub fn text<'a>(value: impl Into<Cow<'a, str>> + 'a) -> WidgetFn<'a> {
  Box::new(move |tree: &mut WidgetTree| {
    let id = Id::new();
    tree.position.insert(id, Position(0, 0));
    tree.widget.insert(id, Widget::Text(value.into()));
    id
  })
}

pub fn viewport<'a>(id: Id) -> WidgetFn<'a> {
  Box::new(move |tree: &mut WidgetTree| {
    tree.position.insert(id, Position(0, 0));
    tree.widget.insert(id, Widget::Viewport());
    id
  })
}
