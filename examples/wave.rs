#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::fmt::Write;
use core::time::Duration;

use alloc_cortex_m::CortexMHeap;
use bluepill::hal::delay::Delay;
use bluepill::hal::gpio::gpioc::PC13;
use bluepill::hal::gpio::{Output, PushPull};
use bluepill::hal::prelude::*;
use bluepill::led::Led;
use bluepill::sensor::HcSr04;
use bluepill::*;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use hal::{
    gpio::{IOPinSpeed, OutputSpeed},
    pac::interrupt,
    pac::Interrupt,
    pac::{USART1, USART2},
    prelude::*,
    serial::{Config, Rx, Tx, *},
    time::{Instant, MonoTimer},
    timer::Timer,
};
use heapless::Vec;
use panic_semihosting as _;
/// 堆内存分配器
#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();
/// 堆内存 12K
const HEAP_SIZE: usize = 1024 * 12;

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
    let timer = MonoTimer::new(p.core.DWT, p.core.DCB, clocks);
    let mut delay = Delay::new(p.core.SYST, clocks); //配置延时器
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh); //配置LED
    let (mut tx, _) = bluepill::hal::serial::Serial::usart1(
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
    tx.to_stdout();
    let mut trigger = gpioa.pa0.into_push_pull_output(&mut gpioa.crl); //.into_alternate_push_pull(&mut gpioa.crl);
    trigger.set_speed(&mut gpioa.crl, IOPinSpeed::Mhz50);
    let echo = gpioa.pa1.into_pull_down_input(&mut gpioa.crl); // 下拉输入
    let mut sensor = HcSr04::new((trigger, echo), delay, timer);
    let mut tim = Timer::tim1(p.device.TIM1, &clocks, &mut rcc.apb2).start_count_down(1.hz());

    sprintln!("超声波测距");
    loop {
        led.toggle();
        let distance = sensor.measure().unwrap();
        sprintln!("距离:{}毫米", distance.mm());
        nb::block!(tim.wait()).ok();
    }
}

// 内存不足执行此处代码(调试用)
#[alloc_error_handler]
fn oom(_layout: core::alloc::Layout) -> ! {
    cortex_m::asm::bkpt();
    loop {}
}
