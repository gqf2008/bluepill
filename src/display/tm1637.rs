use embedded_hal::{
    digital::v2::{InputPin, OutputPin},
    timer::CountDown,
};
use nb::block;

pub static DIGIT: [u8; 22] = [
    0x3F, /*0*/
    0x06, /*1*/
    0x5B, /*2*/
    0x4F, /*3*/
    0x66, /*4*/
    0x6D, /*5*/
    0x7D, /*6*/
    0x07, /*7*/
    0x7F, /*8*/
    0x6F, /*9*/
    0x77, /*10 A*/
    0x7C, /*11 b*/
    0x58, /*12 c*/
    0x5E, /*13 d*/
    0x79, /*14 E*/
    0x71, /*15 F*/
    0x76, /*16 H*/
    0x38, /*17 L*/
    0x54, /*18 n*/
    0x73, /*19 P*/
    0x3E, /*20 U*/
    0x00, /*21 黑屏*/
];

pub struct TM1637<'chip, DIO, CLK, TIMER> {
    dio: DIO,
    clk: CLK,
    timer: &'chip mut TIMER,
    brightness: u8,
}

impl<'chip, DIO, CLK, TIMER> TM1637<'chip, DIO, CLK, TIMER>
where
    DIO: OutputPin + InputPin,
    CLK: OutputPin,
    TIMER: CountDown,
{
    pub fn new(dio: DIO, clk: CLK, timer: &'chip mut TIMER) -> Self {
        Self {
            dio,
            clk,
            timer,
            brightness: 0x00,
        }
    }

    //I2C时序，
    //起始位 CLK为高电平时，DIO由高变低
    fn start(&mut self) {
        self.clk.set_high().ok();
        self.dio.set_high().ok();
        block!(self.timer.wait()).unwrap();
        self.dio.set_low().ok();
        block!(self.timer.wait()).unwrap();
        self.clk.set_low().ok();
        block!(self.timer.wait()).unwrap();
    }

    //停止位 CLK为高电平时，DIO由低变高
    fn stop(&mut self) {
        self.clk.set_low().ok();
        block!(self.timer.wait()).unwrap();
        self.dio.set_low().ok();
        block!(self.timer.wait()).unwrap();
        self.clk.set_high().ok();
        block!(self.timer.wait()).unwrap();
        self.dio.set_high().ok();
        block!(self.timer.wait()).unwrap();
    }

    fn write_bit(&mut self, bit: bool) {
        self.clk.set_low().ok();
        block!(self.timer.wait()).unwrap();
        if bit {
            self.dio.set_high().ok();
        } else {
            self.dio.set_low().ok();
        }
        block!(self.timer.wait()).unwrap();
        self.clk.set_high().ok();
        block!(self.timer.wait()).unwrap();
    }

    fn write_byte(&mut self, byte: u8) {
        for i in 0..8 {
            self.write_bit((byte >> i) & 0x01 != 0);
        }
        self.clk.set_low().ok();
        block!(self.timer.wait()).unwrap();
        self.dio.set_high().ok();
        block!(self.timer.wait()).unwrap();
        self.clk.set_high().ok();
        block!(self.timer.wait()).unwrap();
        while let Ok(true) = self.dio.is_high() {}
    }

    fn write_cmd(&mut self, cmd: u8) {
        self.start();
        self.write_byte(cmd);
        self.stop();
    }

    fn write_to(&mut self, addr: u8, data: u8) {
        self.start();
        self.write_byte(addr);
        self.write_byte(data);
        self.stop();
    }

    pub fn set_brightness(&mut self, v: u8) {
        self.brightness = v;
    }

    pub fn write(&mut self, data: &[u8; 4], colon: Option<bool>) {
        self.write_cmd(0x44);
        let colon = colon.is_some();
        for (i, b) in data.iter().enumerate() {
            if colon && i == 1 {
                self.write_to(0xc0 + i as u8, *b | 0x80);
            } else {
                self.write_to(0xc0 + i as u8, *b);
            }
        }
        self.write_cmd(0x88 | self.brightness); //0x07表示最大亮度
    }
}
