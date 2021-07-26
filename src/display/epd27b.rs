use crate::sprintln;
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::spi::Write;
use embedded_hal::digital::v2::*;

const WIDTH: usize = 264;
const HEIGHT: usize = 176;
const WIDE: usize = WIDTH / 8;

const LUT_VCOM_DC: [u8; 44] = [
    0x00, 0x00, 0x00, 0x1A, 0x1A, 0x00, 0x00, 0x01, 0x00, 0x0A, 0x0A, 0x00, 0x00, 0x08, 0x00, 0x0E,
    0x01, 0x0E, 0x01, 0x10, 0x00, 0x0A, 0x0A, 0x00, 0x00, 0x08, 0x00, 0x04, 0x10, 0x00, 0x00, 0x05,
    0x00, 0x03, 0x0E, 0x00, 0x00, 0x0A, 0x00, 0x23, 0x00, 0x00, 0x00, 0x01,
];

const LUT_WW: [u8; 42] = [
    0x90, 0x1A, 0x1A, 0x00, 0x00, 0x01, 0x40, 0x0A, 0x0A, 0x00, 0x00, 0x08, 0x84, 0x0E, 0x01, 0x0E,
    0x01, 0x10, 0x80, 0x0A, 0x0A, 0x00, 0x00, 0x08, 0x00, 0x04, 0x10, 0x00, 0x00, 0x05, 0x00, 0x03,
    0x0E, 0x00, 0x00, 0x0A, 0x00, 0x23, 0x00, 0x00, 0x00, 0x01,
];

//# R22H    r
const LUT_BW: [u8; 42] = [
    0xA0, 0x1A, 0x1A, 0x00, 0x00, 0x01, 0x00, 0x0A, 0x0A, 0x00, 0x00, 0x08, 0x84, 0x0E, 0x01, 0x0E,
    0x01, 0x10, 0x90, 0x0A, 0x0A, 0x00, 0x00, 0x08, 0xB0, 0x04, 0x10, 0x00, 0x00, 0x05, 0xB0, 0x03,
    0x0E, 0x00, 0x00, 0x0A, 0xC0, 0x23, 0x00, 0x00, 0x00, 0x01,
];

//# R23H    w
const LUT_BB: [u8; 42] = [
    0x90, 0x1A, 0x1A, 0x00, 0x00, 0x01, 0x40, 0x0A, 0x0A, 0x00, 0x00, 0x08, 0x84, 0x0E, 0x01, 0x0E,
    0x01, 0x10, 0x80, 0x0A, 0x0A, 0x00, 0x00, 0x08, 0x00, 0x04, 0x10, 0x00, 0x00, 0x05, 0x00, 0x03,
    0x0E, 0x00, 0x00, 0x0A, 0x00, 0x23, 0x00, 0x00, 0x00, 0x01,
];
// # R24H    b
const LUT_WB: [u8; 42] = [
    0x90, 0x1A, 0x1A, 0x00, 0x00, 0x01, 0x20, 0x0A, 0x0A, 0x00, 0x00, 0x08, 0x84, 0x0E, 0x01, 0x0E,
    0x01, 0x10, 0x10, 0x0A, 0x0A, 0x00, 0x00, 0x08, 0x00, 0x04, 0x10, 0x00, 0x00, 0x05, 0x00, 0x03,
    0x0E, 0x00, 0x00, 0x0A, 0x00, 0x23, 0x00, 0x00, 0x00, 0x01,
];

pub struct EPD27b<SPI, IN, OUT, DELAY> {
    spi: SPI,
    cs: OUT,
    busy: IN,
    dc: OUT,
    rst: OUT,
    delay: DELAY,
}

