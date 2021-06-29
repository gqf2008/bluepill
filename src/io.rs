use core::fmt::Write;
use heapless::String;
use heapless::Vec;
use nb::block;

pub trait BufRead {
    fn read_line<const N: usize>(&mut self) -> core::result::Result<String<N>, core::fmt::Error> {
        let mut str: String<N> = String::new();
        let buf = unsafe { str.as_mut_vec() };
        self.read_until('\n' as u8, buf)?;
        Ok(str)
    }
    fn read_until<const N: usize>(
        &mut self,
        byte: u8,
        buf: &mut Vec<u8, N>,
    ) -> core::result::Result<usize, core::fmt::Error>;
    fn read_exact(&mut self, buf: &mut [u8]) -> core::result::Result<(), core::fmt::Error>;
}

pub struct Stdin<'p, T>(pub &'p mut T);

impl<'p, T> BufRead for Stdin<'p, T>
where
    T: embedded_hal::serial::Read<u8>,
{
    fn read_until<const N: usize>(
        &mut self,
        byte: u8,
        buf: &mut Vec<u8, N>,
    ) -> core::result::Result<usize, core::fmt::Error> {
        let mut read = 0;
        loop {
            match nb::block!(self.0.read()) {
                Ok(b) => {
                    if b == byte {
                        break;
                    }
                    buf.push(b).ok();
                    read += 1;
                }
                Err(err) => return Err(core::fmt::Error),
            }
        }
        Ok(read)
    }
    fn read_exact(&mut self, buf: &mut [u8]) -> core::result::Result<(), core::fmt::Error> {
        buf.iter_mut()
            .try_for_each(|b| match nb::block!(self.0.read()) {
                Ok(r) => {
                    *b = r;
                    Ok(())
                }
                Err(err) => return Err(core::fmt::Error),
            })
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
