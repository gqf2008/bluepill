use crate::io::{self, *};
use core::fmt::{self, Write};
use cortex_m::interrupt;
use heapless::String;

use stm32f1xx_hal::pac::USART1;
use stm32f1xx_hal::serial::{Rx, Tx};

static mut STDOUT: Option<Stdout<Tx<USART1>>> = None;
static mut STDIN: Option<Stdin<Rx<USART1>>> = None;

pub fn use_rx1(rx: Rx<USART1>) {
    interrupt::free(|_| unsafe {
        STDIN.replace(Stdin(rx));
    })
}

pub fn use_tx1(tx: Tx<USART1>) {
    interrupt::free(|_| unsafe {
        STDOUT.replace(Stdout(tx));
    })
}

pub struct Stdin<T>(pub T);

impl<T> BufRead for Stdin<T> where T: embedded_hal::serial::Read<u8> {}

impl<T> embedded_hal::serial::Read<u8> for Stdin<T>
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

pub struct Stdout<T>(pub T);

impl<T> Write for Stdout<T>
where
    T: embedded_hal::serial::Write<u8, Error = ::core::convert::Infallible>,
{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        s.as_bytes()
            .iter()
            .try_for_each(|c| nb::block!(self.0.write(*c)))
            .map_err(|_| core::fmt::Error)
    }
}

pub fn read_line<const N: usize>() -> io::Result<String<N>> {
    interrupt::free(|_| unsafe {
        if let Some(stdin) = STDIN.as_mut() {
            stdin.read_line()
        } else {
            Err(Error::NoIoDevice)
        }
    })
}

/// Writes string to stdout
pub fn write_str(s: &str) {
    interrupt::free(|_| unsafe {
        if let Some(stdout) = STDOUT.as_mut() {
            let _ = stdout.write_str(s);
        }
    })
}

/// Writes formatted string to stdout
pub fn write_fmt(args: fmt::Arguments) {
    interrupt::free(|_| unsafe {
        if let Some(stdout) = STDOUT.as_mut() {
            let _ = stdout.write_fmt(args);
        }
    })
}

/// Macro for printing to the serial standard output
#[macro_export]
macro_rules! sprint {
    ($s:expr) => {
        $crate::stdio::write_str($s)
    };
    ($($tt:tt)*) => {
        $crate::stdio::write_fmt(format_args!($($tt)*))
    };
}

/// Macro for printing to the serial standard output, with a newline.
#[macro_export]
macro_rules! sprintln {
    () => {
        $crate::stdio::write_str("\n")
    };
    ($s:expr) => {
        $crate::stdio::write_str(concat!($s, "\n"))
    };
    ($s:expr, $($tt:tt)*) => {
        $crate::stdio::write_fmt(format_args!(concat!($s, "\n"), $($tt)*))
    };
}