impl<SPI, IN, OUT, DELAY> EPD27b<SPI, IN, OUT, DELAY>
where
    SPI: Write<u8>,
    IN: InputPin,
    OUT: OutputPin,
    DELAY: DelayMs<u8>,
{
    pub fn new(spi: SPI, cs: OUT, dc: OUT, rst: OUT, busy: IN, delay: DELAY) -> Self {
        Self {
            spi,
            cs,
            busy,
            dc,
            rst,
            delay,
        }
    }
    //Hardware reset
    pub fn reset(&mut self) {
        self.rst.set_high().ok();
        self.delay.delay_ms(200);
        self.rst.set_low().ok();
        self.delay.delay_ms(2);
        self.rst.set_high().ok();
        self.delay.delay_ms(200);
    }

    pub fn send_command(&mut self, cmd: u8) {
        self.dc.set_low().ok();
        self.cs.set_low().ok();
        self.spi.write(&[cmd]).ok();
        self.cs.set_high().ok();
    }

    fn send_data(&mut self, data: u8) {
        self.dc.set_high().ok();
        self.cs.set_low().ok();
        self.spi.write(&[data]).ok();
        self.cs.set_high().ok();
    }

    pub fn send_cmd_and_data(&mut self, cmd: u8, data: &[u8]) {
        self.send_command(cmd);
        self.dc.set_high().ok();
        self.cs.set_low().ok();
        self.spi.write(data).ok();
        self.cs.set_high().ok();
    }

    pub fn wait_until_idle(&mut self) {
        sprintln!("e-Paper busy");
        while let Ok(true) = self.busy.is_low() {
            //忙状态输出引脚（低电平表示忙）
            self.delay.delay_ms(100);
        }
        self.delay.delay_ms(20);
        sprintln!("e-Paper busy release");
    }

    fn set_lut(&mut self) {
        self.send_command(0x20);
        LUT_VCOM_DC.iter().for_each(|b| self.send_data(*b));
        self.send_command(0x21);
        LUT_WW.iter().for_each(|b| self.send_data(*b));
        self.send_command(0x22);
        LUT_BW.iter().for_each(|b| self.send_data(*b));
        self.send_command(0x23);
        LUT_BB.iter().for_each(|b| self.send_data(*b));
        self.send_command(0x24);
        LUT_WB.iter().for_each(|b| self.send_data(*b));
    }

    pub fn turn_on_display(&mut self) {
        self.send_command(0x12);
        self.delay.delay_ms(100);
        self.wait_until_idle();
    }

    pub fn init(&mut self) {
        self.reset();
        self.send_command(0x04);
        self.wait_until_idle();

        self.send_command(0x00);
        self.send_data(0xaf);
        self.send_command(0x30); //# PLL_CONTROL
        self.send_data(0x3a); // #3A 100HZ   29 150Hz 39 200HZ    31 171HZ
        self.send_command(0x01); // # POWER_SETTING
        self.send_data(0x03); // # VDS_EN, VDG_EN
        self.send_data(0x00); // # VCOM_HV, VGHL_LV[1], VGHL_LV[0]
        self.send_data(0x2b); // # VDH
        self.send_data(0x2b); // # VDL
        self.send_data(0x09); // # VDHR

        self.send_command(0x06); // # BOOSTER_SOFT_START
        self.send_data(0x07); //
        self.send_data(0x07); //
        self.send_data(0x17); //

        //# Power optimization
        self.send_command(0xF8); //
        self.send_data(0x60); //
        self.send_data(0xA5); //

        //# Power optimization
        self.send_command(0xF8); //
        self.send_data(0x89); //
        self.send_data(0xA5); //

        //# Power optimization
        self.send_command(0xF8); //
        self.send_data(0x90); //
        self.send_data(0x00); //

        //# Power optimization
        self.send_command(0xF8); //
        self.send_data(0x93); //
        self.send_data(0x2A); //

        //# Power optimization
        self.send_command(0xF8); //
        self.send_data(0x73); //
        self.send_data(0x41); //

        self.send_command(0x82); // # VCM_DC_SETTING_REGISTER
        self.send_data(0x12); //
        self.send_command(0x50); // # VCOM_AND_DATA_INTERVAL_SETTING
        self.send_data(0x87); // # define by OTP

        self.set_lut(); //

        self.send_command(0x16); //# PARTIAL_DISPLAY_REFRESH
        self.send_data(0x00);
    }

    pub fn clear(&mut self, color: u8) {
        self.send_command(0x10);
        (0..HEIGHT * WIDE).for_each(|_| {
            self.send_data(color);
        });
        self.send_command(0x11);

        self.send_command(0x13);
        (0..HEIGHT * WIDE).for_each(|_| {
            self.send_data(color);
        });
        self.send_command(0x11);
        self.send_command(0x12);
        self.turn_on_display();
    }

    pub fn clear_red(&mut self) {
        self.send_command(0x10);
        (0..HEIGHT * WIDE).for_each(|_| {
            self.send_data(0xff);
        });
        self.send_command(0x11);
        self.send_command(0x13);
        (0..HEIGHT * WIDE).for_each(|_| {
            self.send_data(0xff);
        });
        self.send_command(0x11);
        self.send_command(0x12);
        self.turn_on_display();
    }

    pub fn clear_black(&mut self) {
        self.send_command(0x10);
        (0..HEIGHT * WIDE).for_each(|_| {
            self.send_data(0x00);
        });
        self.send_command(0x11);
        self.send_command(0x13);
        (0..HEIGHT * WIDE).for_each(|_| {
            self.send_data(0x00);
        });
        self.send_command(0x11);
        self.send_command(0x12);
        self.turn_on_display();
    }

    pub fn display(&mut self, black: &[u8; WIDTH * HEIGHT / 8], red: &[u8; WIDTH * HEIGHT / 8]) {
        self.send_command(0x10);
        (0..HEIGHT * WIDE).for_each(|i| {
            self.send_data(black[i]);
        });
        self.send_command(0x11);

        self.send_command(0x13);
        (0..HEIGHT * WIDE).for_each(|i| {
            self.send_data(red[i]);
        });
        self.send_command(0x11);
        self.send_command(0x12);
        self.turn_on_display();
    }

    pub fn sleep(&mut self) {
        self.send_command(0x50);
        self.send_data(0xf7);
        self.send_command(0x02);
        self.send_command(0x07);
        self.send_data(0xA5);
    }
}
