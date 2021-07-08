#![no_main]
#![no_std]
#![feature(alloc_error_handler)]
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
    pac::USART1,
    prelude::*,
    serial::{Config, Rx, Serial, Tx},
};
use bluepill::led::*;
use core::borrow::Borrow;
use core::cell::RefCell;
use core::fmt::Write;
use cortex_m::asm;
use stm32f1xx_hal::dma::dma1::C5;

use cortex_m::{asm::wfi, interrupt::Mutex};
use cortex_m_rt::entry;
use embedded_dma::ReadBuffer;
use heapless::spsc::{Consumer, Producer, Queue};
use heapless::Vec;
use panic_halt as _;

use alloc_cortex_m::CortexMHeap;

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

    let channels = p.device.DMA1.split(&mut rcc.ahb);
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
    let mut rx = stdin.with_dma(channels.5);
    read_dma1(&mut stdout, rx);
    loop {}
}

fn read_dma_circ(stdout: &mut Tx<USART1>, rx: RxDma<Rx<USART1>, C5>) {
    let mut rx = rx;
    let buf = singleton!(: [[u8; 8]; 2] = [[0; 8]; 2]).unwrap();
    let mut s: heapless::String<16384> = heapless::String::new();
    read_dma(stdout, rx)
}

fn read_dma1(stdout: &mut Tx<USART1>, rx: RxDma<Rx<USART1>, C5>) -> ! {
    let mut rx = rx;
    let mut buf = singleton!(: [u8; 8]  = [0; 8]).unwrap();
    let mut s: heapless::String<4096> = heapless::String::new();
    loop {
        let t = rx.read(buf);
        while !t.is_done() {}
        let (out, _rx) = t.wait();
        out.iter().for_each(|b| {
            let c = *b as char;
            if c == '\n' {
                stdout.write_fmt(format_args!("{}", s));
                s.clear();
            } else {
                if s.len() == 4096 {
                    stdout.write_fmt(format_args!("{}", s));
                    s.clear();
                }
                s.push(c).ok();
            }
        });
        rx = _rx;
        buf = out;
        //led.toggle();
    }
}

fn read_dma(stdout: &mut Tx<USART1>, rx: RxDma<Rx<USART1>, C5>) {
    let mut rx = rx;
    let mut buf = singleton!(: [u8; 1]  = [0; 1]).unwrap();
    let mut s: heapless::String<4096> = heapless::String::new();
    loop {
        let (out, _rx) = rx.read(buf).wait();
        out.iter().for_each(|b| {
            let c = *b as char;
            if c == '\n' {
                //s.push(c).ok();
                stdout.write_fmt(format_args!("{}", s));
                s.clear();
            } else {
                s.push(*b as char).ok();
            }
        });
        if s.len() == 4096 {
            stdout.write_fmt(format_args!("{}", s));
            s.clear();
        }

        rx = _rx;
        buf = out;
        //led.toggle();
    }
}
// 内存不足执行此处代码(调试用)
#[alloc_error_handler]
fn alloc_error(_layout: core::alloc::Layout) -> ! {
    cortex_m::asm::bkpt();
    loop {}
}
