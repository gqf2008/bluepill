pub mod esp826601s;
pub mod tcp;

use crate::io::Result;
use heapless::String;

pub enum Status {
    Cnnected,
    Disconnect,
}

pub struct IfInfo {
    pub inet4: String<15>,
    pub netmask: String<15>,
    pub gateway: String<15>,
    pub ether: String<17>,
}

pub trait TcpStream: Sized {
    fn connect(addr: (&str, u16)) -> Result<Self>;
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
    fn write(&mut self, buf: &[u8]) -> Result<usize>;
    fn set_write_buffer(&mut self, size: usize) -> Result<()>;
    fn set_read_buffer(&mut self, size: usize) -> Result<()>;
}

pub trait Net {
    fn connect() -> Self;
    fn disconnect(&mut self) -> Result<()>;
    fn hello(&mut self) -> Result<()>;
    fn state(&mut self) -> Result<Status>;
    fn reset(&mut self) -> Result<()>;
    fn recover(&mut self) -> Result<()>;
    fn ifinfo(&mut self) -> Result<IfInfo>;
    fn mac(&mut self) -> Result<String<17>>;
    fn ip(&mut self) -> Result<String<15>>;
    fn resolve(&mut self) -> Result<String<15>>;
    fn ping(&mut self, host: &str) -> Result<()>;
    fn open_tcp<T: TcpStream>(&mut self, addr: (&str, u16)) -> Result<T>;
    fn close_tcp<T: TcpStream>(&mut self, addr: (&str, u16)) -> Result<T>;
}
