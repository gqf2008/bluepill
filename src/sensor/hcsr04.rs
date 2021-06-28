//!超声波测距传感器

use crate::hal::time::MonoTimer;
use crate::sprintln;
use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::digital::v2::{InputPin, OutputPin};

#[derive(Debug)]
pub enum Error {
    Timeout,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match *self {
            Error::Timeout => write!(f, "Timeout waiting for sensor"),
        }
    }
}

type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Copy, Clone)]
pub struct Distance(f64);

impl Distance {
    pub fn cm(&self) -> f64 {
        self.0 / 10.0
    }
    pub fn mm(&self) -> f64 {
        self.0
    }
}

pub struct HcSr04<Triger, Echo, Delay> {
    trig: Triger,
    echo: Echo,
    delay: Delay,
    timer: MonoTimer,
}

impl<Triger, Echo, Delay> HcSr04<Triger, Echo, Delay>
where
    Triger: OutputPin,
    Echo: InputPin,
    Delay: DelayUs<u32>,
{
    pub fn new(pin: (Triger, Echo), delay: Delay, timer: MonoTimer) -> Self {
        let mut trig = pin.0;
        trig.set_low().ok();
        let echo = pin.1;
        HcSr04 {
            trig,
            echo,
            delay,
            timer,
        }
    }

    pub fn measure(&mut self) -> Result<Distance> {
        let mut sum = 0f64;
        (0..5).into_iter().for_each(|_| {
            sum += self.measure1().unwrap();
            self.delay.delay_us(60000u32);
        });
        Ok(Distance(sum / 5.0))
    }

    fn measure1(&mut self) -> Result<f64> {
        //发送信号
        self.trig.set_high().ok();
        self.delay.delay_us(20u32);
        self.trig.set_low().ok();
        let start_wait = self.timer.now();
        //等高电平
        while let Ok(true) = self.echo.is_low() {
            if start_wait.elapsed() > self.timer.frequency().0 {
                return Err(Error::Timeout);
            }
        }
        //等低电平（高电平持续的时间就是信号往返的时间）
        let start_instant = self.timer.now();
        while let Ok(true) = self.echo.is_high() {
            if start_instant.elapsed() > self.timer.frequency().0 {
                return Err(Error::Timeout);
            }
        }
        let ticks = start_instant.elapsed();
        Ok(ticks as f64 / self.timer.frequency().0 as f64 * 340.0 / 2.0 * 1000.0)
    }
}
