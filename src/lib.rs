pub mod id;
pub mod terminal;
pub mod spatial_map;

pub use terminal::*;
pub use id::*;
pub use spatial_map::*;

#[macro_export]
macro_rules! or_continue {
  ($value:expr) => {
    match $value {
      Some(v) => v,
      None => continue,
    }
  }
}
