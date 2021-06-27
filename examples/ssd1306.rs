#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;
use alloc_cortex_m::CortexMHeap;
use bluepill::display::*;
use bluepill::hal::{
    i2c::{BlockingI2c, DutyCycle, Mode},
    prelude::*,
    stm32,
};
use cortex_m_rt::{entry, exception, ExceptionFrame};
use embedded_graphics::{
    image::{Image, ImageRaw},
    pixelcolor::BinaryColor,
    prelude::*,
};
use panic_semihosting as _;

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
    let p = bluepill::Peripherals::take().unwrap();

    let mut flash = p.device.FLASH.constrain();
    let mut rcc = p.device.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut afio = p.device.AFIO.constrain(&mut rcc.apb2);

    let mut gpiob = p.device.GPIOB.split(&mut rcc.apb2);

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

    let raw: ImageRaw<BinaryColor> = ImageRaw::new(include_bytes!("./rust.raw"), 64);

    let im = Image::new(&raw, Point::new(32, 0));

    im.draw(&mut display).unwrap();

    display.flush().unwrap();

    loop {}
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
