use crate::*;

#[derive(Debug)]
pub enum Event {
  //Action(Id, Action),
  Turn(Id, Turn),
}

pub fn update_current_event(world: &mut World) {
  let Some(current_event) = world.current_event.take() else {
    return;
  };
  match current_event {
    //Event::Action(id, action) => update_action(world, id, action),
    Event::Turn(id, turn) => update_turn(world, id, turn),
  };
}
