//! STM32F103C8T6 BSP
#![no_std]

pub mod clocks;
pub mod display;
pub mod gpio;
pub mod io;
pub mod led;
pub mod net;
pub mod sensor;
pub mod serial;
pub mod stdio;
pub mod timer;

pub use stm32f1xx_hal as hal;

pub struct Peripherals {
    pub core: cortex_m::Peripherals,
    pub device: stm32f1xx_hal::pac::Peripherals,
}
static mut TAKEN: bool = false;

impl Peripherals {
    pub fn take() -> Option<Self> {
        cortex_m::interrupt::free(|_| {
            if unsafe { TAKEN } {
                None
            } else {
                unsafe { TAKEN = true };
                let core = cortex_m::Peripherals::take().unwrap();
                let device = stm32f1xx_hal::pac::Peripherals::take().unwrap();
                Some(Self { core, device })
            }
        })
    }
}

pub fn enable_interrupt(interrupt: hal::pac::Interrupt) {
    unsafe {
        cortex_m::peripheral::NVIC::unmask(interrupt);
    }
}

pub fn disable_interrupt(interrupt: hal::pac::Interrupt) {
    cortex_m::peripheral::NVIC::mask(interrupt);
}
