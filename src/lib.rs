pub mod action;
pub mod event;
pub mod grid;
pub mod id;
pub mod input;
pub mod layout;
pub mod spatial_map;
pub mod terminal;
pub mod timeline;
pub mod ui;
pub mod visibility;
pub mod world;

pub use action::*;
pub use event::*;
pub use id::Id;
pub use input::Input;
pub use layout::*;
pub use spatial_map::SpatialMap;
pub use terminal::Terminal;
pub use timeline::Timeline;
pub use ui::*;
pub use visibility::*;
pub use world::*;

#[macro_export]
macro_rules! log {
  ($message:expr, $($args:expr),+ $(,)?) => {
    eprintln!(
      concat!(
        "{}",
        $(
          concat!("\n  ", stringify!($args), ": {:?}")
        ),+,
      ),
      $message,
      $($args),+
    );
  }
}
