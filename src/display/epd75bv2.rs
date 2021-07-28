use crate::sprintln;
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::spi::Write;
use embedded_hal::digital::v2::*;

const WIDTH: usize = 800;
const HEIGHT: usize = 480;
const WIDE: usize = WIDTH / 8;

pub struct EPD75b<SPI, CS, DC, RST, BUSY, DELAY> {
    spi: SPI,
    cs: CS,
    busy: BUSY,
    dc: DC,
    rst: RST,
    delay: DELAY,
}

impl<SPI, CS, DC, RST, BUSY, DELAY> EPD75b<SPI, CS, DC, RST, BUSY, DELAY>
where
    SPI: Write<u8>,
    BUSY: InputPin,
    DC: OutputPin,
    CS: OutputPin,
    RST: OutputPin,
    DELAY: DelayMs<u8>,
{
    pub fn new(spi: SPI, cs: CS, dc: DC, rst: RST, busy: BUSY, delay: DELAY) -> Self {
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
            self.delay.delay_ms(20);
        }
        self.delay.delay_ms(20);
        sprintln!("e-Paper busy release");
    }

    pub fn turn_on_display(&mut self) {
        self.send_command(0x12);
        self.delay.delay_ms(100);
        self.wait_until_idle();
    }

    pub fn init(&mut self) {
        self.reset();
        self.send_command(0x06);
        self.send_data(0x17);
        self.send_data(0x17);
        self.send_data(0x28);
        self.send_data(0x17);

        self.send_command(0x04);
        self.delay.delay_ms(100);
        self.wait_until_idle();

        self.send_command(0x00);
        self.send_data(0x0F);

        self.send_command(0x61);
        self.send_data(0x03);
        self.send_data(0x20);
        self.send_data(0x01);
        self.send_data(0xE0);

        self.send_command(0x15);
        self.send_data(0x00);

        self.send_command(0x50);
        self.send_data(0x11);
        self.send_data(0x07);

        self.send_command(0x60);
        self.send_data(0x22);

        self.send_command(0x65);
        self.send_data(0x00);
        self.send_data(0x00);
        self.send_data(0x00);
        self.send_data(0x00);
    }

    pub fn clear(&mut self) {
        self.send_command(0x10);
        (0..HEIGHT).for_each(|_| {
            (0..WIDE).for_each(|_| {
                self.send_data(0xff);
            });
        });

        self.send_command(0x13);
        (0..HEIGHT).for_each(|_| {
            (0..WIDE).for_each(|_| {
                self.send_data(0x00);
            });
        });
        self.turn_on_display();
    }

    pub fn clear_red(&mut self) {
        self.send_command(0x10);
        (0..HEIGHT).for_each(|_| {
            (0..WIDE).for_each(|_| {
                self.send_data(0xff);
            });
        });
        self.send_command(0x13);
        (0..HEIGHT).for_each(|_| {
            (0..WIDE).for_each(|_| {
                self.send_data(0xff);
            });
        });
        self.turn_on_display();
    }

    pub fn clear_black(&mut self) {
        self.send_command(0x10);
        (0..HEIGHT).for_each(|_| {
            (0..WIDE).for_each(|_| {
                self.send_data(0x00);
            });
        });
        self.send_command(0x13);
        (0..HEIGHT).for_each(|_| {
            (0..WIDE).for_each(|_| {
                self.send_data(0x00);
            });
        });
        self.turn_on_display();
    }

    pub fn display(&mut self, black: &[u8; WIDTH * HEIGHT / 8], red: &[u8; WIDTH * HEIGHT / 8]) {
        self.send_command(0x10);
        for j in 0..HEIGHT {
            for i in 0..WIDE {
                self.send_data(black[i + j * WIDE]);
            }
        }
        self.send_command(0x13);
        for j in 0..HEIGHT {
            for i in 0..WIDE {
                self.send_data(red[i + j * WIDE]);
            }
        }
        self.turn_on_display();
    }

    pub fn sleep(&mut self) {
        self.send_command(0x02);
        self.wait_until_idle();
        self.send_command(0x07);
        self.send_data(0xa5);
    }
}
