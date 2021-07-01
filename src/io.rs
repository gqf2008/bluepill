use core::fmt::Write;
use heapless::String;
use heapless::Vec;
use nb::block;

pub type Result<T> = core::result::Result<T, crate::io::Error>;

#[derive(Debug)]
pub enum Error {
    Timeout,
    EOF,
    WriteError,
    ReadError,
    Other(u8),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match *self {
            Error::Timeout => write!(f, "timeout"),
            Error::EOF => write!(f, "end of file"),
            Error::WriteError => write!(f, "write error"),
            Error::ReadError => write!(f, "read error"),
            Error::Other(code) => write!(f, "error {}", code),
        }
    }
}

pub trait Timeout: embedded_hal::serial::Read<u8> {
    fn read_timeout(&mut self, ms: u32) -> Result<u8> {
        loop {
            match self.read() {
                Ok(b) => return Ok(b),
                Err(nb::Error::WouldBlock) => {}
                Err(nb::Error::Other(_)) => return Err(Error::ReadError),
            }
        }
    }
}

pub trait BufRead: embedded_hal::serial::Read<u8> {
    fn read_line<const N: usize>(&mut self) -> Result<String<N>> {
        let mut str: String<N> = String::new();
        let buf = unsafe { str.as_mut_vec() };
        self.read_until('\n' as u8, buf)?;
        Ok(str)
    }
    fn read_until<const N: usize>(&mut self, byte: u8, buf: &mut Vec<u8, N>) -> Result<usize> {
        let mut read = 0;
        loop {
            match nb::block!(self.read()) {
                Ok(b) => {
                    if b == byte {
                        break;
                    }
                    buf.push(b).ok();
                    read += 1;
                }
                Err(_err) => return Err(Error::ReadError),
            }
        }
        Ok(read)
    }
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        buf.iter_mut()
            .try_for_each(|b| match nb::block!(self.read()) {
                Ok(r) => {
                    *b = r;
                    Ok(())
                }
                Err(_err) => return Err(Error::ReadError),
            })
    }
}

impl<'p, T> BufRead for Stdin<'p, T> where T: embedded_hal::serial::Read<u8> {}
pub struct Stdin<'p, T>(pub &'p mut T);

impl<'p, T> embedded_hal::serial::Read<u8> for Stdin<'p, T>
where
    T: embedded_hal::serial::Read<u8>,
{
    type Error = Error;
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        match self.0.read() {
            Ok(b) => Ok(b),
            Err(_err) => return Err(nb::Error::Other(Error::ReadError)),
        }
    }
}
pub struct Stdout<'p, T>(pub &'p mut T);

impl<'p, T> Write for Stdout<'p, T>
where
    T: embedded_hal::serial::Write<u8>,
{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.as_bytes() {
            if *byte == b'\n' {
                let res = block!(self.0.write(b'\r'));
                if res.is_err() {
                    return Err(core::fmt::Error);
                }
            }

            let res = block!(self.0.write(*byte));

            if res.is_err() {
                return Err(core::fmt::Error);
            }
        }
        Ok(())
    }
}
