use crate::grid::Point;
use std::collections::HashMap;

#[derive(Default)]
pub struct Navigation {
  cells: HashMap<Point, usize>,
}

impl Navigation {
  pub fn reset(&mut self) {
    self.cells.clear();
  }

  pub fn set_value(&mut self, point: Point, value: usize) {
    self.cells.insert(point, value);
  }

  pub fn get_value(&self, point: Point) -> Option<usize> {
    self.cells.get(&point).copied()
  }

  pub fn calculate(&mut self) {
    let mut changed = true;
    while changed {
      changed = false;
      let points = self.cells.keys().copied().collect::<Vec<_>>();
      for point in points {
        let Some((_, smallest_value)) = self.best_neighbor(point) else {
          continue;
        };
        let Some(value) = self.cells.get_mut(&point) else {
          continue;
        };
        if smallest_value < value.saturating_sub(1) {
          changed = true;
          *value = smallest_value.saturating_add(1);
        }
      }
    }
  }

  pub fn best_neighbor(&self, point: Point) -> Option<(Point, usize)> {
    let neighbors = [
      (point.0 - 1, point.1),
      (point.0 + 1, point.1),
      (point.0, point.1 - 1),
      (point.0, point.1 + 1),
    ];
    neighbors
      .into_iter()
      .filter_map(|p| self.cells.get(&p).map(|v| (p, *v)))
      .filter(|(_, v)| *v != usize::MAX)
      .min_by(|a, b| a.1.cmp(&b.1))
  }
}
