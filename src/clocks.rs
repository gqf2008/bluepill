use stm32f1xx_hal::flash::ACR;
use stm32f1xx_hal::prelude::_stm32_hal_time_U32Ext;
use stm32f1xx_hal::rcc::{Clocks, CFGR};

#[inline]
pub fn full_clocks(cfgr: CFGR, acr: &mut ACR) -> Clocks {
    cfgr.use_hse(8.mhz())
        .hclk(72.mhz())
        .sysclk(72.mhz())
        .pclk1(24.mhz())
        .pclk2(24.mhz())
        .freeze(acr)
}

#[inline]
pub fn clocks(cfgr: CFGR, acr: &mut ACR) -> Clocks {
    cfgr.freeze(acr)
}
