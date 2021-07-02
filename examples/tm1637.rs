#![no_std]
#![no_main]

use bluepill::clocks::*;
use bluepill::hal::delay::Delay;
use bluepill::hal::prelude::*;
use bluepill::hal::timer::Timer;
use embedded_hal::blocking::delay::DelayUs;
use panic_halt as _;

use bluepill::display::*;
use cortex_m_rt::entry;

struct NoDelay {}
impl DelayUs<u16> for NoDelay {
    fn delay_us(&mut self, us: u16) {}
}

#[entry]
fn main() -> ! {
    let p = bluepill::Peripherals::take().unwrap();

    let mut rcc = p.device.RCC.constrain();

    let mut gpiob = p.device.GPIOB.split(&mut rcc.apb2);

    let mut flash = p.device.FLASH.constrain();
    let clocks = rcc.cfgr.clocks_72mhz(&mut flash.acr);
    let mut clk = gpiob.pb6.into_open_drain_output(&mut gpiob.crl);
    let mut dio = gpiob.pb7.into_open_drain_output(&mut gpiob.crl);

    let mut tim = Timer::tim1(p.device.TIM1, &clocks, &mut rcc.apb2).start_count_down(1.mhz());
    let mut tm1637 = TM1637::new(dio, clk, &mut tim);
    let mut delay = Delay::new(p.core.SYST, clocks);
    let mut a = [1, 2, 3, 4];

    // // 最高位设置为1时显示 数码管上的":" 符号
    // unsigned char disp_num[] = {0x3F, 0x06 | 0x80, 0x5B, 0x4F, 0x66, 0x6D};
    let mut colon = true;
    loop {
        if colon {
            tm1637.write(&['1', '2', '3', '4'], Some(true));
            colon = false;
        } else {
            tm1637.write(&['1', '2', '3', '4'], None);
            colon = true;
        }
        delay.delay_ms(500u32);
    }
}
