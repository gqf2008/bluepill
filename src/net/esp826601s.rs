//! ESP8266-01S模块
//! AT+CWJAP_DEF="ssid","paasword" 连接WIFI
//! AT+CIPSTAMAC_CUR? 查MAC地址
//! AT+CWAUTOCONN=1 上电自动连接WIFI
//! AT+CIPSTA_CUR? 查IP地址信息

//! TCP客户端
//! AT+CIPSTATUS 查询连接状态
//! AT+CIFSR 查询设备IP地址
//! AT+CIPDOMAIN="www.baidu.com" 域名解析
//! AT+CIPSTART="TCP","iot.espressif.cn",8000 建立TCP连接
//! AT+CIPSTART="TCP","192.168.101.110",1000 建立TCP连接
//! AT+CIPSSLSIZE=4096 设置TCP缓冲区
//! AT+CIPSENDBUF=16 发送16字节数据到TCP缓冲区，满16自己后发送
//! AT+CIPBUFSTATUS 查询 TCP 发包缓存的状态
//! AT+CIPCLOSE=<link ID> 关闭TCP连接

//! TCP服务器
//! AT+CIPSERVER=1,3333 监听3333端口
//! AT+CIPSERVER=0,3333 关闭监听3333端口
//! AT测试

use crate::hal::pac::interrupt;
use crate::hal::pac::{USART1, USART2, USART3};
use crate::hal::serial::Pins;
use crate::hal::serial::Serial;
use crate::hal::serial::{Rx, Tx};
use crate::hal::time::Hertz;
use crate::io::{Error, Result, TimeoutReader};
use core::cell::RefCell;
use core::convert::Infallible;
use cortex_m::interrupt::Mutex;
use embedded_hal::serial::{Read, Write};
use heapless::spsc::Queue;
use heapless::String;

const OK: &str = "OK";
const ERROR: &str = "ERROR";

// pub struct RW<W> {
//     tx: W,
// }

// macro_rules! esp8266 {
//     ($(
//         $(#[$meta:meta])*
//         $USARTX:ident: ($RXX:ident, $BUFX:ident),
//     )+) => {
//         $(
//             $(#[$meta])*

//         impl RW<Tx<$USARTX>> {
//             pub fn new<PINS>(serial: Serial<$USARTX, PINS>) -> Self
//             where
//                 PINS: Pins<$USARTX>,
//             {
//                 let (tx, rx) = serial.split();
//                 let q: Queue<u8, 4096> = Queue::new();
//                 cortex_m::interrupt::free(|cs| {
//                     $RXX.borrow(cs).replace(Some(rx)).unwrap();
//                     unsafe { $BUFX.replace(q) };
//                 });
//                 crate::enable_interrupt(crate::hal::pac::Interrupt::$USARTX);
//                 Self { tx }
//             }
//         }

//         impl Write<u8> for RW<Tx<$USARTX>> {
//             type Error = Infallible;

//             fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
//                 self.tx.write(word)
//             }

//             fn flush(&mut self) -> nb::Result<(), Self::Error> {
//                 self.tx.flush()
//             }
//         }

//         impl Read<u8> for RW<Tx<$USARTX>> {
//             type Error = crate::io::Error;
//             fn read(&mut self) -> nb::Result<u8, Self::Error> {
//                 cortex_m::interrupt::free(|_| match unsafe { $BUFX.as_mut() } {
//                     Some(q) => match q.dequeue() {
//                         Some(w) => Ok(w),
//                         None => Err(nb::Error::WouldBlock),
//                     },
//                     None => Err(nb::Error::Other(crate::io::Error::NoIoDevice)),
//                 })
//             }
//         }
//         )+
//     }
// }

// esp8266! {
//     USART1:(RX1, TX1_BUFFER),
//     USART2:(RX2, TX2_BUFFER),
//     USART3:(RX3, TX3_BUFFER),
// }

