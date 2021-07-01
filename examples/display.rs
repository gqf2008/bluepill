#![no_std]
#![no_main]

use bluepill::clocks::*;
use bluepill::display::ssd1306::*;
use bluepill::display::*;
use bluepill::hal::{
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

#[entry]
fn main() -> ! {
    let p = bluepill::Peripherals::take().unwrap();

    let mut flash = p.device.FLASH.constrain();
    let mut rcc = p.device.RCC.constrain();

    let clocks = rcc.cfgr.clocks(&mut flash.acr);

    let mut afio = p.device.AFIO.constrain(&mut rcc.apb2);

    let mut gpiob = p.device.GPIOB.split(&mut rcc.apb2);

    let clk = gpiob.pb6.into_open_drain_output(&mut gpiob.crl);
    let dio = gpiob.pb7.into_open_drain_output(&mut gpiob.crl);
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
    let (w, h) = display.dimensions();

    let mut tim = Timer::tim1(p.device.TIM1, &clocks, &mut rcc.apb2).start_count_down(1.mhz());
    let mut tm1637 = TM1637::new(dio, clk, &mut tim);
    tm1637.set_brightness(5);
    let mut delay = Delay::new(p.core.SYST, clocks);
    // let bmp = Bmp::from_slice(include_bytes!("./sqb.bmp")).expect("Failed to load BMP image");

    // // The image is an RGB565 encoded BMP, so specifying the type as `Image<Bmp<Rgb565>>` will read
    // // the pixels correctly
    // let im: Image<Bmp<Rgb565>> = Image::new(&bmp, Point::new(32, 0));

    // // We use the `color_converted` method here to automatically convert the RGB565 image data into
    // // BinaryColor values.
    // im.draw(&mut display.color_converted()).unwrap();

    // display.flush().unwrap();
    let raw: ImageRaw<BinaryColor> = ImageRaw::new(include_bytes!("./sqb.raw"), 120);

    Image::new(&raw, Point::new(4, 1))
        .draw(&mut display)
        .unwrap();
    display.flush().unwrap();
    let style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
    Text::with_alignment("100", Point::new(20, 60), style, Alignment::Center)
        .draw(&mut display)
        .unwrap();

    display.flush().unwrap();

    let mut colon = true;
    loop {
        if colon {
            tm1637.write(&['1', '2', '3', '4'], Some(true));
            colon = false;
        } else {
            tm1637.write(&['1', '2', '3', '4'], None);
            colon = true;
        }
        delay.delay_ms(500u32);
    }
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}
