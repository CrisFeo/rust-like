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

#[macro_export]
macro_rules! instrument {
  ($name: expr, $code:expr) => {
    let start = std::time::Instant::now();
    $code;
    let elapsed = std::time::Instant::now() - start;
    if elapsed > std::time::Duration::from_millis(1) {
      log!("TIME", $name, elapsed);
    }
  }
}
