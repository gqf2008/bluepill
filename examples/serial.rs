//! 通过串口连接ESP8266模块，发送AT指令联网
//!

#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::fmt::Write;

use alloc_cortex_m::CortexMHeap;
use bluepill::hal::delay::Delay;
use bluepill::hal::gpio::gpioc::PC13;
use bluepill::hal::gpio::{Output, PushPull};
use bluepill::hal::prelude::*;
use bluepill::led::Led;
use bluepill::*;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use hal::{
    pac::interrupt,
    pac::Interrupt,
    pac::{USART1, USART2},
    prelude::*,
    serial::{Config, Rx, Tx, *},
};
use heapless::Vec;
use panic_semihosting as _;
/// 堆内存分配器
#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();
/// 堆内存 12K
const HEAP_SIZE: usize = 1024 * 12;

static mut STDIN: Option<Rx<USART1>> = None;
static mut STDOUT: Option<Tx<USART1>> = None;
static mut TX2: Option<Tx<USART2>> = None;
static mut RX2: Option<Rx<USART2>> = None;

#[entry]
fn main() -> ! {
    unsafe {
        ALLOCATOR.init(cortex_m_rt::heap_start() as usize, HEAP_SIZE);
    }
    let p = bluepill::Peripherals::take().unwrap(); //核心设备、外围设备
    let mut flash = p.device.FLASH.constrain(); //Flash
    let mut rcc = p.device.RCC.constrain(); //RCC
    let mut afio = p.device.AFIO.constrain(&mut rcc.apb2);
    let clocks = bluepill::clocks::full_clocks(rcc.cfgr, &mut flash.acr); //配置全速时钟
    let mut gpioa = p.device.GPIOA.split(&mut rcc.apb2);
    let mut gpioc = p.device.GPIOC.split(&mut rcc.apb2);

    let mut delay = Delay::new(p.core.SYST, clocks); //配置延时器
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh); //配置LED

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
    let (mut tx2, mut rx2) = bluepill::hal::serial::Serial::usart2(
        p.device.USART2,
        (
            gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl),
            gpioa.pa3,
        ),
        &mut afio.mapr,
        Config::default().baudrate(115200.bps()),
        clocks,
        &mut rcc.apb1,
    )
    .split();

    stdin.listen();
    rx2.listen();

    stdout.write_str("Hello ESP8266\n");
    cortex_m::interrupt::free(|_| unsafe {
        STDOUT.replace(stdout);
        STDIN.replace(stdin);
        TX2.replace(tx2);
        RX2.replace(rx2);
    });
    //开启USART1、USART2中断
    bluepill::enable_interrupt(stm32f1xx_hal::pac::Interrupt::USART1);
    bluepill::enable_interrupt(stm32f1xx_hal::pac::Interrupt::USART2);
    loop {
        led.toggle();
        delay.delay_ms(1_000u32);
    }
}

// 内存不足执行此处代码(调试用)
#[alloc_error_handler]
fn oom(_layout: core::alloc::Layout) -> ! {
    cortex_m::asm::bkpt();
    loop {}
}

#[interrupt]
fn USART1() {
    cortex_m::interrupt::free(|_| unsafe {
        if let Some(stdin) = STDIN.as_mut() {
            match nb::block!(stdin.read()) {
                Ok(w) => {
                    if let Some(tx2) = TX2.as_mut() {
                        tx2.write(w).ok();
                    }
                }
                Err(e) => {
                    if let Some(stdout) = STDOUT.as_mut() {
                        stdout.write_fmt(format_args!("ERROR {:?}", e));
                    }
                }
            }
        }
    })
}

#[interrupt]
fn USART2() {
    cortex_m::interrupt::free(|_| unsafe {
        if let Some(rx2) = RX2.as_mut() {
            match nb::block!(rx2.read()) {
                Ok(w) => {
                    if let Some(stdout) = STDOUT.as_mut() {
                        stdout.write(w).ok();
                    }
                }
                Err(e) => {
                    if let Some(stdout) = STDOUT.as_mut() {
                        stdout.write_fmt(format_args!("ERROR {:?}", e));
                    }
                }
            }
        }
    })
}
