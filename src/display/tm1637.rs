use embedded_hal::{
    digital::v2::{InputPin, OutputPin},
    timer::CountDown,
};
use heapless::FnvIndexMap;
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
    digit: FnvIndexMap<char, u8, 24>, //容量必须是2的幂
}

impl<'chip, DIO, CLK, TIMER> TM1637<'chip, DIO, CLK, TIMER>
where
    DIO: OutputPin + InputPin,
    CLK: OutputPin,
    TIMER: CountDown,
{
    pub fn new(dio: DIO, clk: CLK, timer: &'chip mut TIMER) -> Self {
        let mut digit = FnvIndexMap::<char, u8, 24>::new();
        digit.insert('0', 0x3F).ok();
        digit.insert('1', 0x06).ok();
        digit.insert('2', 0x5B).ok();
        digit.insert('3', 0x4F).ok();
        digit.insert('4', 0x66).ok();
        digit.insert('5', 0x6D).ok();
        digit.insert('6', 0x7D).ok();
        digit.insert('7', 0x07).ok();
        digit.insert('8', 0x7F).ok();
        digit.insert('9', 0x6F).ok();
        digit.insert('A', 0x77).ok();
        digit.insert('b', 0x7C).ok();
        digit.insert('c', 0x58).ok();
        digit.insert('d', 0x5E).ok();
        digit.insert('E', 0x79).ok();
        digit.insert('F', 0x71).ok();
        digit.insert('H', 0x76).ok();
        digit.insert('L', 0x38).ok();
        digit.insert('n', 0x54).ok();
        digit.insert('P', 0x73).ok();
        digit.insert('U', 0x3E).ok();
        Self {
            dio,
            clk,
            timer,
            brightness: 0x00,
            digit,
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

    pub fn write(&mut self, data: &[char; 4], colon: Option<bool>) {
        self.write_cmd(0x44); //40 地址自加模式     44 固定地址模式,TM_WriteByte(0xc0);   //首地址
        let colon = colon.is_some();
        for (i, b) in data.iter().enumerate() {
            if colon && i == 1 {
                if let Some(v) = self.digit.get(b) {
                    let v = *v;
                    self.write_to(0xc0 + i as u8, v | 0x80);
                } else {
                    self.write_to(0xc0 + i as u8, 0x00 | 0x80);
                }
            } else {
                if let Some(v) = self.digit.get(b) {
                    let v = *v;
                    self.write_to(0xc0 + i as u8, v);
                } else {
                    self.write_to(0xc0 + i as u8, 0x00);
                }
            }
        }
        self.write_cmd(0x88 | self.brightness); //0x07表示最大亮度
    }
}
