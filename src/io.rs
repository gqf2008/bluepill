use crate::hal::serial::Error as SerialError;

use core::fmt::Write;
use embedded_hal::timer::CountDown;
use heapless::String;
use heapless::Vec;
use nb::block;
use stm32f1xx_hal::pac::USART1;
use stm32f1xx_hal::serial::Tx;

pub type Result<T> = core::result::Result<T, crate::io::Error>;

#[derive(Debug)]
pub enum Error {
    Timeout,
    EOF,
    WriteError,
    ReadError,
    Other(u8),
    BufferFull,
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
                    //tx.write_str(format!("{:#?}",).as_str()).ok();
                    return Err(Error::ReadError);
                }
                Ok(b) => {
                    read += 1;
                    if let Err(_e) = buf.push(b) {
                        return Err(Error::BufferFull);
                    }
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
                    if b == byte {
                        break;
                    }
                    buf.push(b).ok();
                    read += 1;
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

pub struct Stdin<'p, T>(pub &'p mut T);

impl<'p, T> BufRead for Stdin<'p, T> where T: embedded_hal::serial::Read<u8> {}

impl<'p, T> embedded_hal::serial::Read<u8> for Stdin<'p, T>
where
    T: embedded_hal::serial::Read<u8>,
{
    type Error = Error;
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        match self.0.read() {
            Ok(b) => Ok(b),
            Err(err) => return Err(nb::Error::Other(Error::ReadError)),
        }
    }
}

pub struct Stdout<'p, T>(pub &'p mut T);

impl<'p, T> Write for Stdout<'p, T>
where
    T: embedded_hal::serial::Write<u8, Error = ::core::convert::Infallible>,
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
