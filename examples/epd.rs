#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

use bluepill::clocks::ClockExt;
use bluepill::hal::delay::Delay;
use bluepill::hal::gpio::gpioc::PC13;
use bluepill::hal::gpio::{Output, PushPull};
use bluepill::hal::prelude::*;
use bluepill::hal::spi::{Mode, NoMiso, Phase, Polarity, Spi};

use bluepill::led::Led;
use bluepill::*;
use cortex_m_rt::entry;

use embedded_graphics::{
    image::{Image, ImageRaw},
    mono_font::MonoTextStyleBuilder,
    pixelcolor::BinaryColor,
    pixelcolor::PixelColor,
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle},
    text::{Baseline, Text, TextStyleBuilder},
};
use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use embedded_hal::blocking::spi::Write;
use embedded_hal::digital::v2::*;
use embedded_hal::prelude::*;
use embedded_hal::spi::{MODE_0, MODE_3};
use epd_waveshare::epd2in7b::{Display2in7b, Epd2in7b};
use epd_waveshare::{
    color::*,
    graphics::{DisplayRotation, TriDisplay},
    prelude::*,
};

// use panic_semihosting as _;
use panic_halt as _;

#[macro_use(singleton)]
extern crate cortex_m;

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
    let clocks = rcc.cfgr.clocks_72mhz(&mut flash.acr);
    let mut gpioa = p.device.GPIOA.split(&mut rcc.apb2);
    let gpiob = p.device.GPIOB.split(&mut rcc.apb2);
    let mut gpioc = p.device.GPIOC.split(&mut rcc.apb2);

    let delay = Delay::new(p.core.SYST, clocks); //配置延时器
    let mut led = Led(gpioc.pc13).ppo(&mut gpioc.crh); //配置LED
    let (stdout, _) = bluepill::serial::Serial::with_usart(p.device.USART1)
        .pins(gpioa.pa9, gpioa.pa10) //映射到引脚
        .cr(&mut gpioa.crh) //配置GPIO控制寄存器
        .clocks(clocks) //时钟
        .afio_mapr(&mut afio.mapr) //复用重映射即寄存器
        .bus(&mut rcc.apb2) //配置内核总线
        .build()
        .split();
    stdio::use_tx1(stdout);
    sprintln!("epaper");
    let busy = gpioa.pa1.into_pull_up_input(&mut gpioa.crl); //输入
    let rst = gpioa.pa2.into_push_pull_output(&mut gpioa.crl); //输出
    let dc = gpioa.pa3.into_push_pull_output(&mut gpioa.crl); //输出
    let cs = gpioa.pa4.into_push_pull_output(&mut gpioa.crl); //输出
    let pins = (
        gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl), //时钟输出
        // gpioa.pa6.into_floating_input(&mut gpioa.crl),      //输入
        NoMiso,
        gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl), //输出
    );

    let spi = Spi::spi1(
        p.device.SPI1,
        pins,
        &mut afio.mapr,
        MODE_0,
        4.mhz(),
        clocks,
        &mut rcc.apb2,
    );
    sprintln!("Epd2in7b init");
    let mut epd = bluepill::display::epd27b::EPD27b::new(spi, cs, dc, rst, busy, delay);
    epd.init();
    sprintln!("Epd2in7b init ok");
    sprintln!("clear white");
    epd.clear_white();
    sprintln!("clear white ok");
    let mut frame = Display2in7b::default();
    frame.set_rotation(DisplayRotation::Rotate90);
    draw_text_black(&mut frame, "Hello Epaper", 0, 5);
    epd.send_black(frame.buffer());
    draw_text_red(&mut frame, "\nHello Austin\nHello laopo", 0, 5);
    epd.send_red(frame.buffer());

    epd.display();
    loop {
        led.toggle();
        cortex_m::asm::delay(10000000u32);
        // delay.delay_ms(1_000u32);
    }
}

// fn draw_image_black(display: &mut Display2in7b, x: i32, y: i32) {
//     display.clear_buffer(Color::White);
//     let raw: ImageRaw<BinaryColor> = ImageRaw::new(include_bytes!("./sqb.raw"), 120);
//     Image::new(&raw, Point::new(x, y)).draw(&mut display).ok();
// }

// fn draw_image_red(display: &mut Display2in7b) {
//     display.clear_buffer(Color::Black);
//     let raw: ImageRaw<BinaryColor> = ImageRaw::new(include_bytes!("./sqb.raw"), 120);
//     Image::new(&raw, Point::new(4, 1)).draw(&mut display).ok();
// }
fn draw_text_black(display: &mut Display2in7b, text: &str, x: i32, y: i32) {
    display.clear_buffer(Color::White);
    let style = MonoTextStyleBuilder::new()
        .font(&embedded_graphics::mono_font::ascii::FONT_9X18_BOLD)
        .text_color(BinaryColor::On)
        .background_color(BinaryColor::Off)
        .build();

    let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();

    Text::with_text_style(text, Point::new(x, y), style, text_style)
        .draw(display)
        .ok();
}
fn draw_text_red(display: &mut Display2in7b, text: &str, x: i32, y: i32) {
    display.clear_buffer(Color::Black);
    let style = MonoTextStyleBuilder::new()
        .font(&embedded_graphics::mono_font::ascii::FONT_9X18_BOLD)
        .text_color(BinaryColor::Off)
        .background_color(BinaryColor::On)
        .build();
    let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();
    Text::with_text_style(text, Point::new(x, y), style, text_style)
        .draw(display)
        .ok();
}

// 内存不足执行此处代码(调试用)
#[alloc_error_handler]
fn alloc_error(_layout: core::alloc::Layout) -> ! {
    cortex_m::asm::bkpt();
    loop {}
}
