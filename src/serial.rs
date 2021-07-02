use crate::hal::afio::MAPR;
use crate::hal::gpio::gpioa;
use crate::hal::gpio::gpiob;
use crate::hal::gpio::gpioc;
use crate::hal::gpio::gpiod;
use crate::hal::gpio::{Alternate, Floating, Input, PushPull};
use crate::hal::pac::{USART1, USART2, USART3};
use crate::hal::serial::{Config, StopBits};
use crate::hal::time::U32Ext;
use stm32f1xx_hal::rcc::{Clocks, APB1, APB2};

pub struct Serial<'a, USART, PINx, PINy, BUS, CR> {
    usart: USART,
    pins: Option<(PINx, PINy)>,
    clocks: Option<Clocks>,
    afio_mapr: Option<&'a mut MAPR>,
    apb: Option<&'a mut BUS>,
    cr: Option<&'a mut CR>,
    config: Config,
}

impl<'a, USART, PINx, PINy, BUS, CR> Serial<'a, USART, PINx, PINy, BUS, CR> {
    //绑定串口设备
    pub fn with_usart(usart: USART) -> Self {
        Self {
            usart,
            pins: None,
            clocks: None,
            apb: None,
            afio_mapr: None,
            cr: None,
            config: Config::default().baudrate(115200.bps()),
        }
    }
    //开启AFIO时钟，复用重映射和调试I/O配置寄存器(AFIO_MAPR)
    pub fn afio_mapr(mut self, mapr: &'a mut MAPR) -> Self {
        self.afio_mapr = Some(mapr);
        self
    }
    //映射到GPIO引脚
    pub fn pins(mut self, tx: PINx, rx: PINy) -> Self {
        self.pins = Some((tx, rx));
        self
    }
    //配置时钟
    pub fn clocks(mut self, clocks: Clocks) -> Self {
        self.clocks = Some(clocks);
        self
    }
    //配置内核总线
    pub fn bus(mut self, apb: &'a mut BUS) -> Self {
        self.apb = Some(apb);
        self
    }
    //配置GPIO控制寄存器
    pub fn cr(mut self, cr: &'a mut CR) -> Self {
        self.cr = Some(cr);
        self
    }
    //配置串口波特率
    pub fn baudrate(mut self, baud: u32) -> Self {
        self.config = self.config.baudrate(baud.bps());
        self
    }
    //奇偶校验微None
    pub fn parity_none(mut self) -> Self {
        self.config = self.config.parity_none();
        self
    }
    //奇偶校验微even
    pub fn parity_even(mut self) -> Self {
        self.config = self.config.parity_even();
        self
    }
    //奇偶校验微odd
    pub fn parity_odd(mut self) -> Self {
        self.config = self.config.parity_odd();
        self
    }
    //配置停止位
    pub fn stopbits(mut self, stopbits: StopBits) -> Self {
        self.config = self.config.stopbits(stopbits);
        self
    }
}

macro_rules! serial {
    ($(
        $(#[$meta:meta])*
        $USARTX:ident: (
            $usartX:ident,
            $APBX:ident,
            $gpioX:ident,
            $TX:ident,
            $RX:ident,
            $CR:ident,
        ),
    )+) => {
        $(
            $(#[$meta])*
            impl<'a> Serial<'a, $USARTX, $gpioX::$TX<Input<Floating>>, $gpioX::$RX<Input<Floating>>, $APBX, $gpioX::$CR> {
                pub fn build(
                    self,
                ) -> crate::hal::serial::Serial< $USARTX, ($gpioX::$TX<Alternate<PushPull>>, $gpioX::$RX<Input<Floating>>)> {
                    let (tx, rx) = self.pins.unwrap();
                    let tx = tx.into_alternate_push_pull(self.cr.unwrap());
                    crate::hal::serial::Serial::$usartX(
                        self.usart,
                        (tx, rx),
                        self.afio_mapr.unwrap(),
                        self.config,
                        self.clocks.unwrap(),
                        self.apb.unwrap(),
                    )
                }
            }
        )+
    }
}

serial! {
    /// # USART1 functions
    USART1: (
        usart1,
        APB2,
        gpioa,
        PA9,
        PA10,
        CRH,
    ),
    /// # USART1 functions
    USART1: (
        usart1,
        APB2,
        gpiob,
        PB6,
        PB7,
        CRL,
    ),
    /// # USART2 functions
    USART2: (
        usart2,
        APB1,
        gpioa,
        PA2,
        PA3,
        CRL,
    ),
    /// # USART2 functions
    USART2: (
        usart2,
        APB1,
        gpiod,
        PD5,
        PD6,
        CRL,
    ),
    /// # USART3 functions
    USART3: (
        usart3,
        APB1,
        gpiob,
        PB10,
        PB11,
        CRH,
    ),
    /// # USART3 functions
    USART3: (
        usart3,
        APB1,
        gpioc,
        PC10,
        PC11,
        CRH,
    ),
    /// # USART3 functions
    USART3: (
        usart3,
        APB1,
        gpiod,
        PD8,
        PD9,
        CRH,
    ),
}
