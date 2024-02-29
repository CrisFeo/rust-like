use std::borrow::Cow;
use crate::terminal::Terminal;

struct Constraints {
  min_width: i32,
  max_width: i32,
  min_height: i32,
  max_height: i32,
}

impl Constraints {
  fn clamped_geometry(&self, width: i32, height: i32) -> Geometry {
    Geometry {
      width: width.max(self.min_width).min(self.max_width),
      height: height.max(self.min_height).min(self.max_height),
    }
  }
}

#[derive(Clone)]
struct Geometry {
  width: i32,
  height: i32,
}

trait Widget {
  fn set_position(&mut self, position: (i32, i32));

  fn layout(&mut self, constraints: Constraints) -> Geometry;

  fn render(&self, _terminal: &mut Terminal, _position: (i32, i32)) { }
}

struct Padding {
  position: (i32, i32),
  left: i32,
  right: i32,
  top: i32,
  bottom: i32,
  child: Box<dyn Widget>,
}

impl Widget for Padding {
  fn set_position(&mut self, position: (i32, i32)) {
    self.position = position;
  }

  fn layout(&mut self, constraints: Constraints) -> Geometry {
    self.child.set_position((self.left, self.top));
    let width_total = self.left + self.right;
    let height_total = self.top + self.bottom;
    let child_geometry = self.child.layout(Constraints {
      min_width: constraints.min_width,
      max_width: (constraints.max_width - width_total).max(0),
      min_height: constraints.min_height,
      max_height: (constraints.max_height - height_total).max(0),
    });
    constraints.clamped_geometry(
      child_geometry.width + width_total,
      child_geometry.height + height_total,
    )
  }

  fn render(&self, terminal: &mut Terminal, position: (i32, i32)) {
    let position = (position.0 + self.position.0, position.1 + self.position.1);
    self.child.render(terminal, position);
  }
}
struct FixedWidth {
  position: (i32, i32),
  size: i32,
  child: Box<dyn Widget>,
}

impl Widget for FixedWidth {
  fn set_position(&mut self, position: (i32, i32)) {
    self.position = position;
  }

  fn layout(&mut self, constraints: Constraints) -> Geometry {
    let size = self.size.max(constraints.min_width).min(constraints.max_width);
    self.child.set_position((0, 0));
    self.child.layout(Constraints {
      min_width: size,
      max_width: size,
      min_height: constraints.min_height,
      max_height: constraints.max_height,
    })
  }

  fn render(&self, terminal: &mut Terminal, position: (i32, i32)) {
    let position = (position.0 + self.position.0, position.1 + self.position.1);
    self.child.render(terminal, position);
  }
}

struct FixedHeight {
  position: (i32, i32),
  size: i32,
  child: Box<dyn Widget>,
}

impl Widget for FixedHeight {
  fn set_position(&mut self, position: (i32, i32)) {
    self.position = position;
  }

  fn layout(&mut self, constraints: Constraints) -> Geometry {
    let size = self.size.max(constraints.min_height).min(constraints.max_height);
    self.child.set_position((0, 0));
    self.child.layout(Constraints {
      min_width: constraints.min_width,
      max_width: constraints.max_width,
      min_height: size,
      max_height: size,
    })
  }

  fn render(&self, terminal: &mut Terminal, position: (i32, i32)) {
    let position = (position.0 + self.position.0, position.1 + self.position.1);
    self.child.render(terminal, position);
  }
}

struct Row {
  position: (i32, i32),
  children: Vec<Box<dyn Widget>>,
  child_sizes: Vec<i32>,
}

impl Widget for Row {
  fn set_position(&mut self, position: (i32, i32)) {
    self.position = position;
  }

  fn layout(&mut self, constraints: Constraints) -> Geometry {
    let mut height = constraints.min_height;
    let mut width = 0;
    self.child_sizes.clear();
    for child in self.children.iter_mut() {
      let child_geometry = child.layout(Constraints {
        min_width: 0,
        max_width: constraints.max_width - width,
        min_height: constraints.min_height,
        max_height: constraints.max_height,
      });
      child.set_position((width, 0));
      height = height.max(child_geometry.height);
      width += child_geometry.width;
      self.child_sizes.push(child_geometry.width);
    }
    constraints.clamped_geometry(width, height)
  }

