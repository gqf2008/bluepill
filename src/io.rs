use core::fmt;
use cortex_m::interrupt;
use embedded_hal::serial::{Read, Write};
use heapless::String;
use heapless::Vec;
use nb::block;
use stm32f1xx_hal::pac::USART1;
use stm32f1xx_hal::serial::Tx;

static mut STDOUT: Option<SerialWrapper> = None;
// const MAX_BUF_SIZE: usize = 256;
pub trait BufRead: Read<u8> {
    fn read_line<const N: usize>(&mut self) -> core::result::Result<String<N>, Self::Error> {
        let mut str: String<N> = String::new();
        let buf = unsafe { str.as_mut_vec() };
        self.read_until('\n' as u8, buf)?;
        Ok(str)
    }
    fn read_until<const N: usize>(
        &mut self,
        byte: u8,
        buf: &mut Vec<u8, N>,
    ) -> core::result::Result<usize, Self::Error> {
        let mut read = 0;
        loop {
            match nb::block!(self.read()) {
                Ok(b) => {
                    if b == byte {
                        break;
                    }
                    //TODO error check
                    buf.push(b).ok();
                    read += 1;
                }
                Err(err) => return Err(err),
            }
        }
        Ok(read)
    }
    fn read_exact(&mut self, buf: &mut [u8]) -> core::result::Result<(), Self::Error> {
        buf.iter_mut()
            .try_for_each(|b| match nb::block!(self.read()) {
                Ok(r) => {
                    *b = r;
                    Ok(())
                }
                Err(err) => return Err(err),
            })
    }
}

struct SerialWrapper(Tx<USART1>);

impl fmt::Write for SerialWrapper {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.as_bytes() {
            if *byte == '\n' as u8 {
                let res = block!(self.0.write('\r' as u8));

                if res.is_err() {
                    return Err(::core::fmt::Error);
                }
            }

            let res = block!(self.0.write(*byte));

            if res.is_err() {
                return Err(::core::fmt::Error);
            }
        }
        Ok(())
    }
}

pub trait Stdout {
    fn to_stdout(self);
}

impl Stdout for Tx<USART1> {
    fn to_stdout(self) {
        interrupt::free(|_| unsafe {
            STDOUT.replace(SerialWrapper(self));
        });
    }
}

/// Writes string to stdout
pub fn write_str(s: &str) {
    interrupt::free(|_| unsafe {
        if let Some(stdout) = STDOUT.as_mut() {
            let _ = <SerialWrapper as core::fmt::Write>::write_str(stdout, s);
        }
    })
}

/// Writes formatted string to stdout
pub fn write_fmt(args: fmt::Arguments) {
    interrupt::free(|_| unsafe {
        if let Some(stdout) = STDOUT.as_mut() {
            let _ = <SerialWrapper as core::fmt::Write>::write_fmt(stdout, args);
        }
    })
}

/// Macro for printing to the serial standard output
#[macro_export]
macro_rules! sprint {
    ($s:expr) => {
        $crate::io::write_str($s)
    };
    ($($tt:tt)*) => {
        $crate::io::write_fmt(format_args!($($tt)*))
    };
}

/// Macro for printing to the serial standard output, with a newline.
#[macro_export]
macro_rules! sprintln {
    () => {
        $crate::io::write_str("\n")
    };
    ($s:expr) => {
        $crate::io::write_str(concat!($s, "\n"))
    };
    ($s:expr, $($tt:tt)*) => {
        $crate::io::write_fmt(format_args!(concat!($s, "\n"), $($tt)*))
    };
}
