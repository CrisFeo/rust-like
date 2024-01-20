use std::collections::HashMap;
use std::io::{
  Read,
  Write,
  Error,
  Result,
  BufWriter,
  StdinLock,
  StdoutLock,
  stdin,
  stdout,
};
use crate::grid::Point;

pub struct Terminal<'a> {
  old_termios: libc::termios,
  stdin: StdinLock<'a>,
  stdout: BufWriter<StdoutLock<'a>>,
  buffer: HashMap<(i32, i32), char>,
  screen: HashMap<(i32, i32), char>,
}

impl<'a> Drop for Terminal<'a> {
  fn drop(&mut self) {
  	unsafe {
      let result = libc::tcsetattr(
        libc::STDOUT_FILENO,
        libc::TCSANOW,
  			&self.old_termios
      );
  		if result == -1 {
  			panic!("{}", Error::last_os_error());
  		}
    }
    write!(self.stdout, "\x1b[?1049l").unwrap(); // exit alt screen
    write!(self.stdout, "\x1b[?47l").unwrap(); // load screen
    write!(self.stdout, "\x1b[u").unwrap(); // load cursor
    write!(self.stdout, "\x1b[?25h").unwrap(); // show cursor
  }
}

impl<'a> Terminal<'a> {
  pub fn new() -> Result<Self> {
  	let old_termios = unsafe {
  		let mut old_termios = std::mem::zeroed();
      let result = libc::tcgetattr(
        libc::STDOUT_FILENO,
  			&mut old_termios
      );
  		if result == -1 {
  			return Err(Error::last_os_error());
  		}
  		let mut raw_termios = old_termios;
  		libc::cfmakeraw(&mut raw_termios);
      let result = libc::tcsetattr(
        libc::STDOUT_FILENO,
        libc::TCSANOW,
  			&raw_termios
      );
  		if result == -1 {
  			return Err(Error::last_os_error());
  		}
  		old_termios
    };
    let stdout = BufWriter::new(stdout().lock());
    let mut t = Terminal {
      old_termios,
      stdin: stdin().lock(),
      stdout,
      buffer: HashMap::new(),
      screen: HashMap::new(),
    };
    write!(t.stdout, "\x1b[?25l")?; // hide cursor
    write!(t.stdout, "\x1b[s")?; // save cursor
    write!(t.stdout, "\x1b[?47h")?; // save screen
    write!(t.stdout, "\x1b[?1049h")?; // enter alt screen
    write!(t.stdout, "\x1b[2J")?; // clear screen
		t.stdout.flush()?;
    Ok(t)
  }

  pub fn set(&mut self, point: Point, c: char) {
    self.buffer.insert((point.0 + 1, point.1 + 1), c);
  }

  pub fn present(&mut self) -> Result<()> {
    for ((col, row), char) in self.buffer.iter() {
      write!(self.stdout, "\x1b[{row};{col}H{char}")?;
    }
    for (pos, _) in self.screen.iter() {
      if !self.buffer.contains_key(pos) {
        let col = pos.0;
        let row = pos.1;
        write!(self.stdout, "\x1b[{row};{col}H ")?;
      }
    }
    std::mem::swap(&mut self.buffer, &mut self.screen);
    self.buffer.clear();
    self.stdout.flush()
  }

  pub fn read(&mut self) -> Result<Option<char>> {
    let mut b = [0u8];
    self.stdin.read_exact(&mut b)?;
    Ok(char::from_u32(b[0].into()))
  }
}