  fn render(&self, terminal: &mut Terminal, position: (i32, i32)) {
    for child in self.children.iter() {
      let position = (position.0 + self.position.0, position.1 + self.position.1);
      child.render(terminal, position);
    }
  }
}

struct Column {
  position: (i32, i32),
  children: Vec<Box<dyn Widget>>,
  child_sizes: Vec<i32>,
}

impl Widget for Column {
  fn set_position(&mut self, position: (i32, i32)) {
    self.position = position;
  }

  fn layout(&mut self, constraints: Constraints) -> Geometry {
    let mut width = constraints.min_width;
    let mut height = 0;
    self.child_sizes.clear();
    for child in self.children.iter_mut() {
      let child_geometry = child.layout(Constraints {
        min_width: constraints.min_width,
        max_width: constraints.max_width,
        min_height: 0,
        max_height: constraints.max_height - height,
      });
      child.set_position((0, height));
      width = width.max(child_geometry.width);
      height += child_geometry.height;
      self.child_sizes.push(child_geometry.height);
    }
    constraints.clamped_geometry(width, height)
  }

  fn render(&self, terminal: &mut Terminal, position: (i32, i32)) {
    for child in self.children.iter() {
      let position = (position.0 + self.position.0, position.1 + self.position.1);
      child.render(terminal, position);
    }
  }
}

struct Text {
  position: (i32, i32),
  geometry: Geometry,
  value: Cow<'static, str>,
}

impl Widget for Text {
  fn set_position(&mut self, position: (i32, i32)) {
    self.position = position;
  }

  fn layout(&mut self, constraints: Constraints) -> Geometry {
    fn div_ceil(a: i32, b: i32) -> i32 {
      (a + (b - 1)) / b
    }
    let characters = self.value.len() as i32;
    let lines = div_ceil(characters, constraints.max_width);
    self.geometry = constraints.clamped_geometry(characters, lines);
    self.geometry.clone()
  }

  fn render(&self, terminal: &mut Terminal, position: (i32, i32)) {
    let position = (position.0 + self.position.0, position.1 + self.position.1);
    let mut column = 0;
    let mut row = 0;
    for char in self.value.chars() {
      if column == self.geometry.width {
        column = 0;
        row += 1;
      }
      if row > self.geometry.height {
        break;
      }
      let position = (position.0 + column, position.1 + row);
      terminal.set(position, char);
      column += 1;
    }
  }
}

fn padding(left: i32, right: i32, top: i32, bottom: i32, child: Box<dyn Widget>) -> Box<dyn Widget> {
  Box::new(Padding {
    position: (0, 0),
    left,
    right,
    top,
    bottom,
    child,
  })
}

fn fixed_width(size: i32, child: Box<dyn Widget>) -> Box<dyn Widget> {
  Box::new(FixedWidth {
    position: (0, 0),
    size,
    child,
  })
}

fn fixed_height(size: i32, child: Box<dyn Widget>) -> Box<dyn Widget> {
  Box::new(FixedHeight {
    position: (0, 0),
    size,
    child,
  })
}

fn row(children: Vec<Box<dyn Widget>>) -> Box<dyn Widget> {
  Box::new(Row {
    position: (0, 0),
    child_sizes: Vec::with_capacity(children.len()),
    children,
  })
}

fn column(children: Vec<Box<dyn Widget>>) -> Box<dyn Widget> {
  Box::new(Column {
    position: (0, 0),
    child_sizes: Vec::with_capacity(children.len()),
    children,
  })
}

fn text(value: impl Into<Cow<'static, str>>) -> Box<dyn Widget> {
  Box::new(Text {
    position: (0, 0),
    geometry: Geometry {
      width: 0,
      height: 0,
    },
    value: value.into(),
  })
}

pub fn scratch() {
  let mut tree = row(vec![
    padding(0, 1, 0, 0,
      fixed_width(20,
        column(vec![
          row(vec![
            text("Name:"),
            text("Joe Borgson the III"),
          ]),
          row(vec![
            text("Health:"),
            text("18/20"),
          ]),
          row(vec![
            text("Speed:"),
            text("13"),
          ]),
        ]),
      ),
    ),
    text("welcome friends and countrymen! to the game-of-games!"),
  ]);
  tree.layout(Constraints {
    min_width: 50,
    max_width: 50,
    min_height: 50,
    max_height: 50,
  });
  let mut t = Terminal::new().unwrap();
  tree.render(&mut t, (0, 0));
  t.present().unwrap();
  _ = t.read();
}

