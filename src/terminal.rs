use crate::grid::Point;
use std::collections::HashMap;
use std::io::{stdin, stdout, BufWriter, Error, Read, Result, StdoutLock, Write};
use std::mem;
use std::sync::mpsc;
use std::thread;
use std::time;

pub enum Event {
  Input(char),
  Tick(time::Instant),
}

pub struct Terminal<'a> {
  old_termios: libc::termios,
  event_receiver: mpsc::Receiver<Event>,
  stdout: BufWriter<StdoutLock<'a>>,
  buffer: HashMap<(i32, i32), char>,
  screen: HashMap<(i32, i32), char>,
}

impl<'a> Drop for Terminal<'a> {
  fn drop(&mut self) {
    unsafe {
      let result = libc::tcsetattr(libc::STDOUT_FILENO, libc::TCSANOW, &self.old_termios);
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
      let mut old_termios = mem::zeroed();
      let result = libc::tcgetattr(libc::STDOUT_FILENO, &mut old_termios);
      if result == -1 {
        return Err(Error::last_os_error());
      }
      let mut raw_termios = old_termios;
      libc::cfmakeraw(&mut raw_termios);
      let result = libc::tcsetattr(libc::STDOUT_FILENO, libc::TCSANOW, &raw_termios);
      if result == -1 {
        return Err(Error::last_os_error());
      }
      old_termios
    };
    let (event_sender, event_receiver) = mpsc::channel();
    set_up_input_event(event_sender.clone());
    set_up_tick_event(event_sender);
    let stdout = BufWriter::new(stdout().lock());
    let mut t = Terminal {
      old_termios,
      stdout,
      event_receiver,
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

  pub fn dimensions(&self) -> Result<(i32, i32)> {
    unsafe {
      let size: libc::winsize = mem::zeroed();
      let result = libc::ioctl(0, libc::TIOCGWINSZ, &size);
      if result == -1 {
        return Err(Error::last_os_error());
      }
      Ok((size.ws_col as i32, size.ws_row as i32))
    }
  }

  pub fn clear_screen(&mut self) -> Result<()> {
    write!(self.stdout, "\x1b[2J")?;
    self.screen.clear();
    Ok(())
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
    mem::swap(&mut self.buffer, &mut self.screen);
    self.buffer.clear();
    self.stdout.flush()
  }

  pub fn poll(&mut self) -> Event {
    self.event_receiver.recv().unwrap()
  }
}

fn set_up_input_event(event_sender: mpsc::Sender<Event>) {
  thread::spawn(move || {
    let mut b = [0u8];
    let mut stdin = stdin().lock();
    loop {
      stdin
        .read_exact(&mut b)
        .expect("reading one byte from input should not fail");
      let char = char::from_u32(b[0].into());
      if let Some(char) = char {
        event_sender
          .send(Event::Input(char))
          .expect("sending input event should not fail");
      }
    }
  });
}

fn set_up_tick_event(event_sender: mpsc::Sender<Event>) {
  thread::spawn(move || {
    let delay = time::Duration::from_millis(100);
    loop {
      thread::sleep(delay);
      let now = time::Instant::now();
      event_sender
        .send(Event::Tick(now))
        .expect("sending tick event should not fail");
    }
  });
}
