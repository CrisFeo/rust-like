use crate::grid::{line, Point};
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
struct Node {
  point: Point,
  children: Vec<usize>,
}

pub struct VisibilityCache {
  nodes: Vec<Node>,
  paths: HashMap<Point, Vec<Point>>,
}

impl VisibilityCache {
  pub fn new(max_radius: i32) -> Self {
    let mut instance = Self {
      nodes: Vec::new(),
      paths: HashMap::new(),
    };
    instance.compute_nodes(max_radius);
    instance
  }

  fn compute_nodes(&mut self, radius: i32) {
    self.add_node((0, 0));
    self.compute_nodes_to(radius, (radius, radius));
    for i in 0..radius {
      self.compute_nodes_to(radius, (radius, i));
      self.compute_nodes_to(radius, (i, radius));
    }
  }

  fn compute_nodes_to(&mut self, radius: i32, destination: Point) {
    let mut current = 0;
    let current_point = self.get_node(current).point;
    for point in line(current_point, destination).skip(1) {
      let d = point.0 * point.0 + point.1 * point.1;
      let d = (d as f64).sqrt().floor() as i32;
      if d > radius {
        continue;
      }
      let mut next = None;
      self
        .paths
        .entry(point)
        .and_modify(|tos| tos.push(destination))
        .or_insert_with(|| vec![destination]);
      let current_children = &self.get_node(current).children;
      for child in current_children.iter() {
        let child_point = self.get_node(*child).point;
        if child_point == point {
          next = Some(child);
          break;
        }
      }
      if let Some(next) = next {
        current = *next;
      } else {
        let next = self.add_node(point);
        self.add_child(current, next);
        current = next;
      }
    }
  }

  fn add_node(&mut self, point: Point) -> usize {
    let index = self.nodes.len();
    self.nodes.push(Node {
      point,
      children: Vec::new(),
    });
    index
  }

  fn get_node(&self, index: usize) -> &Node {
    self.nodes.get(index).unwrap()
  }

  fn add_child(&mut self, index: usize, child_index: usize) {
    self
      .nodes
      .get_mut(index)
      .unwrap()
      .children
      .push(child_index)
  }
}

pub struct FieldOfView {
  cache: Rc<VisibilityCache>,
  lookup: HashMap<Point, usize>,
  generation: usize,
}

impl FieldOfView {
  pub fn new(cache: Rc<VisibilityCache>) -> Self {
    Self {
      cache,
      lookup: HashMap::new(),
      generation: 0,
    }
  }

  pub fn update<F>(&mut self, check_opaque: F)
  where
    F: Fn(Point) -> bool,
  {
    self.generation += 1;
    let quadrant_transforms = [
      |p: Point| (p.0, p.1),
      |p: Point| (-p.1, p.0),
      |p: Point| (-p.0, -p.1),
      |p: Point| (p.1, -p.0),
    ];
    let mut pending = Vec::new();
    for to_quadrant in quadrant_transforms {
      pending.clear();
      pending.push(0);
      while let Some(current) = pending.pop() {
        let node = self.cache.get_node(current);
        let point = to_quadrant(node.point);
        if !check_opaque(point) {
          for child in node.children.iter() {
            pending.push(*child);
          }
        }
        self.lookup.insert(point, self.generation);
      }
    }
  }

  pub fn is_visible(&self, point: Point) -> bool {
    let Some(generation) = self.lookup.get(&point) else {
      return false;
    };
    *generation == self.generation
  }
}
