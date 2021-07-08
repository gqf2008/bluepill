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

use crate::hal::time::Hertz;
use crate::io::{Error, Result, TimeoutReader};
use alloc::format;
use alloc::string::String;

const OK: &str = "OK";
const ERROR: &str = "ERROR";
const BUSY: &str = "busy";

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
    //打个招呼
    pub fn hello(&mut self) -> Result<String> {
        self.request(b"AT\r\n", 5000)
    }
    //设备信息
    pub fn device_info(&mut self) -> Result<String> {
        self.request(b"AT+GMR\r\n", 5000)
    }

    //重置
    pub fn reset(&mut self) -> Result<String> {
        self.request(b"AT+RST\r\n", 5000)
    }

    //恢复出厂设置
    pub fn restore(&mut self) -> Result<String> {
        self.request(b"AT+RESTORE\r\n", 5000)
    }

    //连接AP
    pub fn dial(&mut self, ssid: &str, password: &str, autoconnect: bool) -> Result<String> {
        let mut cmd = String::from("AT+CWJAP_DEF=\"");
        cmd.push_str(ssid);
        cmd.push_str("\",\"");
        cmd.push_str(password);
        cmd.push_str("\"\r\n");
        let reply = self.request(cmd.as_bytes(), 15000)?;
        if autoconnect {
            self.request(b"AT+CWAUTOCONN=1\r\n", 5000)?;
        } else {
            self.request(b"AT+CWAUTOCONN=0\r\n", 5000)?;
        }

        Ok(reply)
    }

    //断开与AP的连接
    pub fn hangup(&mut self) -> Result<String> {
        self.request(b"AT+CWQAP\r\n", 5000)
    }

    #[inline]
    fn request(&mut self, cmd: &[u8], timeout: u32) -> Result<String> {
        self.write_exact(cmd).ok();
        let mut buf = String::new();
        let mut reader = TimeoutReader(&mut self.port, &mut self.timer);
        loop {
            match reader.read_line(timeout)? {
                line if line.starts_with(OK) => {
                    buf.push_str(line.as_str());
                    return Ok(buf);
                }
                line if line.starts_with(ERROR) => {
                    buf.push_str(line.as_str());
                    return Err(Error::Other(buf));
                }
                line if line.starts_with(BUSY) => {
                    return self.request(cmd, timeout);
                    // buf.push_str(line.as_str());
                    // return Err(Error::DeviceBusy);
                }
                line => {
                    crate::sprint!(line.as_str());
                    buf.push_str(line.as_str());
                }
            }
        }
    }

    /////////////////////////////////////////////////////////////

    pub fn ifconfig(&mut self) -> Result<String> {
        let mut reply1 = self.request(b"AT+CIFSR\r\n", 5000)?;
        let reply2 = self.request(b"AT+CIPSTA_CUR?\r\n", 5000)?;
        reply1.push_str(reply2.as_str());
        Ok(reply1)
    }

    pub fn ping(&mut self, domain: &str) -> Result<String> {
        //AT+PING="www.shouqianba.com"
        let mut cmd = String::from("AT+PING=\"");
        cmd.push_str(domain);
        cmd.push_str("\"\r\n");
        self.request(cmd.as_bytes(), 5000)
    }

    pub fn reslove(&mut self, domain: &str) -> Result<String> {
        let mut cmd = String::from("AT+CIPDOMAIN=\"");
        cmd.push_str(domain);
        cmd.push_str("\"\r\n");
        self.request(cmd.as_bytes(), 5000)
    }

    pub fn net_state(&mut self) -> Result<String> {
        self.request(b"AT+CIPSTATUS\r\n", 5000)
    }

    pub fn connect(&mut self, addr: (&str, &str)) -> Result<String> {
        //AT+CIPSTART="TCP","iot.espressif.cn",8000 建立TCP连接
        let mut cmd = String::from("AT+CIPSTART=\"TCP\",\"");
        cmd.push_str(addr.0);
        cmd.push_str("\",");
        cmd.push_str(addr.1);
        cmd.push_str("\r\n");
        self.request(cmd.as_bytes(), 15000)
    }

    pub fn disconnect(&mut self) -> Result<String> {
        self.request(b"AT+CIPCLOSE\r\n", 5000)
    }

    pub fn write_exact(&mut self, buf: &[u8]) -> Result<usize> {
        if let Err(_err) = buf.iter().try_for_each(|b| nb::block!(self.port.write(*b))) {
            return Err(Error::WriteError);
        }
        Ok(buf.len())
    }

    // pub fn read_exact<const N: usize>(&mut self, buf: &mut [u8], timeout: u32) -> Result<()> {
    //     let mut reader = TimeoutReader(&mut self.port, &mut self.timer);
    //     reader.read_exact(buf, timeout)?;
    //     Ok(())
    // }

    pub fn send_data(&mut self, buf: &[u8]) -> Result<usize> {
        let mut cmd = String::from("AT+CIPSEND=");
        cmd.push_str(format!("{}", buf.len()).as_str());
        cmd.push_str("\r\n");
        self.request(cmd.as_bytes(), 5000)?;
        {
            let mut reader = TimeoutReader(&mut self.port, &mut self.timer);
            let mut reply = [0; 1];
            //read '>'
            reader.read_exact(&mut reply, 5000)?;
        }
        // self.request(cmd.as_bytes(), 5000)?;
        self.write_exact(buf)?;
        let mut reader = TimeoutReader(&mut self.port, &mut self.timer);
        loop {
            match reader.read_line(5000)? {
                line if line.starts_with("SEND OK") || line.starts_with("OK") => {
                    return Ok(buf.len())
                }
                line if line.starts_with("SEND FAIL") || line.starts_with("ERROR") => {
                    return Err(Error::WriteError)
                }
                _ => {}
            }
        }
    }
    pub fn read_data(&mut self, len: usize) -> Result<String> {
        //AT+CIPRECVDATA=<len>
        let mut cmd = String::from("AT+CIPSEND=");
        cmd.push_str(format!("{}", len).as_str());
        cmd.push_str("\r\n");
        self.request(cmd.as_bytes(), 5000)
    }
}
