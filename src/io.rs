use crate::hal::time::Hertz;
use crate::hal::time::U32Ext;
use embedded_hal::timer::CountDown;
use heapless::String;
use heapless::Vec;

pub type Result<T> = core::result::Result<T, crate::io::Error>;

#[derive(Debug)]
pub enum Error {
    Timeout,
    EOF,
    WriteError,
    ReadError,
    Other(String<256>),
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
    pub fn read_line<const N: usize>(&mut self, milliseconds: u32) -> Result<String<N>> {
        let mut str: String<N> = String::new();
        let buf = unsafe { str.as_mut_vec() };
        self.read_until('\n' as u8, buf, milliseconds)?;
        Ok(str)
    }

    pub fn read_until<const N: usize>(
        &mut self,
        byte: u8,
        buf: &mut Vec<u8, N>,
        milliseconds: u32,
    ) -> Result<usize> {
        let mut read = 0;
        self.1.start(1.khz());
        let mut timeout = 0;
        loop {
            match self.0.read() {
                Err(nb::Error::WouldBlock) => {
                    timeout += 1;
                }
                Err(nb::Error::Other(e)) => {
                    return Err(Error::ReadError);
                }
                Ok(b) => {
                    if let Err(_e) = buf.push(b) {
                        return Err(Error::BufferFull);
                    }
                    read += 1;
                    if byte == b {
                        return Ok(read);
                    }
                }
            }
            match self.1.wait() {
                Err(nb::Error::Other(_e)) => {
                    unreachable!()
                }
                Err(nb::Error::WouldBlock) => {}
                Ok(()) => {
                    if timeout == milliseconds {
                        return Err(Error::Timeout);
                    }
                    self.1.start(1.khz());
                }
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
                Err(err) => return Err(Error::ReadError),
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
                    if let Err(_e) = buf.push(b) {
                        return Err(Error::BufferFull);
                    }
                    read += 1;
                    if b == byte {
                        break;
                    }
                }
                Err(err) => return Err(Error::ReadError),
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
                Err(err) => return Err(Error::ReadError),
            })
    }
}