// static RX1: Mutex<RefCell<Option<Rx<USART1>>>> = Mutex::new(RefCell::new(None));
// static mut TX1_BUFFER: Option<Queue<u8, 4096>> = None;
// static RX2: Mutex<RefCell<Option<Rx<USART2>>>> = Mutex::new(RefCell::new(None));
// static mut TX2_BUFFER: Option<Queue<u8, 4096>> = None;
// static RX3: Mutex<RefCell<Option<Rx<USART3>>>> = Mutex::new(RefCell::new(None));
// static mut TX3_BUFFER: Option<Queue<u8, 4096>> = None;

// #[interrupt]
// unsafe fn USART1() {
//     static mut RX: Option<Rx<USART1>> = None;
//     let rx = RX.get_or_insert_with(|| {
//         cortex_m::interrupt::free(|cs| RX1.borrow(cs).replace(None).unwrap())
//     });
//     if let Ok(w) = nb::block!(rx.read()) {
//         cortex_m::interrupt::free(|_| {
//             if let Some(buf) = TX1_BUFFER.as_mut() {
//                 buf.enqueue(w).ok();
//             }
//         })
//     }
// }

// #[interrupt]
// unsafe fn USART2() {
//     static mut RX: Option<Rx<USART2>> = None;
//     let rx = RX.get_or_insert_with(|| {
//         cortex_m::interrupt::free(|cs| RX2.borrow(cs).replace(None).unwrap())
//     });
//     if let Ok(w) = nb::block!(rx.read()) {
//         cortex_m::interrupt::free(|_| {
//             if let Some(buf) = TX2_BUFFER.as_mut() {
//                 buf.enqueue(w).ok();
//             }
//         })
//     }
// }

// #[interrupt]
// unsafe fn USART3() {
//     static mut RX: Option<Rx<USART3>> = None;
//     let rx = RX.get_or_insert_with(|| {
//         cortex_m::interrupt::free(|cs| RX3.borrow(cs).replace(None).unwrap())
//     });
//     if let Ok(w) = nb::block!(rx.read()) {
//         cortex_m::interrupt::free(|_| {
//             if let Some(buf) = TX3_BUFFER.as_mut() {
//                 buf.enqueue(w).ok();
//             }
//         })
//     }
// }

pub struct Esp8266<T, TIM> {
    port: T,
    timer: TIM,
}

