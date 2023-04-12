use std::io::{self, Write};

#[inline]
pub fn flush() -> io::Result<()> {
    io::stdout().lock().flush()
}
