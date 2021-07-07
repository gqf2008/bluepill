//!参考hal库实现基于时间的定时器

use stm32f1xx_hal::pac::{TIM1, TIM2, TIM3, TIM4};
use stm32f1xx_hal::rcc::{Clocks, APB1, APB2};
use stm32f1xx_hal::timer::Timer;
pub struct TimerBuilder<'a, TIM, BUS> {
    tim: Option<TIM>,
    clocks: Option<Clocks>,
    apb: Option<&'a mut BUS>,
}

impl<'a, TIM, BUS> TimerBuilder<'a, TIM, BUS> {
    pub fn with_tim(tim: TIM) -> Self {
        Self {
            tim: Some(tim),
            clocks: None,
            apb: None,
        }
    }

    //配置时钟
    pub fn clocks(mut self, clocks: Clocks) -> Self {
        self.clocks = Some(clocks);
        self
    }
    //配置内核总线
    pub fn bus(mut self, apb: &'a mut BUS) -> Self {
        self.apb = Some(apb);
        self
    }
}

macro_rules! timerbuild {
    ($(
        $(#[$meta:meta])*
        $TIMX:ident: (
            $timX:ident,
            $APBX:ident
        ),
    )+) => {
        $(
            $(#[$meta])*
            impl<'a> TimerBuilder<'a, $TIMX, $APBX> {
                pub fn build(
                    self,
                ) -> Timer<$TIMX> {
                    Timer::$timX(self.tim.unwrap(), &self.clocks.unwrap(), self.apb.unwrap())
                }
            }
        )+
    }
}

timerbuild! {
    TIM1: (tim1,APB2),
    TIM2: (tim2,APB1),
    TIM3: (tim3,APB1),
    TIM4: (tim4,APB1),
}

// pub struct Timer<TIM> {
//     pub(crate) tim: TIM,
//     pub(crate) clk: Hertz,
// }

// pub struct AHB;

// impl AHB {
//     // TODO remove `allow`
//     #[allow(dead_code)]
//     pub(crate) fn enr(&mut self) -> &AHBENR {
//         unsafe { &(*RCC::ptr()).ahbenr }
//     }
// }

// pub struct APB1;

// impl APB1 {
//     pub(crate) fn enr(&mut self) -> &APB1ENR {
//         unsafe { &(*RCC::ptr()).apb1enr }
//     }

//     pub(crate) fn rstr(&mut self) -> &APB1RSTR {
//         unsafe { &(*RCC::ptr()).apb1rstr }
//     }
// }

// impl APB1 {
//     pub fn set_pwren(&mut self) {
//         self.enr().modify(|_r, w| w.pwren().set_bit())
//     }
// }

// pub struct APB2;

// impl APB2 {
//     pub(crate) fn enr(&mut self) -> &APB2ENR {
//         unsafe { &(*RCC::ptr()).apb2enr }
//     }

//     pub(crate) fn rstr(&mut self) -> &APB2RSTR {
//         unsafe { &(*RCC::ptr()).apb2rstr }
//     }
// }

// impl Timer<TIM1> {
//     pub fn tim1(tim: TIM1, clocks: &Clocks) -> Self {
//         let mut apb = APB2 {};
//         &mut apb.enr().modify(|_, w| w.tim1en().set_bit());
//         &mut apb.rstr().modify(|_, w| w.tim1rst().set_bit());
//         &mut apb.rstr().modify(|_, w| w.tim1rst().clear_bit());
//         Self {
//             tim,
//             clk: clocks.pclk2_tim(),
//         }
//     }

//     pub fn reset(&mut self) {
//         self.tim.cr1.modify(|_, w| w.urs().set_bit());

//         self.tim.egr.write(|w| w.ug().set_bit());
//         self.tim.cr1.modify(|_, w| w.urs().clear_bit());
//     }

//     pub fn restart_raw(&mut self, psc: u16, arr: u16) {
//         self.tim.cr1.modify(|_, w| w.cen().clear_bit());
//         self.tim.psc.write(|w| w.psc().bits(psc));
//         #[allow(unused_unsafe)]
//         self.tim.arr.write(|w| unsafe { w.arr().bits(arr) });
//         self.reset();
//         self.tim.cr1.modify(|_, w| w.cen().set_bit());
//     }
// }

// impl CountDown for Timer<TIM1> {
//     type Time = MilliSeconds;
//     fn wait(&mut self) -> nb::Result<(), void::Void> {
//         if self.tim.sr.read().uif().bit_is_clear() {
//             Err(nb::Error::WouldBlock)
//         } else {
//             self.tim.sr.modify(|_, w| w.uif().clear_bit());
//             Ok(())
//         }
//     }
//     fn start<T>(&mut self, timeout: T)
//     where
//         T: Into<MilliSeconds>,
//     {
//         let (psc, arr) = compute_arr_presc(timeout.into().0, self.clk.0);
//         self.restart_raw(psc, arr);
//     }
// }

// impl Timer<TIM2> {
//     pub fn tim2(tim: TIM2, clocks: &Clocks) -> Self {
//         let mut apb = APB1 {};
//         apb.enr().modify(|_, w| w.tim2en().set_bit());
//         apb.rstr().modify(|_, w| w.tim2rst().set_bit());
//         apb.rstr().modify(|_, w| w.tim2rst().clear_bit());
//         Self {
//             tim,
//             clk: clocks.pclk2_tim(),
//         }
//     }

//     /// Resets the counter
//     pub fn reset(&mut self) {
//         self.tim.cr1.modify(|_, w| w.urs().set_bit());

//         self.tim.egr.write(|w| w.ug().set_bit());
//         self.tim.cr1.modify(|_, w| w.urs().clear_bit());
//     }

//     pub fn restart_raw(&mut self, psc: u16, arr: u16) {
//         self.tim.cr1.modify(|_, w| w.cen().clear_bit());
//         self.tim.psc.write(|w| w.psc().bits(psc));
//         #[allow(unused_unsafe)]
//         self.tim.arr.write(|w| unsafe { w.arr().bits(arr) });
//         self.reset();
//         self.tim.cr1.modify(|_, w| w.cen().set_bit());
//     }
// }

// impl CountDown for Timer<TIM2> {
//     type Time = MilliSeconds;
//     fn wait(&mut self) -> nb::Result<(), void::Void> {
//         if self.tim.sr.read().uif().bit_is_clear() {
//             Err(nb::Error::WouldBlock)
//         } else {
//             self.tim.sr.modify(|_, w| w.uif().clear_bit());
//             Ok(())
//         }
//     }
//     fn start<T>(&mut self, timeout: T)
//     where
//         T: Into<MilliSeconds>,
//     {
//         let (psc, arr) = compute_arr_presc(timeout.into().0, self.clk.0);
//         self.restart_raw(psc, arr);
//     }
// }

// impl Timer<TIM3> {
//     pub fn tim3(tim: TIM3, clocks: &Clocks) -> Self {
//         let mut apb = APB1 {};
//         apb.enr().modify(|_, w| w.tim3en().set_bit());
//         apb.rstr().modify(|_, w| w.tim3rst().set_bit());
//         apb.rstr().modify(|_, w| w.tim3rst().clear_bit());
//         Self {
//             tim,
//             clk: clocks.pclk2_tim(),
//         }
//     }

//     /// Resets the counter
//     pub fn reset(&mut self) {
//         self.tim.cr1.modify(|_, w| w.urs().set_bit());

//         self.tim.egr.write(|w| w.ug().set_bit());
//         self.tim.cr1.modify(|_, w| w.urs().clear_bit());
//     }

//     pub fn restart_raw(&mut self, psc: u16, arr: u16) {
//         self.tim.cr1.modify(|_, w| w.cen().clear_bit());
//         self.tim.psc.write(|w| w.psc().bits(psc));
//         #[allow(unused_unsafe)]
//         self.tim.arr.write(|w| unsafe { w.arr().bits(arr) });
//         self.reset();
//         self.tim.cr1.modify(|_, w| w.cen().set_bit());
//     }
// }

// impl CountDown for Timer<TIM3> {
//     type Time = MilliSeconds;
//     fn wait(&mut self) -> nb::Result<(), void::Void> {
//         if self.tim.sr.read().uif().bit_is_clear() {
//             Err(nb::Error::WouldBlock)
//         } else {
//             self.tim.sr.modify(|_, w| w.uif().clear_bit());
//             Ok(())
//         }
//     }
//     fn start<T>(&mut self, timeout: T)
//     where
//         T: Into<MilliSeconds>,
//     {
//         let (psc, arr) = compute_arr_presc(timeout.into().0, self.clk.0);
//         self.restart_raw(psc, arr);
//     }
// }

// impl Timer<TIM4> {
//     pub fn tim4(tim: TIM4, clocks: &Clocks) -> Self {
//         let mut apb = APB1 {};
//         apb.enr().modify(|_, w| w.tim4en().set_bit());
//         apb.rstr().modify(|_, w| w.tim4rst().set_bit());
//         apb.rstr().modify(|_, w| w.tim4rst().clear_bit());
//         Self {
//             tim,
//             clk: clocks.pclk2_tim(),
//         }
//     }

//     /// Resets the counter
//     pub fn reset(&mut self) {
//         self.tim.cr1.modify(|_, w| w.urs().set_bit());

//         self.tim.egr.write(|w| w.ug().set_bit());
//         self.tim.cr1.modify(|_, w| w.urs().clear_bit());
//     }

//     pub fn restart_raw(&mut self, psc: u16, arr: u16) {
//         self.tim.cr1.modify(|_, w| w.cen().clear_bit());
//         self.tim.psc.write(|w| w.psc().bits(psc));
//         #[allow(unused_unsafe)]
//         self.tim.arr.write(|w| unsafe { w.arr().bits(arr) });
//         self.reset();
//         self.tim.cr1.modify(|_, w| w.cen().set_bit());
//     }
// }

// impl CountDown for Timer<TIM4> {
//     type Time = MilliSeconds;
//     fn wait(&mut self) -> nb::Result<(), void::Void> {
//         if self.tim.sr.read().uif().bit_is_clear() {
//             Err(nb::Error::WouldBlock)
//         } else {
//             self.tim.sr.modify(|_, w| w.uif().clear_bit());
//             Ok(())
//         }
//     }
//     fn start<T>(&mut self, timeout: T)
//     where
//         T: Into<MilliSeconds>,
//     {
//         let (psc, arr) = compute_arr_presc(timeout.into().0, self.clk.0);
//         self.restart_raw(psc, arr);
//     }
// }

// #[inline(always)]
// fn compute_arr_presc(timeout: u32, clock: u32) -> (u16, u16) {
//     let ticks = clock / 1_000 * timeout;
//     let psc = ((ticks - 1) / (1 << 16)) as u16;
//     let arr = (ticks / (psc + 1) as u32) as u16;
//     (psc, arr)
// }
