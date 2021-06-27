use embedded_hal::digital::v2::OutputPin;
use stm32f1xx_hal::gpio::gpioc::{CRH, PC13};

use stm32f1xx_hal::gpio::{Floating, Input, Output, PushPull};

pub trait ToLed {
    fn to_led(self, cr: &mut CRH) -> PC13<Output<PushPull>>;
}

impl ToLed for PC13<Input<Floating>> {
    fn to_led(self, cr: &mut CRH) -> PC13<Output<PushPull>> {
        self.into_push_pull_output(cr)
    }
}
impl Led for PC13<Output<PushPull>> {
    fn off(&mut self) {
        self.set_high().unwrap();
    }

    fn on(&mut self) {
        self.set_low().unwrap();
    }
}

pub trait Led {
    fn off(&mut self);

    fn on(&mut self);
}
