#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

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
use bluepill::led::*;
use core::borrow::Borrow;
use core::cell::RefCell;
use core::fmt::Write;
use cortex_m::asm;

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

#[entry]
fn main() -> ! {
    unsafe {
        ALLOCATOR.init(cortex_m_rt::heap_start() as usize, HEAP_SIZE);
    }

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

    let mut apb2 = APB2 { _0: () };
    let mut timer = Timer::tim1(p.device.TIM1, &clocks, &mut apb2);

    let mut reader = TimeoutReader(&mut rx, &mut timer);

    loop {
        tx.write_str("AT+GMR\n").ok();
        match reader.read_line::<256>(5000.ms()) {
            Ok(line) => stdout.write_str(line.as_str()).ok(),
            Err(err) => stdout.write_str(format!("ERROR {}\r\n", err).as_str()).ok(),
        };
    }
}

// 内存不足执行此处代码(调试用)
#[alloc_error_handler]
fn alloc_error(_layout: core::alloc::Layout) -> ! {
    cortex_m::asm::bkpt();
    loop {}
}

use stm32f1xx_hal::pac::RCC;
use stm32f1xx_hal::rcc::{Clocks, Enable, GetBusFreq, Reset};
use stm32f1xx_hal::stm32::rcc::{APB2ENR, APB2RSTR};
use stm32f1xx_hal::time::Hertz;
pub struct Timer<TIM> {
    pub(crate) tim: TIM,
    pub(crate) clk: Hertz,
}
pub struct APB2 {
    _0: (),
}

impl APB2 {
    pub(crate) fn enr(&mut self) -> &APB2ENR {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*RCC::ptr()).apb2enr }
    }

    pub(crate) fn rstr(&mut self) -> &APB2RSTR {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*RCC::ptr()).apb2rstr }
    }
}

impl Timer<TIM1> {
    pub fn tim1(tim: TIM1, clocks: &Clocks, apb: &mut APB2) -> Self {
        apb.enr().modify(|_, w| w.tim1en().set_bit());
        apb.rstr().modify(|_, w| w.tim1rst().set_bit());
        apb.rstr().modify(|_, w| w.tim1rst().clear_bit());
        Self {
            tim,
            clk: clocks.pclk2_tim(),
        }
    }

    /// Resets the counter
    pub fn reset(&mut self) {
        // Sets the URS bit to prevent an interrupt from being triggered by
        // the UG bit
        self.tim.cr1.modify(|_, w| w.urs().set_bit());

        self.tim.egr.write(|w| w.ug().set_bit());
        self.tim.cr1.modify(|_, w| w.urs().clear_bit());
    }

    pub fn restart_raw(&mut self, psc: u16, arr: u16) {
        // pause
        self.tim.cr1.modify(|_, w| w.cen().clear_bit());

        self.tim.psc.write(|w| w.psc().bits(psc));

        // TODO: Remove this `allow` once this field is made safe for stm32f100
        #[allow(unused_unsafe)]
        self.tim.arr.write(|w| unsafe { w.arr().bits(arr) });

        // Trigger an update event to load the prescaler value to the clock
        self.reset();

        // start counter
        self.tim.cr1.modify(|_, w| w.cen().set_bit());
    }
}

#[inline(always)]
fn compute_arr_presc(timeout: u32, clock: u32) -> (u16, u16) {
    let ticks = clock / 1_000 * timeout;
    let psc = ((ticks - 1) / (1 << 16)) as u16;
    let arr = (ticks / (psc + 1) as u32) as u16;
    (psc, arr)
}

impl CountDown for Timer<TIM1> {
    type Time = MilliSeconds;
    fn wait(&mut self) -> nb::Result<(), void::Void> {
        if self.tim.sr.read().uif().bit_is_clear() {
            Err(nb::Error::WouldBlock)
        } else {
            self.tim.sr.modify(|_, w| w.uif().clear_bit());
            Ok(())
        }
    }
    fn start<T>(&mut self, timeout: T)
    where
        T: Into<MilliSeconds>,
    {
        let (psc, arr) = compute_arr_presc(timeout.into().0, self.clk.0);
        self.restart_raw(psc, arr);
    }
}
