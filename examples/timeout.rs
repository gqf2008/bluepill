#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

use alloc::format;
use alloc::string::ToString;
use alloc_cortex_m::CortexMHeap;
use bluepill::clocks::ClockExt;
use bluepill::hal::{
    delay::Delay,
    dma::dma1::C5,
    gpio::gpioc::PC13,
    gpio::{Output, PushPull},
    pac::interrupt,
    pac::Interrupt,
    pac::TIM1,
    pac::{USART1, USART2},
    prelude::*,
    time::MilliSeconds,
};
use bluepill::io::TimeoutReader;
use bluepill::led::Led;
use bluepill::timer::Timer;
use core::borrow::Borrow;
use core::cell::RefCell;
use core::fmt::Write;
use cortex_m::asm;
use cortex_m::{asm::wfi, interrupt::Mutex};
use cortex_m_rt::entry;
use embedded_dma::ReadBuffer;
use embedded_hal::timer::CountDown;
use heapless::spsc::{Consumer, Producer, Queue};
use heapless::Vec;
use panic_halt as _;

/// 堆内存分配器
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

    //let mut delay = Delay::new(p.core.SYST, clocks); //配置延时器
    //let mut led = Led(gpioc.pc13).ppo(&mut gpioc.crh); //配置LED
    let (mut stdout, _) = bluepill::serial::Serial::with_usart(p.device.USART1)
        .pins(gpioa.pa9, gpioa.pa10) //映射到引脚
        .cr(&mut gpioa.crh) //配置GPIO控制寄存器
        .clocks(clocks) //时钟
        .afio_mapr(&mut afio.mapr) //复用重映射即寄存器
        .bus(&mut rcc.apb2) //配置内核总线
        .build()
        .split();
    let (mut tx, mut rx) = bluepill::serial::Serial::with_usart(p.device.USART2)
        .pins(gpioa.pa2, gpioa.pa3) //映射到引脚
        .cr(&mut gpioa.crl) //配置GPIO控制寄存器
        .clocks(clocks) //时钟
        .afio_mapr(&mut afio.mapr) //复用重映射
        .bus(&mut rcc.apb1) //配置内核总线
        .build()
        .split();
    let mut apb2 = bluepill::timer::APB2 {};
    let mut timer = Timer::tim1(p.device.TIM1, &clocks, &mut apb2);

    let mut reader = TimeoutReader(&mut rx, &mut timer);

    loop {
        tx.write_str("AT+GMR\r\n").ok();
        loop {
            match reader.read_line::<256>(5000.ms()) {
                Ok(line) => {
                    stdout.write_str(line.as_str()).ok();
                    if line.starts_with("OK") || line.starts_with("ERROR") {
                        break;
                    }
                }
                Err(err) => {
                    stdout.write_str(format!("ERROR {}\r\n", err).as_str()).ok();
                    break;
                }
            };
        }
    }
}

// 内存不足执行此处代码(调试用)
#[alloc_error_handler]
fn alloc_error(_layout: core::alloc::Layout) -> ! {
    cortex_m::asm::bkpt();
    loop {}
}
