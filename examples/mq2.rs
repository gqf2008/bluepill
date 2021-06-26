//! 通过串口连接ESP8266模块，发送AT指令联网
//!

#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::fmt::Write;

use alloc_cortex_m::CortexMHeap;
use bluepill::hal::delay::Delay;
use bluepill::hal::prelude::*;
use bluepill::led::{Blink, Led};
use bluepill::sensor::MQ2;
use bluepill::serial::BufRead;
use bluepill::*;
use core::cell::RefCell;
use core::ops::MulAssign;
use cortex_m::{asm::wfi, interrupt::Mutex};
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use embedded_hal::digital::v2::InputPin;
use hal::gpio::gpioa::PA6;
use hal::gpio::ExtiPin;
use hal::gpio::PullDown;
use hal::{
    gpio::IOPinSpeed,
    gpio::Input,
    gpio::OutputSpeed,
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
/// 堆内存 16K
const HEAP_SIZE: usize = 16384;

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

    ////////////////初始化设备///////////////////
    let mut delay = Delay::new(p.core.SYST, clocks); //配置延时器
    let mut led = Blink::configure(gpioc.pc13, &mut gpioc.crh); //配置LED

    let (mut stdout, _) = bluepill::serial::usart1(
        p.device.USART1,
        (gpioa.pa9, gpioa.pa10),
        &mut afio.mapr,
        Config::default().baudrate(115200.bps()),
        clocks,
        &mut rcc.apb2,
        &mut gpioa.crh,
    );
    bluepill::stdout(stdout);

    let mut aout = gpioa.pa6.into_pull_down_input(&mut gpioa.crl);
    let mq2 = MQ2::new(aout);
    sprintln!("烟雾传感器");
    loop {
        led.toggle();
        nb::block!(mq2.wait()).ok();
        sprintln!("Alart!");
        delay.delay_ms(1000u32);
    }
}

// 内存不足执行此处代码(调试用)
#[alloc_error_handler]
fn oom(_layout: core::alloc::Layout) -> ! {
    cortex_m::asm::bkpt();
    loop {}
}
