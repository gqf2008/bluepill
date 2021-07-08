use crate::hal::afio::MAPR;
use crate::hal::gpio::gpioa;
use crate::hal::gpio::gpiob;
use crate::hal::gpio::gpioc;
use crate::hal::gpio::gpiod;
use crate::hal::gpio::{Alternate, Floating, Input, PushPull};
use crate::hal::pac::interrupt;
use crate::hal::pac::{USART1, USART2, USART3};
use crate::hal::rcc::{Clocks, APB1, APB2};
use crate::hal::serial::Pins;
use crate::hal::serial::{Config, StopBits};
use crate::hal::serial::{Rx, Tx};
use crate::hal::time::U32Ext;

use alloc::collections::VecDeque;
use core::cell::RefCell;
use core::convert::Infallible;
use cortex_m::interrupt::Mutex;
use embedded_hal::serial::{Read, Write};

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

                pub fn build_rw(
                    self,
                ) -> RW<Tx<$USARTX>> {
                    let my = self.build();
                    RW::<Tx<$USARTX>>::new(my)
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

pub struct RW<W> {
    tx: W,
}

macro_rules! rw {
    ($(
        $(#[$meta:meta])*
        $USARTX:ident: ($RXX:ident, $BUFX:ident),
    )+) => {
        $(
            $(#[$meta])*

        impl RW<Tx<$USARTX>> {
           pub fn new<PINS>(serial: crate::hal::serial::Serial<$USARTX, PINS>) -> Self
            where
                PINS: Pins<$USARTX>,
            {
                let (tx, mut rx) = serial.split();
                rx.listen();
                cortex_m::interrupt::free(|cs| unsafe{
                    $RXX.replace(rx);
                    $BUFX.borrow(cs).replace(Some(VecDeque::with_capacity(64)));
                });
                crate::enable_interrupt(crate::hal::pac::Interrupt::$USARTX);
                Self { tx }
            }
        }

        impl Write<u8> for RW<Tx<$USARTX>> {
            type Error = Infallible;

            fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
                self.tx.write(word)
            }

            fn flush(&mut self) -> nb::Result<(), Self::Error> {
                self.tx.flush()
            }
        }

        impl Read<u8> for RW<Tx<$USARTX>> {
            type Error = crate::io::Error;
            fn read(&mut self) -> nb::Result<u8, Self::Error> {
                cortex_m::interrupt::free(|cs| {
                    match $BUFX.borrow(cs).borrow_mut().deref_mut() {
                        Some(buf) => {
                            match buf.pop_front() {
                                Some(w) => {
                                    return Ok(w);
                                }
                                None => {
                                    return Err(nb::Error::WouldBlock);
                                }
                            }
                        }
                        None => return Err(nb::Error::Other(crate::io::Error::NoIoDevice)),
                    }
                })
            }
        }
        )+
    }
}

rw! {
    USART1:(RX1, TX1_BUFFER),
    USART2:(RX2, TX2_BUFFER),
    USART3:(RX3, TX3_BUFFER),
}

static mut RX1: Option<Rx<USART1>> = None;
static TX1_BUFFER: Mutex<RefCell<Option<VecDeque<u8>>>> = Mutex::new(RefCell::new(None));

static mut RX2: Option<Rx<USART2>> = None;
static TX2_BUFFER: Mutex<RefCell<Option<VecDeque<u8>>>> = Mutex::new(RefCell::new(None));

static mut RX3: Option<Rx<USART3>> = None;
static TX3_BUFFER: Mutex<RefCell<Option<VecDeque<u8>>>> = Mutex::new(RefCell::new(None));

#[interrupt]
unsafe fn USART1() {
    if let Some(rx) = RX1.as_mut() {
        if let Ok(w) = nb::block!(rx.read()) {
            cortex_m::interrupt::free(|cs| {
                if let Some(buf) = TX1_BUFFER.borrow(cs).borrow_mut().deref_mut() {
                    buf.push_back(w);
                }
            });
        }
    }
}

use core::ops::DerefMut;
#[interrupt]
unsafe fn USART2() {
    // static mut RX: Option<Rx<USART2>> = None;
    // let rx = RX.get_or_insert_with(|| {
    //     cortex_m::interrupt::free(|cs| RX2.borrow(cs).replace(None).unwrap())
    // });
    if let Some(rx) = RX2.as_mut() {
        if let Ok(w) = nb::block!(rx.read()) {
            cortex_m::interrupt::free(|cs| {
                if let Some(buf) = TX2_BUFFER.borrow(cs).borrow_mut().deref_mut() {
                    buf.push_back(w);
                }
            });
        }
    }
}

#[interrupt]
unsafe fn USART3() {
    if let Some(rx) = RX3.as_mut() {
        if let Ok(w) = nb::block!(rx.read()) {
            cortex_m::interrupt::free(|cs| {
                if let Some(buf) = TX3_BUFFER.borrow(cs).borrow_mut().deref_mut() {
                    buf.push_back(w);
                }
            });
        }
    }
}
