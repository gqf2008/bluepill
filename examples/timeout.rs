#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;
#[macro_use(singleton)]
extern crate cortex_m;
use bluepill::clocks::*;
use bluepill::hal::delay::Delay;
use bluepill::hal::dma::{CircReadDma, Half, RxDma};
use bluepill::hal::gpio::gpioc::PC13;
use bluepill::hal::gpio::{Output, PushPull};
use bluepill::hal::{
    pac::interrupt,
    pac::Interrupt,
    pac::{USART1, USART2},
    prelude::*,
    serial::{Config, Rx, Serial, Tx},
};
use bluepill::io::TimeoutReader;
use bluepill::led::*;
use core::borrow::Borrow;
use core::cell::RefCell;
use core::fmt::Write;
use cortex_m::asm;
use stm32f1xx_hal::time::MilliSeconds;

use stm32f1xx_hal::dma::dma1::C5;
use stm32f1xx_hal::pac::TIM1;

use alloc::format;
use alloc::string::ToString;
use alloc_cortex_m::CortexMHeap;
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
    let clocks = rcc.cfgr.full_clocks(&mut flash.acr); //配置全速时钟

    let channels = p.device.DMA1.split(&mut rcc.ahb);
    let mut gpioa = p.device.GPIOA.split(&mut rcc.apb2);

    let (mut stdout, mut stdin) = bluepill::hal::serial::Serial::usart1(
        p.device.USART1,
        (
            gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh),
            gpioa.pa10,
        ),
        &mut afio.mapr,
        Config::default().baudrate(bluepill::hal::time::U32Ext::bps(115200)),
        clocks,
        &mut rcc.apb2,
    )
    .split();
    let mut timer =
        bluepill::timer::Timer::tim1(p.device.TIM1, &clocks, &mut bluepill::timer::APB2 {});
    let mut reader = TimeoutReader(&mut stdin, &mut timer);
    loop {
        match reader.read_line::<256>(5000.ms()) {
            Ok(line) => {
                stdout.write_str(line.as_str()).ok();
            }
            Err(err) => {
                stdout.write_str(format!("ERROR {}", err).as_str()).ok();
            }
        }
    }
}

// 内存不足执行此处代码(调试用)
#[alloc_error_handler]
fn alloc_error(_layout: core::alloc::Layout) -> ! {
    cortex_m::asm::bkpt();
    loop {}
}
