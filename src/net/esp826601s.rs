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
//! AT+CIPCLOSE=<link	ID> 关闭TCP连接

//! TCP服务器
//! AT+CIPSERVER=1,3333 监听3333端口
//! AT+CIPSERVER=0,3333 关闭监听3333端口
//! AT测试

use crate::io::{Error, Result, TimeoutReader};
use heapless::String;
use stm32f1xx_hal::time::Hertz;

const OK: &str = "OK";
const ERROR: &str = "ERROR";
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

    pub fn connect(
        &mut self,
        ssid: &str,
        password: &str,
        autoconnect: bool,
    ) -> Result<String<256>> {
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

    pub fn ifconfig(&mut self) -> Result<String<256>> {
        self.request("AT+CIFSR\r\n", 5000)
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
}
