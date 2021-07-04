use core::fmt::Write;
use embedded_hal::timer::CountDown;
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
    BufferFull,
    NoIoDevice,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match *self {
            Error::Timeout => write!(f, "timeout"),
            Error::EOF => write!(f, "end of file"),
            Error::WriteError => write!(f, "write error"),
            Error::ReadError => write!(f, "read error"),
            Error::Other(code) => write!(f, "error {}", code),
            Error::BufferFull => write!(f, "buffer full"),
            Error::NoIoDevice => write!(f, "no io device"),
        }
    }
}

pub struct TimeoutReader<'a, R, TIM>(pub &'a mut R, pub &'a mut TIM);

impl<'a, R, TIM> TimeoutReader<'a, R, TIM>
where
    R: embedded_hal::serial::Read<u8>,
    TIM: CountDown,
{
    pub fn read_line<const N: usize>(&mut self, timeout: TIM::Time) -> Result<String<N>> {
        let mut str: String<N> = String::new();
        let buf = unsafe { str.as_mut_vec() };
        self.read_until('\n' as u8, buf, timeout)?;
        Ok(str)
    }

    pub fn read_until<const N: usize>(
        &mut self,
        byte: u8,
        buf: &mut Vec<u8, N>,
        timeout: TIM::Time,
    ) -> Result<usize> {
        let mut read = 0;
        self.1.start(timeout);

        loop {
            match self.0.read() {
                Err(nb::Error::WouldBlock) => {}
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
                Ok(()) => return Err(Error::Timeout),
            }
        }
    }

    pub fn read_exact(&mut self, buf: &mut [u8], timeout: TIM::Time) -> Result<()> {
        self.1.start(timeout);
        let len = buf.len();
        let mut i = 0;
        loop {
            match self.0.read() {
                Ok(r) => {
                    buf[i] = r;
                    i += 1;
                    if i == len {
                        return Ok(());
                    }
                }
                Err(nb::Error::WouldBlock) => {}
                Err(err) => return Err(Error::ReadError),
            }
            match self.1.wait() {
                Err(nb::Error::Other(_e)) => {
                    unreachable!()
                }
                Err(nb::Error::WouldBlock) => continue,
                Ok(()) => return Err(Error::Timeout),
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
