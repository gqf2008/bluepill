//! 通过串口连接ESP8266模块，发送AT指令联网
//!

#![no_main]
#![no_std]

#[macro_use(singleton)]
extern crate cortex_m;

use bluepill::clocks::*;
use bluepill::hal::delay::Delay;
use bluepill::hal::dma::{CircReadDma, Half, RxDma};
use bluepill::hal::gpio::gpioc::PC13;
use bluepill::hal::gpio::{Output, PushPull};
use bluepill::hal::{
    pac::interrupt,
    pac::Interrupt,
    pac::{USART1, USART2},
    prelude::*,
    serial::{Config, Rx, Serial, Tx},
};
use bluepill::led::*;
use core::borrow::Borrow;
use core::cell::RefCell;
use core::fmt::Write;
use cortex_m::asm;
use stm32f1xx_hal::dma::dma1::C5;

use cortex_m::{asm::wfi, interrupt::Mutex};
use cortex_m_rt::entry;
use embedded_dma::ReadBuffer;
use heapless::spsc::{Consumer, Producer, Queue};
use heapless::Vec;
use panic_halt as _;

#[entry]
fn main() -> ! {
    let p = bluepill::Peripherals::take().unwrap(); //核心设备、外围设备
    let mut flash = p.device.FLASH.constrain(); //Flash

    let mut rcc = p.device.RCC.constrain(); //RCC
    let mut afio = p.device.AFIO.constrain(&mut rcc.apb2);
    let clocks = rcc.cfgr.full_clocks(&mut flash.acr); //配置全速时钟

    let channels = p.device.DMA1.split(&mut rcc.ahb);
    let mut gpioa = p.device.GPIOA.split(&mut rcc.apb2);
    let mut gpioc = p.device.GPIOC.split(&mut rcc.apb2);

    let mut delay = Delay::new(p.core.SYST, clocks); //配置延时器
    let mut led = gpioc.pc13.to_led(&mut gpioc.crh); //配置LED

    let (mut stdout, mut stdin) = bluepill::hal::serial::Serial::usart1(
        p.device.USART1,
        (
            gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh),
            gpioa.pa10,
        ),
        &mut afio.mapr,
        Config::default().baudrate(115200.bps()),
        clocks,
        &mut rcc.apb2,
    )
    .split();
    let mut rx = stdin.with_dma(channels.5);
    read_dma1(&mut stdout, rx);
    loop {}
}

fn read_dma_circ(stdout: &mut Tx<USART1>, rx: RxDma<Rx<USART1>, C5>) {
    let mut rx = rx;
    let buf = singleton!(: [[u8; 8]; 2] = [[0; 8]; 2]).unwrap();
    let mut s: heapless::String<16384> = heapless::String::new();
    read_dma(stdout, rx)
}

fn read_dma1(stdout: &mut Tx<USART1>, rx: RxDma<Rx<USART1>, C5>) -> ! {
    let mut rx = rx;
    let mut buf = singleton!(: [u8; 8]  = [0; 8]).unwrap();
    let mut s: heapless::String<4096> = heapless::String::new();
    loop {
        let t = rx.read(buf);
        while !t.is_done() {
            // let out = t.peek();
            // out.iter().for_each(|b| {
            //     let c = *b as char;
            //     if c == '\n' {
            //         stdout.write_fmt(format_args!("{}", s));
            //         s.clear();
            //     } else {
            //         if s.len() == 4096 {
            //             stdout.write_fmt(format_args!("{}", s));
            //             s.clear();
            //         }
            //         s.push(c).ok();
            //     }
            // });
        }
        let (out, _rx) = t.wait();
        out.iter().for_each(|b| {
            let c = *b as char;
            if c == '\n' {
                stdout.write_fmt(format_args!("{}", s));
                s.clear();
            } else {
                if s.len() == 4096 {
                    stdout.write_fmt(format_args!("{}", s));
                    s.clear();
                }
                s.push(c).ok();
            }
        });
        rx = _rx;
        buf = out;
        //led.toggle();
    }
}

fn read_dma(stdout: &mut Tx<USART1>, rx: RxDma<Rx<USART1>, C5>) {
    let mut rx = rx;
    let mut buf = singleton!(: [u8; 1]  = [0; 1]).unwrap();
    let mut s: heapless::String<4096> = heapless::String::new();
    loop {
        let (out, _rx) = rx.read(buf).wait();
        out.iter().for_each(|b| {
            let c = *b as char;
            if c == '\n' {
                //s.push(c).ok();
                stdout.write_fmt(format_args!("{}", s));
                s.clear();
            } else {
                s.push(*b as char).ok();
            }
        });
        if s.len() == 4096 {
            stdout.write_fmt(format_args!("{}", s));
            s.clear();
        }

        rx = _rx;
        buf = out;
        //led.toggle();
    }
}

static RX2: Mutex<RefCell<Option<Rx<USART2>>>> = Mutex::new(RefCell::new(None));
static mut Q: Queue<Message, 4096> = Queue::new();

#[derive(Debug)]
enum Message {
    Byte(u8),
    Error(bluepill::hal::serial::Error),
}
#[interrupt]
unsafe fn USART2() {
    static mut RX: Option<Rx<USART2>> = None;
    let mut producer = unsafe { Q.split().0 };

    let rx2 = RX.get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs| RX2.borrow(cs).replace(None).unwrap())
    });

    // let queue = QUEUE.get_or_insert_with(|| {
    //     cortex_m::interrupt::free(|cs| STREAM.borrow(cs).replace(None).unwrap())
    // });
    let msg = match nb::block!(rx2.read()) {
        Ok(w) => Message::Byte(w),
        Err(e) => Message::Error(e),
    };
    producer.enqueue(msg).ok();
    // cortex_m::interrupt::free(|cs| {
    //     if let Some(rx2) = RX2.as_mut() {
    //         let msg = match nb::block!(rx2.read()) {
    //             Ok(w) => Message::Byte(w),
    //             Err(e) => Message::Error(e),
    //         };
    //         STREAM.borrow(cs).borrow_mut().enqueue(msg).ok();
    //     }
    // })
}
