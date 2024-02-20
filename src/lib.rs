pub mod action;
pub mod event;
pub mod grid;
pub mod id;
pub mod spatial_map;
pub mod terminal;
pub mod visibility;
pub mod world;

pub use action::*;
pub use event::Event;
pub use id::Id;
pub use spatial_map::SpatialMap;
pub use terminal::Terminal;
pub use visibility::*;
pub use world::*;

#[macro_export]
macro_rules! or_continue {
  ($value:expr) => {
    match $value {
      Some(v) => v,
      None => continue,
    }
  }
}
