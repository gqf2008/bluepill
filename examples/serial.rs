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
use bluepill::io::*;
use bluepill::led::Led;
use bluepill::*;
use core::fmt::Write;
use cortex_m_rt::entry;
use hal::{
    pac::interrupt,
    pac::Interrupt,
    pac::{USART1, USART2},
    prelude::*,
    serial::{Config, Rx, Tx, *},
};
use heapless::Vec;
use panic_halt as _;

static mut STDIN: Option<Rx<USART1>> = None;
static mut STDOUT: Option<Tx<USART1>> = None;
static mut TX2: Option<Tx<USART2>> = None;
static mut RX2: Option<Rx<USART2>> = None;

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

    let mut delay = Delay::new(p.core.SYST, clocks); //配置延时器
    let mut led = Led(gpioc.pc13).ppo(&mut gpioc.crh); //配置LED

    let (mut stdout, mut stdin) = bluepill::serial::Serial::with_usart(p.device.USART1)
        .pins(gpioa.pa9, gpioa.pa10) //映射到引脚
        .cr(&mut gpioa.crh) //配置GPIO控制寄存器
        .clocks(clocks) //时钟
        .afio_mapr(&mut afio.mapr) //复用重映射即寄存器
        .bus(&mut rcc.apb2) //配置内核总线
        .build()
        .split();
    let (mut tx2, mut rx2) = bluepill::serial::Serial::with_usart(p.device.USART2)
        .pins(gpioa.pa2, gpioa.pa3)
        .cr(&mut gpioa.crl)
        .clocks(clocks)
        .afio_mapr(&mut afio.mapr)
        .bus(&mut rcc.apb1)
        .build()
        .split();

    stdin.listen();
    rx2.listen();

    stdout.write_str("Hello ESP8266\n");
    cortex_m::interrupt::free(|_| unsafe {
        STDOUT.replace(stdout);
        STDIN.replace(stdin);
        TX2.replace(tx2);
        RX2.replace(rx2);
    });
    //开启USART1、USART2中断
    bluepill::enable_interrupt(stm32f1xx_hal::pac::Interrupt::USART1);
    bluepill::enable_interrupt(stm32f1xx_hal::pac::Interrupt::USART2);
    loop {
        led.toggle();
        delay.delay_ms(1_000u32);
    }
}

//#[interrupt]
fn USART1() {
    cortex_m::interrupt::free(|_| unsafe {
        if let Some(stdin) = STDIN.as_mut() {
            match nb::block!(stdin.read()) {
                Ok(w) => {
                    if let Some(tx2) = TX2.as_mut() {
                        tx2.write(w).ok();
                    }
                }
                Err(e) => {
                    if let Some(stdout) = STDOUT.as_mut() {
                        stdout.write_fmt(format_args!("ERROR {:?}", e));
                    }
                }
            }
        }
    })
}

//#[interrupt]
fn USART2() {
    cortex_m::interrupt::free(|_| unsafe {
        if let Some(rx2) = RX2.as_mut() {
            match nb::block!(rx2.read()) {
                Ok(w) => {
                    if let Some(stdout) = STDOUT.as_mut() {
                        stdout.write(w).ok();
                    }
                }
                Err(e) => {
                    if let Some(stdout) = STDOUT.as_mut() {
                        stdout.write_fmt(format_args!("ERROR {:?}", e));
                    }
                }
            }
        }
    })
}

// 内存不足执行此处代码(调试用)
#[alloc_error_handler]
fn alloc_error(_layout: core::alloc::Layout) -> ! {
    cortex_m::asm::bkpt();
    loop {}
}
