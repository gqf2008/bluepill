//! 烟雾传感器

#![no_main]
#![no_std]

use bluepill::hal::delay::Delay;
use bluepill::hal::gpio::gpioc::PC13;
use bluepill::hal::gpio::{Output, PushPull};
use bluepill::hal::prelude::*;
use bluepill::io::Stdout;
use bluepill::sensor::MQ2;
use bluepill::ClockConfig;
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
use panic_halt as _;

#[entry]
fn main() -> ! {
    let p = bluepill::Peripherals::take().unwrap(); //核心设备、外围设备
    let mut flash = p.device.FLASH.constrain(); //Flash
    let mut rcc = p.device.RCC.constrain(); //RCC
    let mut afio = p.device.AFIO.constrain(&mut rcc.apb2);
    let clocks = rcc.cfgr.full_clocks(&mut flash.acr); //配置全速时钟
    let mut gpioa = p.device.GPIOA.split(&mut rcc.apb2);
    let mut gpioc = p.device.GPIOC.split(&mut rcc.apb2);

    ////////////////初始化设备///////////////////
    let mut delay = Delay::new(p.core.SYST, clocks); //配置延时器
    let mut led = gpioc.pc13.to_led(&mut gpioc.crh); //配置LED

    let (mut stdout, _) = bluepill::hal::serial::Serial::usart1(
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
    stdout.to_stdout();
    //bluepill::stdout(stdout);

    let mq2 = MQ2::new(gpioa.pa6.into_pull_down_input(&mut gpioa.crl));
    sprintln!("烟雾传感器");
    loop {
        led.toggle();
        nb::block!(mq2.wait()).ok();
        sprintln!("Alart!");
        delay.delay_ms(1000u32);
    }
}

// // 内存不足执行此处代码(调试用)
// #[alloc_error_handler]
// fn oom(_layout: core::alloc::Layout) -> ! {
//     cortex_m::asm::bkpt();
//     loop {}
// }
