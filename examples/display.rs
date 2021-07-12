#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

use bluepill::clocks::*;
use bluepill::display::ssd1306::*;
use bluepill::display::*;
use bluepill::hal::{
    adc,
    delay::Delay,
    i2c::{BlockingI2c, DutyCycle, Mode},
    prelude::*,
    stm32,
    timer::Timer,
};

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
use panic_halt as _;
use tinybmp::Bmp;

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
    let p = bluepill::Peripherals::take().unwrap();
    let mut flash = p.device.FLASH.constrain();
    let mut rcc = p.device.RCC.constrain();
    let clocks = rcc.cfgr.clocks_48mhz(&mut flash.acr);
    let mut afio = p.device.AFIO.constrain(&mut rcc.apb2);
    let mut gpiob = p.device.GPIOB.split(&mut rcc.apb2);
    let clk = gpiob.pb6.into_open_drain_output(&mut gpiob.crl);
    let dio = gpiob.pb7.into_open_drain_output(&mut gpiob.crl);
    let scl = gpiob.pb8.into_alternate_open_drain(&mut gpiob.crh);
    let sda = gpiob.pb9.into_alternate_open_drain(&mut gpiob.crh);
    let mut adc1 = adc::Adc::adc1(p.device.ADC1, &mut rcc.apb2, clocks);
    bluepill::init_adc(adc1);
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
    let (w, h) = display.dimensions();

    let mut delay = Delay::new(p.core.SYST, clocks);

    let raw: ImageRaw<BinaryColor> = ImageRaw::new(include_bytes!("./sqb.raw"), 120);

    Image::new(&raw, Point::new(4, 1))
        .draw(&mut display)
        .unwrap();
    display.flush().unwrap();
    let style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
    loop {
        let temp = bluepill::chip_temp().unwrap();
        display.clear();
        Image::new(&raw, Point::new(4, 1))
            .draw(&mut display)
            .unwrap();
        Text::with_alignment(
            alloc::format!("Temp: {}C", temp).as_str(),
            Point::new(45, 60),
            style,
            Alignment::Center,
        )
        .draw(&mut display)
        .unwrap();
        display.flush().unwrap();
        delay.delay_ms(500u32);
    }
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}
// 内存不足执行此处代码(调试用)
#[alloc_error_handler]
fn alloc_error(_layout: core::alloc::Layout) -> ! {
    cortex_m::asm::bkpt();
    loop {}
}
