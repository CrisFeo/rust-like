use crate::*;

#[derive(Debug)]
pub enum Event {
  Turn(Id),
  Action(Id, Action),
}

#[derive(Debug)]
pub enum Action {
  Move((i32, i32)),
  Attack((i32, i32)),
}
