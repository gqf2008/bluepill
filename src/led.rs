use embedded_hal::digital::v2::{OutputPin, StatefulOutputPin, ToggleableOutputPin};
use stm32f1xx_hal::gpio::gpioa::{self, *};
use stm32f1xx_hal::gpio::gpiob::{self, *};
use stm32f1xx_hal::gpio::gpioc::{self, *};
use stm32f1xx_hal::gpio::gpiod::{self, *};
use stm32f1xx_hal::gpio::gpioe::{self, *};
use stm32f1xx_hal::gpio::{Floating, Input, Output, PushPull};

pub struct Led<PIN>(pub PIN);

impl<T> Led<T>
where
    T: OutputPin + StatefulOutputPin + ToggleableOutputPin,
{
    pub fn opp(pin: T) -> Self {
        Self(pin)
    }

    pub fn off(&mut self) {
        self.0.set_high().ok();
    }

    pub fn on(&mut self) {
        self.0.set_low().ok();
    }

    pub fn is_on(&mut self) -> bool {
        self.0.is_set_low().ok().unwrap()
    }

    pub fn is_off(&mut self) -> bool {
        self.0.is_set_high().ok().unwrap()
    }

    pub fn toggle(&mut self) {
        self.0.toggle().ok();
    }
}

macro_rules! led_pin {
    ($(
        $(#[$meta:meta])*
        $PINX:ident: (
            $gpioX:ident,
            $CR:ident
        ),
    )+) => {
        $(
            $(#[$meta])*
            impl Led<$gpioX::$PINX<Input<Floating>>> {
                pub fn ppo(self, cr: &mut $gpioX::$CR) -> Led<$PINX<Output<PushPull>>> {
                    Led::opp(self.0.into_push_pull_output(cr))
                }
            }
        )+
    }
}

led_pin! {
    PA0: (gpioa, CRL),
    PA1: (gpioa, CRL),
    PA2: (gpioa, CRL),
    PA3: (gpioa, CRL),
    PA4: (gpioa, CRL),
    PA5: (gpioa, CRL),
    PA6: (gpioa, CRL),
    PA7: (gpioa, CRL),
    PA8: (gpioa, CRH),
    PA9: (gpioa, CRH),
    PA10: (gpioa, CRH),
    PA11: (gpioa, CRH),
    PA12: (gpioa, CRH),
    PA13: (gpioa, CRH),
    PA14: (gpioa, CRH),
    PA15: (gpioa, CRH),
}

led_pin! {
    PB0: (gpiob, CRL),
    PB1: (gpiob, CRL),
    PB2: (gpiob, CRL),
    PB3: (gpiob, CRL),
    PB4: (gpiob, CRL),
    PB5: (gpiob, CRL),
    PB6: (gpiob, CRL),
    PB7: (gpiob, CRL),
    PB8: (gpiob, CRH),
    PB9: (gpiob, CRH),
    PB10: (gpiob, CRH),
    PB11: (gpiob, CRH),
    PB12: (gpiob, CRH),
    PB13: (gpiob, CRH),
    PB14: (gpiob, CRH),
    PB15: (gpiob, CRH),
}

led_pin! {
    PC0: (gpioc,CRL),
   PC1: (gpioc,CRL),
   PC2: (gpioc,CRL),
   PC3: (gpioc,CRL),
   PC4: (gpioc,CRL),
   PC5: (gpioc,CRL),
   PC6: (gpioc,CRL),
   PC7: (gpioc,CRL),
   PC8: (gpioc,CRH),
   PC9: (gpioc,CRH),
   PC10: (gpioc,CRH),
   PC11: (gpioc,CRH),
   PC12: (gpioc,CRH),
   PC13: (gpioc,CRH),
}

led_pin! {
    PD0: (gpiod, CRL),
    PD1: (gpiod, CRL),
    PD2: (gpiod, CRL),
    PD3: (gpiod, CRL),
    PD4: (gpiod, CRL),
    PD5: (gpiod, CRL),
    PD6: (gpiod, CRL),
    PD7: (gpiod, CRL),
    PD8: (gpiod, CRH),
    PD9: (gpiod, CRH),
    PD10: (gpiod, CRH),
    PD11: (gpiod, CRH),
    PD12: (gpiod, CRH),
    PD13: (gpiod, CRH),
    PD14: (gpiod, CRH),
    PD15: (gpiod, CRH),
}

led_pin! {
    PE0: (gpioe, CRL),
    PE1: (gpioe, CRL),
    PE2: (gpioe, CRL),
    PE3: (gpioe, CRL),
    PE4: (gpioe, CRL),
    PE5: (gpioe, CRL),
    PE6: (gpioe, CRL),
    PE7: (gpioe, CRL),
    PE8: (gpioe, CRH),
    PE9: (gpioe, CRH),
    PE10: (gpioe, CRH),
    PE11: (gpioe, CRH),
    PE12: (gpioe, CRH),
    PE13: (gpioe, CRH),
    PE14: (gpioe, CRH),
    PE15: (gpioe, CRH),
}
