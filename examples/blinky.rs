#![no_main]
#![no_std]

use bluepill::clocks::ClockExt;
use bluepill::hal::delay::Delay;
use bluepill::hal::gpio::gpioc::PC13;
use bluepill::hal::gpio::{Output, PushPull};
use bluepill::hal::prelude::*;
use bluepill::led::Led;
use cortex_m_rt::entry;
use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use panic_semihosting as _;

#[entry]
fn main() -> ! {
    let p = bluepill::Peripherals::take().unwrap(); //核心设备、外围设备
    let mut flash = p.device.FLASH.constrain(); //Flash
    let mut rcc = p.device.RCC.constrain(); //RCC
    let clocks = rcc.cfgr.clocks_72mhz(&mut flash.acr);
    let mut delay = Delay::new(p.core.SYST, clocks); //配置延时器
    let mut gpioc = p.device.GPIOC.split(&mut rcc.apb2);
    let mut led = Led(gpioc.pc13).ppo(&mut gpioc.crh); //配置LED

    loop {
        led.toggle();
        delay.delay_ms(1_000u32);
    }
}
