//! 定时器
//!

#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

use alloc_cortex_m::CortexMHeap;
use bluepill::hal::delay::Delay;
use bluepill::hal::gpio::gpioc::PC13;
use bluepill::hal::gpio::{Output, PushPull};
use bluepill::hal::pac::{TIM1, TIM2};
use bluepill::hal::prelude::*;
use bluepill::hal::serial::Config;
use bluepill::hal::timer::CountDownTimer;
use bluepill::hal::timer::Timer;
use bluepill::io::*;
use bluepill::*;
use core::cell::RefCell;
use core::ops::MulAssign;
use cortex_m::{asm::wfi, interrupt::Mutex};
use cortex_m_rt::entry;
use embedded_hal::timer::Cancel;
use panic_semihosting as _;
use stm32f1xx_hal::pac::interrupt;
use stm32f1xx_hal::pac::Interrupt;
use stm32f1xx_hal::timer::Event;

/// 堆内存分配器
#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();
/// 堆内存 16K
const HEAP_SIZE: usize = 16384;

#[entry]
fn main() -> ! {
    unsafe {
        ALLOCATOR.init(cortex_m_rt::heap_start() as usize, HEAP_SIZE);
    }
    let p = bluepill::Peripherals::take().unwrap(); //核心设备、外围设备
    let mut flash = p.device.FLASH.constrain(); //Flash
    let mut rcc = p.device.RCC.constrain(); //RCC
    let mut afio = p.device.AFIO.constrain(&mut rcc.apb2);
    let clocks = rcc.cfgr.full_clocks(&mut flash.acr); //配置全速时钟
                                                       //let mut delay = Delay::new(cp.SYST, clocks); //配置延时器
    let mut gpioa = p.device.GPIOA.split(&mut rcc.apb2);
    let mut gpioc = p.device.GPIOC.split(&mut rcc.apb2);

    ////////////////初始化设备///////////////////
    let mut delay1 = Delay::new(p.core.SYST, clocks); //配置延时器
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

    let mut led = gpioc.pc13.to_led(&mut gpioc.crh); //配置LED
    let mut timer = Timer::tim1(p.device.TIM1, &clocks, &mut rcc.apb2).start_count_down(1.hz());
    let mut delay = Timer::tim2(p.device.TIM2, &clocks, &mut rcc.apb1).start_count_down(1.hz());
    //delay.start(3000.ms());
    //let mut timer = Timer::syst(cp.SYST, &clocks).start_count_down(1.hz());
    timer.listen(Event::Update);
    delay.listen(Event::Update);

    cortex_m::interrupt::free(|cs| {
        *BLINK.borrow(cs).borrow_mut() = Some(led);
        *DELAY.borrow(cs).borrow_mut() = Some(delay);
        *TIMER.borrow(cs).borrow_mut() = Some(timer);
    });

    bluepill::enable_interrupt(Interrupt::TIM1_UP);
    bluepill::enable_interrupt(Interrupt::TIM2);

    sprintln!("hello timer led");
    loop {
        sprintln!("listen timer");
        cortex_m::interrupt::free(|cs| {
            // Move TIMER pin here, leaving a None in its place
            TIMER
                .borrow(cs)
                .borrow_mut()
                .as_mut()
                .unwrap()
                .listen(Event::Update);
        });
        delay1.delay_ms(5000u32);
        sprintln!("unlisten timer");
        cortex_m::interrupt::free(|cs| {
            // Move TIMER pin here, leaving a None in its place
            TIMER
                .borrow(cs)
                .borrow_mut()
                .as_mut()
                .unwrap()
                .unlisten(Event::Update);
        });
        delay1.delay_ms(5000u32);
        //cortex_m::asm::wfi();
    }
}

// 内存不足执行此处代码(调试用)
#[alloc_error_handler]
fn alloc_error(_layout: core::alloc::Layout) -> ! {
    cortex_m::asm::bkpt();
    loop {}
}

#[interrupt]
unsafe fn TIM2() {
    static mut LED: Option<PC13<Output<PushPull>>> = None;
    static mut TIM: Option<CountDownTimer<TIM2>> = None;

    let led = LED.get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs| {
            // Move LED pin here, leaving a None in its place
            BLINK.borrow(cs).replace(None).unwrap()
        })
    });

    let tim = TIM.get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs| {
            // Move DELAY pin here, leaving a None in its place
            DELAY.borrow(cs).replace(None).unwrap()
        })
    });

    led.toggle();
    tim.wait().ok();
}

static mut COUNT: u32 = 0;
static BLINK: Mutex<RefCell<Option<PC13<Output<PushPull>>>>> = Mutex::new(RefCell::new(None));
static DELAY: Mutex<RefCell<Option<CountDownTimer<TIM2>>>> = Mutex::new(RefCell::new(None));
static TIMER: Mutex<RefCell<Option<CountDownTimer<TIM1>>>> = Mutex::new(RefCell::new(None));

#[interrupt]
unsafe fn TIM1_UP() {
    // static mut TIM: Option<CountDownTimer<TIM1>> = None;
    // let tim = TIM.get_or_insert_with(|| {
    //     cortex_m::interrupt::free(|cs| {
    //         // Move TIMER pin here, leaving a None in its place
    //         TIMER.borrow(cs).borrow_mut().unwrap().wait().ok(); //replace(None).unwrap()
    //     })
    // });
    cortex_m::interrupt::free(|_| unsafe {
        COUNT += 10;
    });
    sprintln!("COUNT {}", unsafe { COUNT });
    unsafe { COUNT = 0 };
    //tim.wait().ok();
    cortex_m::interrupt::free(|cs| {
        // Move TIMER pin here, leaving a None in its place
        TIMER.borrow(cs).borrow_mut().as_mut().unwrap().wait().ok(); //replace(None).unwrap()
    });
}
