extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use embedded_hal::serial::Read;
use stm32f1xx_hal::afio::MAPR;
use stm32f1xx_hal::gpio::gpioa::{CRH, CRL, PA10, PA2, PA3, PA9};
use stm32f1xx_hal::gpio::gpiob::{CRH as BCRH, PB10, PB11};
use stm32f1xx_hal::gpio::{Floating, Input};
use stm32f1xx_hal::rcc::Clocks;
use stm32f1xx_hal::rcc::{APB1, APB2};
use stm32f1xx_hal::serial::{Config, Error, Rx, Serial, Tx};
use stm32f1xx_hal::{pac::USART1, pac::USART2, pac::USART3};

pub fn usart1(
    usart: USART1,
    pins: (PA9<Input<Floating>>, PA10<Input<Floating>>),
    mapr: &mut MAPR,
    config: Config,
    clocks: Clocks,
    apb: &mut APB2,
    crh: &mut CRH,
) -> (Tx<USART1>, Rx<USART1>) {
    let tx = pins.0.into_alternate_push_pull(crh);
    let rx = pins.1;
    let stdout = Serial::usart1(usart, (tx, rx), mapr, config, clocks, apb);
    stdout.split()
}

pub fn usart2(
    usart: USART2,
    pins: (PA2<Input<Floating>>, PA3<Input<Floating>>),
    mapr: &mut MAPR,
    config: Config,
    clocks: Clocks,
    apb: &mut APB1,
    crl: &mut CRL,
) -> (Tx<USART2>, Rx<USART2>) {
    let tx = pins.0.into_alternate_push_pull(crl);
    let rx = pins.1;
    let serial = Serial::usart2(usart, (tx, rx), mapr, config, clocks, apb);
    serial.split()
}

pub fn usart3(
    usart: USART3,
    pins: (PB10<Input<Floating>>, PB11<Input<Floating>>),
    mapr: &mut MAPR,
    config: Config,
    clocks: Clocks,
    apb: &mut APB1,
    crh: &mut BCRH,
) -> (Tx<USART3>, Rx<USART3>) {
    let tx = pins.0.into_alternate_push_pull(crh);
    let rx = pins.1;
    let serial = Serial::usart3(usart, (tx, rx), mapr, config, clocks, apb);
    serial.split()
}

pub trait BufRead: Read<u8> {
    fn read_line(&mut self) -> core::result::Result<String, Error>;
    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> core::result::Result<usize, Error>;
    fn read_exact(&mut self, buf: &mut [u8]) -> core::result::Result<(), Error>;
}

impl BufRead for Rx<USART1> {
    #[inline]
    fn read_line(&mut self) -> core::result::Result<String, Error> {
        let mut str = String::new();
        let buf = unsafe { str.as_mut_vec() };
        self.read_until('\n' as u8, buf)?;
        Ok(str)
    }
    #[inline]
    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> core::result::Result<usize, Error> {
        let mut read = 0;
        loop {
            match nb::block!(self.read()) {
                Ok(b) => {
                    if b == byte {
                        break;
                    }
                    buf.push(b);
                    read += 1;
                }
                Err(err) => return Err(err),
            }
        }
        Ok(read)
    }
    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> core::result::Result<(), Error> {
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

impl BufRead for Rx<USART2> {
    #[inline]
    fn read_line(&mut self) -> core::result::Result<String, Error> {
        let mut str = String::new();
        let buf = unsafe { str.as_mut_vec() };
        self.read_until('\n' as u8, buf)?;
        Ok(str)
    }
    #[inline]
    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> core::result::Result<usize, Error> {
        let mut read = 0;
        loop {
            match nb::block!(self.read()) {
                Ok(b) => {
                    if b == byte {
                        break;
                    }
                    buf.push(b);
                    read += 1;
                }
                Err(err) => return Err(err),
            }
        }
        Ok(read)
    }
    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> core::result::Result<(), Error> {
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

impl BufRead for Rx<USART3> {
    #[inline]
    fn read_line(&mut self) -> core::result::Result<String, Error> {
        let mut str = String::new();
        let buf = unsafe { str.as_mut_vec() };
        self.read_until('\n' as u8, buf)?;
        Ok(str)
    }
    #[inline]
    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> core::result::Result<usize, Error> {
        let mut read = 0;
        loop {
            match nb::block!(self.read()) {
                Ok(b) => {
                    if b == byte {
                        break;
                    }
                    buf.push(b);
                    read += 1;
                }
                Err(err) => return Err(err),
            }
        }
        Ok(read)
    }
    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> core::result::Result<(), Error> {
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

// struct Guard<'a> {
//     buf: &'a mut Vec<u8, 512>,
//     len: usize,
// }

// impl Drop for Guard<'_> {
//     fn drop(&mut self) {
//         unsafe {
//             self.buf.set_len(self.len);
//         }
//     }
// }

// fn append_to_string<F>(buf: &mut String, f: F) -> Result<usize>
// where
//     F: FnOnce(&mut Vec<u8, 512>) -> Result<usize>,
// {
//     unsafe {
//         let mut g = Guard {
//             len: buf.len(),
//             buf: buf.as_mut_vec(),
//         };
//         let ret = f(g.buf);
//         if str::from_utf8(&g.buf[g.len..]).is_err() {
//             ret.and_then(|_| Err(anyhow!("stream did not contain valid UTF-8")))
//         } else {
//             g.len = g.buf.len();
//             ret
//         }
//     }
// }

// fn read_until<R: BufRead + ?Sized>(r: &mut R, delim: u8, buf: &mut Vec<u8, 512>) -> Result<usize> {
//     let mut read = 0;
//     loop {
//         let (done, used) = {
//             let available = match r.fill_buf() {
//                 Ok(n) => n,
//                 Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
//                 Err(e) => return Err(e),
//             };
//             match memchr::memchr(delim, available) {
//                 Some(i) => {
//                     buf.extend_from_slice(&available[..=i]);
//                     (true, i + 1)
//                 }
//                 None => {
//                     buf.extend_from_slice(available);
//                     (false, available.len())
//                 }
//             }
//         };
//         r.consume(used);
//         read += used;
//         if done || used == 0 {
//             return Ok(read);
//         }
//     }
// }
