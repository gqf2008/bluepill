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
    pixelcolor::BinaryColor,
    prelude::*,
};
use embedded_hal::blocking::delay::DelayUs;
use panic_halt as _;

#[entry]
fn main() -> ! {
    let p = bluepill::Peripherals::take().unwrap();

    let mut flash = p.device.FLASH.constrain();
    let mut rcc = p.device.RCC.constrain();

    let clocks = rcc.cfgr.clocks(&mut flash.acr);

    let mut afio = p.device.AFIO.constrain(&mut rcc.apb2);

    let mut gpiob = p.device.GPIOB.split(&mut rcc.apb2);

    let mut clk = gpiob.pb6.into_open_drain_output(&mut gpiob.crl);
    let mut dio = gpiob.pb7.into_open_drain_output(&mut gpiob.crl);
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
    let mut delay = Delay::new(p.core.SYST, clocks);

    let raw: ImageRaw<BinaryColor> = ImageRaw::new(include_bytes!("./rust.raw"), 64);

    let im = Image::new(&raw, Point::new(32, 0));

    im.draw(&mut display).unwrap();

    display.flush().unwrap();

    let mut colon = true;
    loop {
        if colon {
            tm1637.write(&[DIGIT[1], DIGIT[2], DIGIT[3], DIGIT[4]], Some(true));
            colon = false;
        } else {
            tm1637.write(&[DIGIT[1], DIGIT[2], DIGIT[3], DIGIT[4]], None);
            colon = true;
        }
        delay.delay_ms(500u32);
    }
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}
