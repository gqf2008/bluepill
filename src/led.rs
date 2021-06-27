use embedded_hal::digital::v2::OutputPin;
use stm32f1xx_hal::gpio::gpioc::PC13;
use stm32f1xx_hal::gpio::{Output, PushPull};

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
