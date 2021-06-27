//!烟雾传感器

use embedded_hal::digital::v2::InputPin;

pub struct MQ2<Pin> {
    aout: Pin,
}

impl<Pin> MQ2<Pin>
where
    Pin: InputPin,
{
    pub fn new(aout: Pin) -> Self {
        Self { aout }
    }

    pub fn wait(&self) -> nb::Result<bool, <Pin as InputPin>::Error> {
        match self.aout.is_low() {
            Ok(true) => Ok(true),
            Ok(false) => Err(nb::Error::WouldBlock),
            Err(err) => Err(nb::Error::Other(err)),
        }
    }
}
