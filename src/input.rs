use std::mem;

#[derive(Debug, Default)]
pub enum Input {
  #[default]
  None,
  Some(char),
  Requested,
}

impl Input {
  pub fn is_requested(&self) -> bool {
    matches!(self, Self::Requested)
  }

  pub fn take_or_request(&mut self) -> Option<char> {
    match self {
      Self::None => {
        _ = mem::replace(self, Self::Requested);
        None
      }
      Self::Some(_) => {
        let Self::Some(char) = mem::replace(self, Self::None) else {
          unreachable!()
        };
        Some(char)
      }
      Self::Requested => None,
    }
  }
}
