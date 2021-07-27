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

use embedded_graphics::mock_display::MockDisplay;
use embedded_graphics::{
    mono_font::MonoTextStyleBuilder,
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle},
    text::{Baseline, Text, TextStyleBuilder},
};
use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use embedded_hal::blocking::spi::Write;
use embedded_hal::digital::v2::*;
use embedded_hal::prelude::*;
use embedded_hal::spi::MODE_0;
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
    let mut gpiob = p.device.GPIOB.split(&mut rcc.apb2);
    let mut gpioc = p.device.GPIOC.split(&mut rcc.apb2);

    let mut delay = Delay::new(p.core.SYST, clocks); //配置延时器
    let mut led = Led(gpioc.pc13).ppo(&mut gpioc.crh); //配置LED
    let (mut stdout, _) = bluepill::serial::Serial::with_usart(p.device.USART1)
        .pins(gpioa.pa9, gpioa.pa10) //映射到引脚
        .cr(&mut gpioa.crh) //配置GPIO控制寄存器
        .clocks(clocks) //时钟
        .afio_mapr(&mut afio.mapr) //复用重映射即寄存器
        .bus(&mut rcc.apb2) //配置内核总线
        .build()
        .split();
    stdio::use_tx1(stdout);
    sprintln!("epaper");

    let rst = gpioa.pa1.into_push_pull_output(&mut gpioa.crl); //输出
    let dc = gpioa.pa2.into_push_pull_output(&mut gpioa.crl); //输出
    let mut busy = gpioa.pa3.into_pull_up_input(&mut gpioa.crl); //输入
    let cs = gpioa.pa4.into_push_pull_output(&mut gpioa.crl); //输出
    let pins = (
        gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl), //时钟输出
        // gpioa.pa6.into_floating_input(&mut gpioa.crl),      //输入
        NoMiso,
        gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl), //输出
    );

    let mut spi = Spi::spi1(
        p.device.SPI1,
        pins,
        &mut afio.mapr,
        MODE_0,
        4.mhz(),
        clocks,
        &mut rcc.apb2,
    );
    sprintln!("Epd2in7b init");
    let mut epd =
        Epd2in7b::new(&mut spi, cs, busy, dc, rst, &mut delay).expect("eink initalize error");
    sprintln!("Epd2in7b init ok");
    sprintln!("black_white display init");
    // // the bw-buffer of this tri-color screen
    let mut mono_display = Display2in7b::default();
    sprintln!("black_white display init ok");
    let _ = Line::new(Point::new(10, 10), Point::new(10, 20))
        .into_styled(PrimitiveStyle::with_stroke(Black, 1))
        .draw(&mut mono_display);

    sprintln!("red display init");
    // // Use a second display for red/yellow
    let mut chromatic_display = Display2in7b::default();
    sprintln!("red display init ok");
    // // // We use `Black` but it will be shown as red/yellow
    if let Err(err) = Line::new(Point::new(15, 120), Point::new(15, 200))
        .into_styled(PrimitiveStyle::with_stroke(Black, 1))
        .draw(&mut chromatic_display)
    {
        sprintln!("error")
    }

    epd.update_achromatic_frame(&mut spi, mono_display.buffer());
    epd.update_chromatic_frame(&mut spi, chromatic_display.buffer());
    //epd.update_frame(&mut spi, mono_display.buffer(), &mut delay);
    epd.display_frame(&mut spi, &mut delay).ok();

    // // // Set the EPD to sleep
    //epd.sleep(&mut spi, &mut delay).ok();

    loop {
        led.toggle();
        delay.delay_ms(1_000u32);
        //epd.wake_up(&mut spi, &mut delay).ok();
        sprintln!("clear_frame {}", epd.is_busy());
        epd.clear_frame(&mut spi, &mut delay).ok();
        sprintln!("draw line1 {}", epd.is_busy());
        Line::new(Point::new(10, 10), Point::new(10, 20))
            .into_styled(PrimitiveStyle::with_stroke(Black, 1))
            .draw(&mut mono_display)
            .ok();
        sprintln!("draw line2 {}", epd.is_busy());
        Line::new(Point::new(15, 120), Point::new(15, 200))
            .into_styled(PrimitiveStyle::with_stroke(Black, 1))
            .draw(&mut chromatic_display)
            .ok();
        sprintln!("update_achromatic_frame {}", epd.is_busy());
        epd.update_achromatic_frame(&mut spi, mono_display.buffer());
        sprintln!("update_chromatic_frame {}", epd.is_busy());
        epd.update_chromatic_frame(&mut spi, chromatic_display.buffer());
        sprintln!("display_frame {}", epd.is_busy());
        epd.display_frame(&mut spi, &mut delay).ok();
        //epd.sleep(&mut spi, &mut delay).ok();
    }
}

// fn draw_text(display: &mut Display2in7b, text: &str, x: i32, y: i32) {
//     let style = MonoTextStyleBuilder::new()
//         .font(&embedded_graphics::mono_font::ascii::FONT_6X10)
//         .text_color(TriColor::White)
//         .background_color(TriColor::Black)
//         .build();

//     let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();

//     let _ = Text::with_text_style(text, Point::new(x, y), style, text_style).draw(display);
// }

// 内存不足执行此处代码(调试用)
#[alloc_error_handler]
fn alloc_error(_layout: core::alloc::Layout) -> ! {
    cortex_m::asm::bkpt();
    loop {}
}
