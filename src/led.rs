use core::ops::{Deref, DerefMut};

use embedded_hal::digital::v2::{OutputPin, StatefulOutputPin};
use stm32f1xx_hal::gpio::gpioc::{CRH, PC13};
use stm32f1xx_hal::gpio::{Floating, Input};
use stm32f1xx_hal::gpio::{Output, PushPull};

pub struct Blink(PC13<Output<PushPull>>);

impl Blink {
    pub fn configure(pc13: PC13<Input<Floating>>, crh: &mut CRH) -> Self {
        Self(pc13.into_push_pull_output(crh))
    }
}

impl Deref for Blink {
    type Target = PC13<Output<PushPull>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Blink {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub trait Led {
    fn off(&mut self);

    fn on(&mut self);

    fn toggle(&mut self);
}

impl Led for Blink {
    fn off(&mut self) {
        self.set_high().unwrap();
    }

    fn on(&mut self) {
        self.set_low().unwrap();
    }
    fn toggle(&mut self) {
        if self.is_set_low().unwrap() {
            self.set_high().unwrap()
        } else {
            self.set_low().unwrap()
        }
    }
}

// #[inline]
// pub fn open() {
//     let led = unsafe { crate::peripherals::LED.as_mut_ptr() };
//     if !led.is_null() {
//         unsafe { &mut *led }.set_low().unwrap()
//     }
// }

// #[inline]
// pub fn close() {
//     let led = unsafe { crate::peripherals::LED.as_mut_ptr() };
//     if !led.is_null() {
//         unsafe { &mut *led }.set_high().unwrap()
//     }
// }

// #[inline]
// pub fn toggle() {
//     let led = unsafe { crate::peripherals::LED.as_mut_ptr() };
//     if !led.is_null() {
//         unsafe { &mut *led }.toggle().unwrap()
//     }
// }
