//! STM32F103C8T6 BSP
#![no_std]

extern crate alloc;

pub mod clocks;
pub mod display;
pub mod gpio;
pub mod io;
pub mod led;
pub mod net;
pub mod rng;
pub mod sensor;
pub mod serial;
pub mod stdio;
pub mod timer;

pub use stm32f1xx_hal as hal;

use core::cell::RefCell;

use cortex_m::interrupt::Mutex;
use stm32f1xx_hal::adc::Adc;
use stm32f1xx_hal::pac::ADC1;

static ADC: Mutex<RefCell<Option<Adc<ADC1>>>> = Mutex::new(RefCell::new(None));

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

pub fn init_adc(adc: Adc<ADC1>) {
    cortex_m::interrupt::free(|cs| {
        ADC.borrow(cs).replace(Some(adc));
    })
}

//读取芯片温度
pub fn chip_temp() -> Option<i32> {
    cortex_m::interrupt::free(|cs| {
        if let Some(adc) = ADC.borrow(cs).borrow_mut().as_mut() {
            Some(adc.read_temp())
        } else {
            None
        }
    })
}
