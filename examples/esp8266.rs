//! 通过串口连接ESP8266模块，发送AT指令联网
//!

#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

#[macro_use(singleton)]
extern crate cortex_m;

use alloc_cortex_m::CortexMHeap;
use bluepill::clocks::*;
use bluepill::hal::delay::Delay;
use bluepill::hal::gpio::gpioc::PC13;
use bluepill::hal::gpio::{Output, PushPull};
use bluepill::hal::prelude::*;
use bluepill::hal::timer::Timer;
use bluepill::io::*;
use bluepill::net::esp826601s;
use bluepill::stdio;
use bluepill::timer::TimerBuilder;
use bluepill::*;
use bluepill::*;
use core::borrow::Borrow;
use core::cell::RefCell;
use core::fmt::Write;
use cortex_m::asm;
use cortex_m::{asm::wfi, interrupt::Mutex};
use cortex_m_rt::entry;
use hal::{
    pac::interrupt,
    pac::Interrupt,
    pac::{USART1, USART2},
    prelude::*,
    serial::{Config, Rx, Tx, *},
};
use heapless::spsc::{Consumer, Producer, Queue};
use heapless::String;
use heapless::Vec;
use panic_halt as _;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();
/// 堆内存 8K
const HEAP_SIZE: usize = 8192;

fn init() {
    unsafe {
        ALLOCATOR.init(cortex_m_rt::heap_start() as usize, HEAP_SIZE);
    }
}

#[entry]
fn main() -> ! {
    init();
    let p = bluepill::Peripherals::take().unwrap(); //核心设备、外围设备
    let mut flash = p.device.FLASH.constrain(); //Flash
    let mut rcc = p.device.RCC.constrain(); //RCC
    let mut afio = p.device.AFIO.constrain(&mut rcc.apb2);
    let clocks = rcc.cfgr.clocks_72mhz(&mut flash.acr); //配置全速时钟
    let mut gpioa = p.device.GPIOA.split(&mut rcc.apb2);
    let mut gpioc = p.device.GPIOC.split(&mut rcc.apb2);

    let mut delay = Delay::new(p.core.SYST, clocks);

    let (mut stdout, _) = bluepill::serial::Serial::with_usart(p.device.USART1)
        .pins(gpioa.pa9, gpioa.pa10) //映射到引脚
        .cr(&mut gpioa.crh) //配置GPIO控制寄存器
        .clocks(clocks) //时钟
        .afio_mapr(&mut afio.mapr) //复用重映射即寄存器
        .bus(&mut rcc.apb2) //配置内核总线
        .build()
        .split();
    stdio::use_tx1(stdout);
    sprintln!("build serial");
    let port = bluepill::serial::Serial::with_usart(p.device.USART2)
        .pins(gpioa.pa2, gpioa.pa3) //映射到引脚
        .cr(&mut gpioa.crl) //配置GPIO控制寄存器
        .clocks(clocks) //时钟
        .afio_mapr(&mut afio.mapr) //复用重映射
        .bus(&mut rcc.apb1) //配置内核总线
        .build_rw();
    let timer = TimerBuilder::with_tim(p.device.TIM1)
        .clocks(clocks)
        .bus(&mut rcc.apb2)
        .build()
        .start_count_down(1.hz());
    sprintln!("build serial ok");
    sprintln!("new esp826601s");
    let mut wifi = esp826601s::Esp8266::new(port, timer);
    sprintln!("new esp826601s ok");
    wifi.hello().ok();
    wifi.hangup().ok();
    wifi.dial("Wosai-Guest", "Shouqianba$520", false).ok();
    match wifi.device_info() {
        Ok(inf) => sprint!("{}", inf),
        Err(bluepill::io::Error::Other(err)) => sprint!("{:?}", err),
        Err(err) => {
            sprintln!("{:?}", err)
        }
    }
    match wifi.ifconfig() {
        Ok(inf) => sprint!("{}", inf),
        Err(bluepill::io::Error::Other(err)) => sprint!("{:?}", err),
        Err(err) => {
            sprintln!("{:?}", err)
        }
    }

    loop {
        if let Ok(reply) = wifi.net_state() {
            sprint!("{}", reply);
        }
        delay.delay_ms(5000u32);
    }
}

// 内存不足执行此处代码(调试用)
#[alloc_error_handler]
fn alloc_error(_layout: core::alloc::Layout) -> ! {
    cortex_m::asm::bkpt();
    loop {}
}
