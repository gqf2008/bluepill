use stm32f1xx_hal::flash::ACR;
use stm32f1xx_hal::prelude::_stm32_hal_time_U32Ext;
use stm32f1xx_hal::rcc::{Clocks, CFGR};

pub trait ClockConfig {
    fn full_clocks(self, acr: &mut ACR) -> Clocks;
    fn clocks(self, acr: &mut ACR) -> Clocks;
}

impl ClockConfig for CFGR {
    #[inline]
    fn full_clocks(self, acr: &mut ACR) -> Clocks {
        self.use_hse(8.mhz())
            .hclk(72.mhz())
            .sysclk(72.mhz())
            .pclk1(24.mhz())
            .pclk2(24.mhz())
            .freeze(acr)
    }

    #[inline]
    fn clocks(self, acr: &mut ACR) -> Clocks {
        self.freeze(acr)
    }
}
