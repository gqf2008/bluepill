//! STM32F103C8T6板级支持包

#![no_std]
pub mod clocks;
pub mod example;
pub mod gpio;
pub mod led;
pub mod lte;
pub mod net;
pub mod serial;
pub mod stdout;
pub mod ultrasonic_wave;
pub mod wifi;

use hal::pac::Interrupt;
pub use net::*;
pub use serial::*;
pub use stdout::configure as configure_stdout;
pub use stm32f1xx_hal as hal;
pub struct Peripherals {
    pub cp: cortex_m::Peripherals,
    pub dp: stm32f1xx_hal::pac::Peripherals,
}

impl Peripherals {
    pub fn take() -> (cortex_m::Peripherals, stm32f1xx_hal::pac::Peripherals) {
        (
            cortex_m::Peripherals::take().unwrap(),
            stm32f1xx_hal::pac::Peripherals::take().unwrap(),
        )
    }
}

pub fn enable_interrupt(interrupt: Interrupt) {
    sprintln!("打开{:?}中断", interrupt);
    unsafe {
        cortex_m::peripheral::NVIC::unmask(interrupt);
    }
}

pub fn disable_interrupt(interrupt: Interrupt) {
    sprintln!("关闭{:?}中断", interrupt);
    cortex_m::peripheral::NVIC::mask(interrupt);
}
