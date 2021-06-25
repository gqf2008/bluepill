//! 通过串口连接ESP8266模块，发送AT指令联网
//!

#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

use alloc_cortex_m::CortexMHeap;
use bluepill::hal::delay::Delay;
use bluepill::hal::prelude::*;
use bluepill::led::{Blink, Led};
use bluepill::sprintln;
use cortex_m_rt::entry;
use embedded_hal::blocking::delay::{DelayMs, DelayUs};
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
    let (cp, dp) = bluepill::Peripherals::take(); //核心设备、外围设备
    let mut flash = dp.FLASH.constrain(); //Flash
    let mut rcc = dp.RCC.constrain(); //RCC
    let clocks = bluepill::clocks::init_full_clocks(rcc.cfgr, &mut flash.acr); //配置全速时钟
    let mut delay = Delay::new(cp.SYST, clocks); //配置延时器
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
    let mut led = Blink::configure(gpioc.pc13, &mut gpioc.crh); //配置LED
                                                                //esp8266::init();

    sprintln!("hello bluepill led");

    loop {
        led.toggle();
        delay.delay_ms(1_000u32);
    }
}

// 内存不足执行此处代码(调试用)
#[alloc_error_handler]
fn alloc_error(_layout: core::alloc::Layout) -> ! {
    cortex_m::asm::bkpt();
    loop {}
}
