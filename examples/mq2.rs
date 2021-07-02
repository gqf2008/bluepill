//! 烟雾传感器

#![no_main]
#![no_std]

use bluepill::clocks::*;
use bluepill::display::*;
use bluepill::hal::delay::Delay;
use bluepill::hal::gpio::gpioc::PC13;
use bluepill::hal::gpio::{Output, PushPull};
use bluepill::hal::prelude::*;
use bluepill::io::Stdout;
use bluepill::led::*;
use bluepill::sensor::MQ2;
use bluepill::*;
use core::cell::RefCell;
use core::fmt::Write;
use core::ops::MulAssign;
use cortex_m::{asm::wfi, interrupt::Mutex};
use cortex_m_rt::entry;
use embedded_hal::digital::v2::InputPin;
use hal::{
    gpio::gpioa::PA6,
    gpio::ExtiPin,
    gpio::IOPinSpeed,
    gpio::Input,
    gpio::OutputSpeed,
    gpio::PullDown,
    pac::interrupt,
    pac::Interrupt,
    pac::{USART1, USART2},
    prelude::*,
    timer::Timer,
};
use heapless::Vec;
use panic_halt as _;

#[entry]
fn main() -> ! {
    let p = bluepill::Peripherals::take().unwrap(); //核心设备、外围设备
    let mut flash = p.device.FLASH.constrain(); //Flash
    let mut rcc = p.device.RCC.constrain(); //RCC
    let mut afio = p.device.AFIO.constrain(&mut rcc.apb2);
    let clocks = rcc.cfgr.clocks_72mhz(&mut flash.acr); //配置全速时钟
    let mut gpioa = p.device.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = p.device.GPIOB.split(&mut rcc.apb2);
    //let mut gpioc = p.device.GPIOC.split(&mut rcc.apb2);

    ////////////////初始化设备///////////////////
    let clk = gpiob.pb6.into_open_drain_output(&mut gpiob.crl); //开漏输出
    let dio = gpiob.pb7.into_open_drain_output(&mut gpiob.crl);
    let mut delay = Delay::new(p.core.SYST, clocks); //配置延时器
    let mut tim = Timer::tim1(p.device.TIM1, &clocks, &mut rcc.apb2).start_count_down(1.mhz());
    let mut tm1637 = TM1637::new(dio, clk, &mut tim);
    tm1637.set_brightness(5);
    tm1637.write(&['-', '-', '-', '-'], Some(true));
    let mq2 = MQ2::new(gpioa.pa6.into_pull_down_input(&mut gpioa.crl));
    let mut colon = true;
    loop {
        nb::block!(mq2.wait()).ok();
        (0..10).into_iter().for_each(|_| {
            if colon {
                tm1637.write(&['E', 'E', 'E', 'E'], Some(true));
                colon = false;
            } else {
                tm1637.write(&['E', 'E', 'E', 'E'], None);
                colon = true;
            }
            delay.delay_ms(100u32);
        });
        delay.delay_ms(5000u32);
    }
}
