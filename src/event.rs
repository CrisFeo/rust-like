use crate::*;

#[derive(Debug)]
pub enum Event {
  Turn(Id, TurnType),
}

pub fn update_current_event(world: &mut World) {
  let Some(current_event) = world.current_event.take() else {
    return;
  };
  match current_event {
    Event::Turn(id, turn) => turn.update(world, id),
  };
}
