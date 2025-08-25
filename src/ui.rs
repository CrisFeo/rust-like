use crate::*;
use std::fmt;

pub fn update_ui(world: &mut World) {
  let ui = row(vec![
    border(
      (0, 1, 0, 0),
      fixed_width(
        20,
        column(vec![
          player_stats(world),
          text(" "),
          world_stats(world),
          text(" "),
          expand_height(timeline(world)),
        ]),
      ),
    ),
    flex(expand_width(expand_height(viewport(world.viewport_id)))),
    turn_controls(world),
  ]);
  world.ui.update(ui);
}

fn border(width: (i32, i32, i32, i32), child: WidgetFn) -> WidgetFn {
  fill(
    '┃',
    padding(
      (width.0, width.1, 0, 0),
      fill('━', padding((0, 0, width.2, width.3), fill(' ', child))),
    ),
  )
}

fn player_stats(world: &World) -> WidgetFn<'static> {
  let target_id = world.view_target;
  fn stat(name: &'static str, value: Option<impl fmt::Display>) -> WidgetFn<'static> {
    let value = value.map(|v| v.to_string()).unwrap_or_else(|| "-".into());
    row(vec![text(name), flex(expand_width(text(" "))), text(value)])
  }
  column(vec![
    stat("Name:", world.name.get(&target_id)),
    stat("Health:", world.health.get(&target_id)),
  ])
}

fn world_stats(world: &World) -> WidgetFn<'static> {
  column(vec![row(vec![
    text("Time:"),
    flex(expand_width(text(" "))),
    text(world.time.to_string()),
  ])])
}

fn timeline(world: &World) -> WidgetFn<'static> {
  let format_event = |time: usize, event: &Event| {
    let (icon, description) = match event {
      Event::Turn(id, turn) => {
        let icon = world.icon.get(id)?;
        let description = match turn {
          TurnType::Player(_) => "turn".to_string(),
          TurnType::Ai(ai) => format_action_description(&ai.pending_action),
        };
        (icon, description)
      } //      Event::Action(id, action) => {
        //        let Some(icon) = world.icon.get(id) else {
        //          return None;
        //        };
        //        let description = format_action_description(action);
        //        (icon, description)
        //      }
    };
    let time = time - world.time;
    Some((time.to_string(), icon.to_string(), description))
  };
  let current_event = match &world.current_event {
    Some(event) => format_event(world.time, event),
    None => None,
  };
  let mut events = vec![current_event];
  for (time, event) in world.timeline.iter() {
    events.push(format_event(time, event))
  }
  let entries = events
    .into_iter()
    .flatten()
    .map(|e| row(vec![text(e.0), text(" "), text(e.1), text(" "), text(e.2)]))
    .collect::<Vec<_>>();
  column(vec![text("Events:"), column(entries)])
}

fn turn_controls(world: &World) -> WidgetFn<'static> {
  let Some(current_event) = &world.current_event else {
    return column(vec![]);
  };
  let Event::Turn(id, TurnType::Player(turn)) = current_event else {
    return column(vec![]);
  };
  let activities = collect_activities(world, *id)
    .enumerate()
    .map(|(i, (id, activity))| {
      let from_name = world.name.get(&id).unwrap_or(&"???");
      let selector = if i == turn.selected_activity_index {
        text("> ")
      } else {
        text("  ")
      };
      row(vec![
        selector,
        text(activity.name.to_string()),
        text(format!(" {}t", activity.speed)),
        text(format!(" ({from_name})")),
      ])
    })
    .collect();
  border(
    (1, 0, 0, 0),
    column(vec![
      text("Activities:"),
      column(activities),
    ])
  )
}

fn format_action_description(action: &Option<Action>) -> String {
  match action {
    None => "wait".to_string(),
    Some(Action::Move(v)) => format!("move {},{}", v.0, v.1),
    Some(Action::Attack(v, d)) => format!("attack {},{} {}", v.0, v.1, d),
  }
}
