use crate::hal::time::Hertz;
use crate::hal::time::U32Ext;
use alloc::string::String;
use alloc::vec::Vec;
use embedded_hal::timer::CountDown;

pub type Result<T> = core::result::Result<T, crate::io::Error>;

#[derive(Debug)]
pub enum Error {
    Timeout,
    EOF,
    WriteError,
    ReadError,
    Other(String),
    BufferFull,
    NoIoDevice,
    NoNetwork,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::Timeout => write!(f, "timeout"),
            Error::EOF => write!(f, "end of file"),
            Error::WriteError => write!(f, "write error"),
            Error::ReadError => write!(f, "read error"),
            Error::Other(detail) => write!(f, "error {}", detail),
            Error::BufferFull => write!(f, "buffer full"),
            Error::NoIoDevice => write!(f, "no io device"),
            Error::NoNetwork => write!(f, "no network"),
        }
    }
}

pub struct TimeoutReader<'a, R, TIM>(pub &'a mut R, pub &'a mut TIM);

impl<'a, R, TIM> TimeoutReader<'a, R, TIM>
where
    R: embedded_hal::serial::Read<u8>,
    TIM: CountDown<Time = Hertz>,
{
    pub fn read_line(&mut self, milliseconds: u32) -> Result<String> {
        let mut str = String::new();
        let buf = unsafe { str.as_mut_vec() };
        self.read_until('\n' as u8, buf, milliseconds)?;
        Ok(str)
    }

    pub fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>, milliseconds: u32) -> Result<usize> {
        let mut read = 0;
        self.1.start(1.khz());
        let mut timeout = 0;
        loop {
            match self.0.read() {
                Err(nb::Error::WouldBlock) => {}
                Err(nb::Error::Other(_e)) => {
                    return Err(Error::ReadError);
                }
                Ok(b) => {
                    buf.push(b);
                    read += 1;
                    if byte == b {
                        return Ok(read);
                    }
                }
            }
            nb::block!(self.1.wait()).ok();
            timeout += 1;
            if timeout >= milliseconds {
                return Err(Error::Timeout);
            }
        }
    }

    pub fn read_exact(&mut self, buf: &mut [u8], milliseconds: u32) -> Result<()> {
        self.1.start(1.khz());
        let len = buf.len();
        let mut i = 0;
        let mut timeout = 0;
        loop {
            match self.0.read() {
                Ok(r) => {
                    buf[i] = r;
                    i += 1;
                    if i == len {
                        return Ok(());
                    }
                }
                Err(nb::Error::WouldBlock) => {
                    timeout += 1;
                }
                Err(_err) => return Err(Error::ReadError),
            }
            match self.1.wait() {
                Err(nb::Error::Other(_e)) => {
                    unreachable!()
                }
                Err(nb::Error::WouldBlock) => continue,
                Ok(()) => {
                    if timeout == milliseconds {
                        return Err(Error::Timeout);
                    }
                    self.1.start(1.khz());
                }
            }
        }
    }
}

pub trait BufRead: embedded_hal::serial::Read<u8> {
    fn read_line(&mut self) -> Result<String> {
        let mut str = String::new();

        let buf = unsafe { str.as_mut_vec() };
        self.read_until('\n' as u8, buf)?;
        Ok(str)
    }

    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> Result<usize> {
        let mut read = 0;
        loop {
            match nb::block!(self.read()) {
                Ok(b) => {
                    buf.push(b);
                    read += 1;
                    if b == byte {
                        break;
                    }
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
