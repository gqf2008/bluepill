//! ESP8266-01w模块

use anyhow::anyhow;
use anyhow::Result;
use core::fmt::Write;
use cortex_m::singleton;
use cortex_m_semihosting::hprintln;
use nb::block;
use stm32f1xx_hal::{
    dma::{dma1::C2, dma1::C3, RxDma, TxDma},
    gpio::gpiob::*,
    gpio::*,
    pac::interrupt,
    pac::Interrupt,
    pac::USART1,
    pac::USART3,
    prelude::*,
    serial::{Config, Rx, Serial, Tx},
};
//AT+CWJAP_DEF="ssid","paasword" 连接WIFI
//AT+CIPSTAMAC_CUR? 查MAC地址
//AT+CWAUTOCONN=1 上电自动连接WIFI
//AT+CIPSTA_CUR? 查IP地址信息

//TCP客户端
//AT+CIPSTATUS 查询连接状态
//AT+CIFSR 查询设备IP地址
//AT+CIPDOMAIN="www.baidu.com" 域名解析
//AT+CIPSTART="TCP","iot.espressif.cn",8000 建立TCP连接
//AT+CIPSTART="TCP","192.168.101.110",1000 建立TCP连接
//AT+CIPSSLSIZE=4096 设置TCP缓冲区
//AT+CIPSENDBUF=16 发送16字节数据到TCP缓冲区，满16自己后发送
//AT+CIPBUFSTATUS 查询 TCP 发包缓存的状态
//AT+CIPCLOSE=<link	ID> 关闭TCP连接

//TCP服务器
//AT+CIPSERVER=1,3333 监听3333端口
//AT+CIPSERVER=0,3333 关闭监听3333端口
//AT测试
pub fn ping() -> Result<()> {
    self::write(b"AT\r\n")
    // let usart3 = unsafe { crate::peripherals::USART3.as_mut_ptr() };
    // if !usart3.is_null() {
    //     let (tx, _rx) = unsafe { &mut *usart3 };
    //     if let Err(err) = tx.write_str("AT\r\n") {
    //         return Err(anyhow!(err));
    //     }
    // }
    // Ok(())
}
//获取版本信息
pub fn info() -> Result<()> {
    self::write(b"AT+GMR\r\n")
    // let usart3 = unsafe { crate::peripherals::USART3.as_mut_ptr() };
    // if !usart3.is_null() {
    //     let (tx, _rx) = unsafe { &mut *usart3 };
    //     if let Err(err) = tx.write_str("AT+GMR\r\n") {
    //         return Err(anyhow!(err));
    //     }
    // }
    // Ok(())
}
//重置WIFI设备
pub fn reset() -> Result<()> {
    self::write(b"AT+RST\r\n")
    // let usart3 = unsafe { crate::peripherals::USART3.as_mut_ptr() };
    // if !usart3.is_null() {
    //     let (tx, _rx) = unsafe { &mut *usart3 };
    //     if let Err(err) = tx.write_str("AT+RST\r\n") {
    //         return Err(anyhow!(err));
    //     }
    // }
    // Ok(())
}

//恢复出厂设置
pub fn restore() -> Result<()> {
    self::write(b"AT+RESTORE\r\n")
}
//连接WIFI
pub fn connect(ssid: &str, passwd: &str) -> Result<()> {
    todo!()
}

pub fn read(b: &mut [u8]) -> Result<()> {
    todo!()
}

pub fn write(b: &[u8]) -> Result<()> {
    // let usart3 = unsafe { crate::peripherals::USART3.as_mut_ptr() };
    // if usart3.is_null() {
    //     return Err(anyhow!("USART3 not init"));
    // }
    // let (tx, _rx) = unsafe { &mut *usart3 };
    // b.iter().try_for_each(|w| block!(tx.write(*w))).unwrap();
    Ok(())
}

// pub(crate) struct Wifi {
//     serial: Serial<USART3, (PB10<Alternate<PushPull>>, PB11<Input<Floating>>)>,
// }

// impl Wifi {
//     pub(crate) fn new(
//         serial: Serial<USART3, (PB10<Alternate<PushPull>>, PB11<Input<Floating>>)>,
//     ) -> Self {
//         //let (tx, rx) = serial.split();
//         Self { serial }
//     }
//     pub(crate) fn ping(&mut self) -> core::fmt::Result {
//         self.write(b"AT")
//     }
//     pub(crate) fn connect(&self, ssid: &str, user: &str, passwd: &str) -> Result<()> {
//         todo!()
//     }

//     pub(crate) fn read(&self, b: &mut [u8]) -> Result<()> {
// let mut tx = tx.with_dma(dma.2);
// let mut rx = rx.with_dma(dma.3);
// let mut buf = singleton!(: [u8; 128] = [0; 128]).unwrap();
// let (nbuf, nrx) = rx.read(buf).wait();
// rx = nrx;
// buf = nbuf;
// hprintln!("{:?}", &buf[..]).unwrap();

// let (_, ntx) = tx.write(b"AT+GMR\r\n").wait();
// tx = ntx;
//         todo!()
//     }

//     pub(crate) fn write(&mut self, b: &[u8]) -> core::fmt::Result {
//         b.iter()
//             .try_for_each(|b| block!(self.serial.write(*b)))
//             .map_err(|_| core::fmt::Error)
//     }
// }
