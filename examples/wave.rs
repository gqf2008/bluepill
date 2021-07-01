#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

use alloc::format;
use alloc::string::ToString;
use alloc_cortex_m::CortexMHeap;
use bluepill::clocks::*;
use bluepill::display::ssd1306::*;
use bluepill::hal::{
    delay::Delay,
    gpio::gpioc::PC13,
    gpio::{IOPinSpeed, Output, OutputSpeed, PushPull},
    i2c::{BlockingI2c, DutyCycle, Mode},
    pac::interrupt,
    pac::Interrupt,
    pac::{USART1, USART2},
    prelude::*,
    prelude::*,
    serial::{Config, Rx, Tx, *},
    stm32,
    time::{Instant, MonoTimer},
    timer::Timer,
};

use bluepill::sensor::HcSr04;
use core::fmt::Write;
use core::time::Duration;
use cortex_m_rt::{entry, exception, ExceptionFrame};
use embedded_graphics::{
    image::{Image, ImageRaw},
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::BinaryColor,
    pixelcolor::Rgb565,
    prelude::*,
    text::{Alignment, Text},
};
use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use panic_halt as _;
use tinybmp::Bmp;

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
    let clocks = rcc.cfgr.full_clocks(&mut flash.acr); //配置全速时钟
    let mut gpioa = p.device.GPIOA.split(&mut rcc.apb2);
    //let mut gpioc = p.device.GPIOC.split(&mut rcc.apb2);
    let mut gpiob = p.device.GPIOB.split(&mut rcc.apb2);

    ////////////////初始化设备///////////////////
    let timer = MonoTimer::new(p.core.DWT, p.core.DCB, clocks);
    let mut delay = Delay::new(p.core.SYST, clocks); //配置延时器
                                                     // let mut led = gpioc.pc13.to_led(&mut gpioc.crh); //配置LED
                                                     // let (mut tx, _) = bluepill::hal::serial::Serial::usart1(
                                                     //     p.device.USART1,
                                                     //     (
                                                     //         gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh),
                                                     //         gpioa.pa10,
                                                     //     ),
                                                     //     &mut afio.mapr,
                                                     //     Config::default().baudrate(115200.bps()),
                                                     //     clocks,
                                                     //     &mut rcc.apb2,
                                                     // )
                                                     // .split();
                                                     // let mut stdout = Stdout(&mut tx);
    let scl = gpiob.pb8.into_alternate_open_drain(&mut gpiob.crh);
    let sda = gpiob.pb9.into_alternate_open_drain(&mut gpiob.crh);

    let i2c = BlockingI2c::i2c1(
        p.device.I2C1,
        (scl, sda),
        &mut afio.mapr,
        Mode::Fast {
            frequency: 400_000.hz(),
            duty_cycle: DutyCycle::Ratio2to1,
        },
        clocks,
        &mut rcc.apb1,
        1000,
        10,
        1000,
        1000,
    );

    let mut display = Ssd1306::new(
        I2CDisplayInterface::new(i2c),
        DisplaySize128x64,
        DisplayRotation::Rotate0,
    )
    .into_buffered_graphics_mode();
    display.init().unwrap();

    let mut trigger = {
        let mut trigger = gpioa.pa0.into_push_pull_output(&mut gpioa.crl); //.into_alternate_push_pull(&mut gpioa.crl);
        trigger.set_speed(&mut gpioa.crl, IOPinSpeed::Mhz50);
        trigger
    };
    let echo = gpioa.pa1.into_pull_down_input(&mut gpioa.crl); // 下拉输入
    let mut sensor = HcSr04::new(trigger, echo, delay, timer);
    let mut tim = Timer::tim1(p.device.TIM1, &clocks, &mut rcc.apb2).start_count_down(50.hz());
    let raw: ImageRaw<BinaryColor> = ImageRaw::new(include_bytes!("./sqb.raw"), 120);

    Image::new(&raw, Point::new(4, 1))
        .draw(&mut display)
        .unwrap();
    let style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
    //im.draw(&mut display).unwrap();
    // Text::with_alignment(
    //     format_args!("{}", 100).as_str().unwrap(),
    //     Point::new(10, 40),
    //     style,
    //     Alignment::Center,
    // )
    // .draw(&mut display)
    // .unwrap();

    // writeln!(stdout, "超声波测距");
    loop {
        // led.toggle();
        let distance = sensor.measure().unwrap();
        display.clear();
        Image::new(&raw, Point::new(4, 1))
            .draw(&mut display)
            .unwrap();
        Text::with_alignment(
            format!("D: {}cm", distance.cm() as u32).as_str(),
            Point::new(64, 60),
            style,
            Alignment::Center,
        )
        .draw(&mut display)
        .unwrap();
        display.flush().unwrap();
        // writeln!(stdout, "距离:{}毫米", distance.mm());
        nb::block!(tim.wait()).ok();
    }
}

// 内存不足执行此处代码(调试用)
#[alloc_error_handler]
fn alloc_error(_layout: core::alloc::Layout) -> ! {
    cortex_m::asm::bkpt();
    loop {}
}
