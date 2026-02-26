// Copyright © ickk, 2026

use {crate::SlimVec, ::std::io};

impl io::Write for SlimVec<u8> {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    self.extend(buf);
    Ok(buf.len())
  }

  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}
