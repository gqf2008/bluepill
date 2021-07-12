use embedded_hal::blocking::rng;
use stm32f1xx_hal::adc::SampleTime;

#[derive(Default)]
pub struct Rng;

impl rng::Read for Rng {
    type Error = ();
    fn read(&mut self, buffer: &mut [u8]) -> Result<(), Self::Error> {
        cortex_m::interrupt::free(|cs| {
            if let Some(adc) = crate::ADC.borrow(cs).borrow_mut().as_mut() {
                let prev_cfg = adc.save_cfg();
                adc.set_sample_time(SampleTime::T_1);
                for buf in buffer {
                    let mut random: u8 = 0;
                    for _i in 0..8 {
                        let mut num = 0;
                        for _i in 0..5 {
                            num += adc.read_vref() & 0x01;
                        }
                        random = (random << 1) + (num & 0x01) as u8;
                    }
                    *buf = random;
                }
                adc.restore_cfg(prev_cfg);
            }
        });
        Ok(())
    }
}
