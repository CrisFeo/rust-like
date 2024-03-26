pub mod action;
pub mod activity;
pub mod event;
pub mod grid;
pub mod id;
pub mod input;
pub mod layout;
pub mod relations;
pub mod terminal;
pub mod timeline;
pub mod turn;
pub mod ui;
pub mod visibility;
pub mod world;

pub use action::*;
pub use activity::*;
pub use event::*;
pub use id::Id;
pub use input::Input;
pub use layout::*;
pub use relations::*;
pub use terminal::Terminal;
pub use timeline::Timeline;
pub use turn::*;
pub use ui::*;
pub use visibility::*;
pub use world::*;

#[macro_export]
macro_rules! log {
  ($system: expr, $message:expr, $($args:expr),+ $(,)?) => {
    eprintln!(
      concat!(
        "[{}] {}",
        $(
          concat!("\n  ", stringify!($args), ": {:?}")
        ),+,
      ),
      $system,
      $message,
      $($args),+
    );
  }
}
