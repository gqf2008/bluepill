// pub mod esp826601s;
pub mod tcp;

pub trait Wifi {
    fn connect(ssid: &str, passwd: &str) -> Self;
    fn auto_reconnect(&mut self);
    fn mac(&mut self);
    fn ip(&mut self);
    fn ping(&mut self, host: &str) -> bool;
}
