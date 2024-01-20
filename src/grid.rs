pub type Point = (i32, i32);

pub struct LineIter {
  next: Point,
  remaining_steps: i32,
  test: i32,
  delta: Point,
  step: Point,
}

impl Iterator for LineIter {
  type Item = Point;
  
  fn next(&mut self) -> Option<Self::Item> {
    if self.remaining_steps < 0 {
      return None;
    }
    self.remaining_steps -= 1;
    let result = self.next;
    if self.delta.0 >= self.delta.1 {
      self.test -= self.delta.1;
      self.next.0 += self.step.0;
      if self.test < 0 {
        self.next.1 += self.step.1;
        self.test += self.delta.0;
      }
    } else {
      self.test -= self.delta.0;
      self.next.1 += self.step.1;
      if self.test < 0 {
        self.next.0 += self.step.0;
        self.test += self.delta.1;
      }
    }
    Some(result)
  }
}

pub fn line(a: Point, b: Point) -> LineIter {
  let slope = (b.0 - a.0, b.1 - a.1);
  let delta = (slope.0.abs(), slope.1.abs());
  let step = (slope.0.signum(), slope.1.signum());
  let remaining_steps = delta.0.max(delta.1);
  let test = ((step.1 - 1) / 2 + remaining_steps) / 2;
  LineIter {
    next: a,
    remaining_steps,
    test,
    delta,
    step,
  }
}

pub struct SpiralIter {
  next: Point,
  remaining_steps: u32,
  step: Point,
  steps_to_take_on_side: u32,
  steps_taken_on_side: u32,
  has_turned_at_radius: bool,
}

impl Iterator for SpiralIter {
  type Item = Point;
  
  fn next(&mut self) -> Option<Self::Item> {
    if self.remaining_steps == 0 {
      return None;
    }
    let result = self.next;
    self.next = (
      self.next.0 + self.step.0,
      self.next.1 + self.step.1
    );
    self.remaining_steps -= 1;
    self.steps_taken_on_side += 1;
    if self.steps_taken_on_side == self.steps_to_take_on_side {
      self.step = match self.step {
        (1, 0) => (0, 1),
        (0, 1) => (-1, 0),
        (-1, 0) => (0, -1),
        (0, -1) => (1, 0),
        _ => panic!("unexpected step encountered"),
      };
      if self.has_turned_at_radius {
        self.steps_to_take_on_side += 1;
      }
      self.has_turned_at_radius = !self.has_turned_at_radius;
      self.steps_taken_on_side = 0;
    }
    Some(result)
  }
}

pub fn spiral(point: Point, radius: i32) -> SpiralIter {
  SpiralIter {
    next: point,
    remaining_steps: (2 * radius as u32 + 1).pow(2),
    step: (1, 0),
    steps_to_take_on_side: 1,
    steps_taken_on_side: 0,
    has_turned_at_radius: false,
  }
}
