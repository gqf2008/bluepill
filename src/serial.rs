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
