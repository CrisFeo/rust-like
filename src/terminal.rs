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

pub struct Terminal<'a> {
  old_termios: libc::termios,
  stdin: StdinLock<'a>,
  stdout: BufWriter<StdoutLock<'a>>,
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
    };
    write!(t.stdout, "\x1b[?25l")?; // hide cursor
    write!(t.stdout, "\x1b[s")?; // save cursor
    write!(t.stdout, "\x1b[?47h")?; // save screen
    write!(t.stdout, "\x1b[?1049h")?; // enter alt screen
		t.clear()?;
		t.flush()?;
    Ok(t)
  }

  pub fn flush(&mut self) -> Result<()> {
    self.stdout.flush()
  }

  pub fn read(&mut self) -> Result<Option<char>> {
    let mut b = [0u8];
    self.stdin.read_exact(&mut b)?;
    Ok(char::from_u32(b[0].into()))
  }

  pub fn char(&mut self, c: char) -> Result<()> {
    write!(self.stdout, "{c}")
  }

  pub fn clear(&mut self) -> Result<()> {
    write!(self.stdout, "\x1b[2J")
  }

  pub fn go(&mut self, x: i32, y: i32) -> Result<()> {
    let r = y + 1;
    let c = x + 1;
    write!(self.stdout, "\x1b[{r};{c}H")
  }
}

