//! 通过串口连接ESP8266模块，发送AT指令联网
//!

#![no_main]
#![no_std]

use bluepill::clocks::*;
use bluepill::hal::delay::Delay;
use bluepill::hal::gpio::gpioc::PC13;
use bluepill::hal::gpio::{Output, PushPull};
use bluepill::hal::prelude::*;
use bluepill::led::*;

use bluepill::*;
use core::borrow::Borrow;
use core::cell::RefCell;
use core::fmt::Write;
use cortex_m::asm;
use cortex_m::{asm::wfi, interrupt::Mutex};
use cortex_m_rt::entry;
use hal::{
    pac::interrupt,
    pac::Interrupt,
    pac::{USART1, USART2},
    prelude::*,
    serial::{Config, Rx, Tx, *},
};
use heapless::spsc::{Consumer, Producer, Queue};
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
    let mut gpioc = p.device.GPIOC.split(&mut rcc.apb2);

    let mut delay = Delay::new(p.core.SYST, clocks); //配置延时器
    let mut led = Led(gpioc.pc13).ppo(&mut gpioc.crh); //配置LED

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
    let (mut tx2, mut rx2) = bluepill::hal::serial::Serial::usart2(
        p.device.USART2,
        (
            gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl),
            gpioa.pa3,
        ),
        &mut afio.mapr,
        Config::default().baudrate(115200.bps()),
        clocks,
        &mut rcc.apb1,
    )
    .split();

    rx2.listen();
    let mut consumer = unsafe { Q.split().1 };
    cortex_m::interrupt::free(|cs| {
        *RX2.borrow(cs).borrow_mut() = Some(rx2);
    });

    //开启USART1中断
    bluepill::enable_interrupt(stm32f1xx_hal::pac::Interrupt::USART2);

    stdout.write_str("Hello ESP8266\n");
    loop {
        led.toggle();
        if let Some(msg) = consumer.dequeue() {
            match msg {
                Message::Byte(w) => {
                    stdout.write(w).ok();
                }
                Message::Error(err) => {
                    stdout.write_fmt(format_args!("error {:?}", err)).ok();
                }
            }
        } else {
            asm::wfi();
        }
    }
}

static RX2: Mutex<RefCell<Option<Rx<USART2>>>> = Mutex::new(RefCell::new(None));
static mut Q: Queue<Message, 4096> = Queue::new();
#[derive(Debug)]
enum Message {
    Byte(u8),
    Error(bluepill::hal::serial::Error),
}
#[interrupt]
unsafe fn USART2() {
    static mut RX: Option<Rx<USART2>> = None;
    let mut producer = unsafe { Q.split().0 };

    let rx2 = RX.get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs| RX2.borrow(cs).replace(None).unwrap())
    });

    // let queue = QUEUE.get_or_insert_with(|| {
    //     cortex_m::interrupt::free(|cs| STREAM.borrow(cs).replace(None).unwrap())
    // });
    let msg = match nb::block!(rx2.read()) {
        Ok(w) => Message::Byte(w),
        Err(e) => Message::Error(e),
    };
    producer.enqueue(msg).ok();
    // cortex_m::interrupt::free(|cs| {
    //     if let Some(rx2) = RX2.as_mut() {
    //         let msg = match nb::block!(rx2.read()) {
    //             Ok(w) => Message::Byte(w),
    //             Err(e) => Message::Error(e),
    //         };
    //         STREAM.borrow(cs).borrow_mut().enqueue(msg).ok();
    //     }
    // })
}
