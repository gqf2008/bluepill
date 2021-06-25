use stm32f1xx_hal::flash::ACR;
use stm32f1xx_hal::prelude::_stm32_hal_time_U32Ext;
use stm32f1xx_hal::rcc::{Clocks, CFGR};

#[inline]
pub fn init_full_clocks(cfgr: CFGR, acr: &mut ACR) -> Clocks {
    cfgr.use_hse(8.mhz())
        .sysclk(48.mhz())
        .pclk1(24.mhz())
        .pclk2(24.mhz())
        .freeze(acr)
}

#[inline]
pub fn init_clocks(cfgr: CFGR, acr: &mut ACR) -> Clocks {
    cfgr.freeze(acr)
}
