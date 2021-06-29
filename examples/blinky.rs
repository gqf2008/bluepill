//! 通过串口连接ESP8266模块，发送AT指令联网

#![no_main]
#![no_std]

use bluepill::hal::delay::Delay;
use bluepill::hal::gpio::gpioc::PC13;
use bluepill::hal::gpio::{Output, PushPull};
use bluepill::hal::prelude::*;
use bluepill::*;
use cortex_m_rt::entry;
use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use panic_semihosting as _;

#[entry]
fn main() -> ! {
    let p = bluepill::Peripherals::take().unwrap(); //核心设备、外围设备
    let mut flash = p.device.FLASH.constrain(); //Flash
    let mut rcc = p.device.RCC.constrain(); //RCC
    let clocks = rcc.cfgr.full_clocks(&mut flash.acr);
    let mut delay = Delay::new(p.core.SYST, clocks); //配置延时器
    let mut gpioc = p.device.GPIOC.split(&mut rcc.apb2);
    let mut led = gpioc.pc13.to_led(&mut gpioc.crh); //配置LED
                                                     //esp8266::init();

    loop {
        led.toggle();
        delay.delay_ms(1_000u32);
    }
}

// 内存不足执行此处代码(调试用)
// #[alloc_error_handler]
// fn alloc_error(_layout: core::alloc::Layout) -> ! {
//     cortex_m::asm::bkpt();
//     loop {}
// }
