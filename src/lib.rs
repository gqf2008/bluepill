//! STM32F103C8T6 BSP

#![no_std]
pub mod clocks;
pub mod display;
pub mod flash;
pub mod gpio;
pub mod io;
pub mod led;
pub mod net;
pub mod sensor;

pub use clocks::*;
pub use led::*;
pub use net::*;
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
    sprintln!("打开{:?}中断", interrupt);
    unsafe {
        cortex_m::peripheral::NVIC::unmask(interrupt);
    }
}

pub fn disable_interrupt(interrupt: hal::pac::Interrupt) {
    sprintln!("关闭{:?}中断", interrupt);
    cortex_m::peripheral::NVIC::mask(interrupt);
}