impl<T, TIM> Esp8266<T, TIM>
where
    T: embedded_hal::serial::Read<u8> + embedded_hal::serial::Write<u8>,
    TIM: embedded_hal::timer::CountDown<Time = Hertz>,
{
    pub fn new(port: T, timer: TIM) -> Self {
        Self { port, timer }
    }

    pub fn hello(&mut self) -> Result<String<256>> {
        self.request("AT\r\n", 5000)
    }

    pub fn device_info(&mut self) -> Result<String<256>> {
        self.request("AT+GMR\r\n", 5000)
    }

    //重置
    pub fn reset(&mut self) -> Result<String<256>> {
        self.request("AT+RST\r\n", 5000)
    }

    //恢复出厂设置
    pub fn restore(&mut self) -> Result<String<256>> {
        self.request("AT+RESTORE\r\n", 5000)
    }

    //连接AP
    pub fn dial(&mut self, ssid: &str, password: &str, autoconnect: bool) -> Result<String<256>> {
        let mut cmd: String<128> = String::from("AT+CWJAP_DEF=\"");
        cmd.push_str(ssid).ok();
        cmd.push_str("\",\"").ok();
        cmd.push_str(password).ok();
        cmd.push_str("\"\r\n").ok();
        let reply = self.request(cmd.as_str(), 15000)?;
        if autoconnect {
            self.request("AT+CWAUTOCONN=1\r\n", 5000)?;
        } else {
            self.request("AT+CWAUTOCONN=0\r\n", 5000)?;
        }

        Ok(reply)
    }

    //断开与AP的连接
    pub fn hangup(&mut self) -> Result<String<256>> {
        self.request("AT+CWQAP\r\n", 5000)
    }

    #[inline]
    fn request(&mut self, cmd: &str, timeout: u32) -> Result<String<256>> {
        self.write_exact(cmd.as_bytes()).ok();
        let mut buf: String<256> = String::new();
        let mut reader = TimeoutReader(&mut self.port, &mut self.timer);
        loop {
            match reader.read_line::<256>(timeout)? {
                line if line.starts_with(OK) => {
                    buf.push_str(line.as_str()).ok();
                    return Ok(buf);
                }
                line if line.starts_with(ERROR) => {
                    buf.push_str(line.as_str()).ok();
                    return Err(Error::Other(buf));
                }
                line => {
                    buf.push_str(line.as_str()).ok();
                }
            }
        }
    }

    /////////////////////////////////////////////////////////////

    pub fn ifconfig(&mut self) -> Result<String<256>> {
        self.request("AT+CIFSR\r\n", 5000)
    }

    pub fn ping(&mut self, domain: &str) -> Result<String<256>> {
        //AT+PING="www.shouqianba.com"
        let mut cmd: String<128> = String::from("AT+PING=\"");
        cmd.push_str(domain).ok();
        cmd.push_str("\"\r\n").ok();
        self.request(cmd.as_str(), 5000)
    }

    pub fn reslove(&mut self, domain: &str) -> Result<String<256>> {
        let mut cmd: String<128> = String::from("AT+CIPDOMAIN=\"");
        cmd.push_str(domain).ok();
        cmd.push_str("\"\r\n").ok();
        self.request(cmd.as_str(), 5000)
    }

    pub fn net_state(&mut self) -> Result<String<256>> {
        self.request("AT+CIPSTATUS\r\n", 5000)
    }

    pub fn connect(&mut self, addr: (&str, &str)) -> Result<String<256>> {
        //AT+CIPSTART="TCP","iot.espressif.cn",8000 建立TCP连接
        let mut cmd: String<128> = String::from("AT+CIPSTART=\"TCP\",\"");
        cmd.push_str(addr.0).ok();
        cmd.push_str("\",").ok();
        cmd.push_str(addr.1).ok();
        cmd.push_str("\r\n").ok();
        self.request(cmd.as_str(), 15000)
    }

    pub fn disconnect(&mut self) -> Result<String<256>> {
        self.request("AT+CIPCLOSE", 5000)
    }

    pub fn write_exact(&mut self, buf: &[u8]) -> Result<usize> {
        if let Err(_err) = buf.iter().try_for_each(|b| nb::block!(self.port.write(*b))) {
            return Err(Error::WriteError);
        }
        Ok(buf.len())
    }

    pub fn read_exact<const N: usize>(&mut self, buf: &mut [u8], timeout: u32) -> Result<()> {
        let mut reader = TimeoutReader(&mut self.port, &mut self.timer);
        reader.read_exact(buf, timeout)?;
        Ok(())
    }

    pub fn send_data(&mut self, buf: &[u8]) -> Result<usize> {
        let mut cmd: String<128> = String::from("AT+CIPSEND=");
        cmd.push_str("111").ok();
        cmd.push_str("\r\n").ok();
        self.request(cmd.as_str(), 5000)?;
        {
            let mut reader = TimeoutReader(&mut self.port, &mut self.timer);
            let mut reply = [0; 1];
            //read '>'
            reader.read_exact(&mut reply, 5000)?;
        }
        self.write_exact(buf)?;
        let mut reader = TimeoutReader(&mut self.port, &mut self.timer);
        loop {
            match reader.read_line::<256>(5000)? {
                line if line.starts_with("SEND  OK") => return Ok(buf.len()),
                line if line.starts_with("SEND	FAIL") || line.starts_with("ERROR") => {
                    return Err(Error::WriteError)
                }
                _ => {}
            }
        }
    }
    pub fn read_data(&mut self, buf: &mut [u8]) -> Result<usize> {
        //AT+CIPRECVDATA=<len>
        todo!()
    }
}
