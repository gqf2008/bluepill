// use cortex_m::interrupt::Mutex;

use crate::{sprint, sprintln};

use embedded_hal::digital::v2::{InputPin, OutputPin};
use panic_semihosting as _;
use stm32f1xx_hal::dma::dma1::Channels as DMA1;
use stm32f1xx_hal::flash::ACR;
use stm32f1xx_hal::gpio::gpioa::{PA0, PA4};
use stm32f1xx_hal::rcc::{Clocks, CFGR};
use stm32f1xx_hal::{
    delay::Delay,
    gpio::*,
    pac::{self, *},
    prelude::*,
    serial::{Config, Rx, Serial, Tx},
    timer::CountDownTimer,
    timer::Timer,
};

//use core::cell::RefCell;
use core::mem::MaybeUninit;

// //内核外设
// pub(crate) static CP: Mutex<RefCell<Option<cortex_m::Peripherals>>> =
//     Mutex::new(RefCell::new(None));

// //芯片外设
// pub(crate) static PAC: Mutex<RefCell<Option<pac::Peripherals>>> = Mutex::new(RefCell::new(None));

//板载LED
pub(crate) static mut LED: MaybeUninit<stm32f1xx_hal::gpio::gpioc::PC13<Output<PushPull>>> =
    MaybeUninit::uninit();
//USART1标准输出
pub static mut STDOUT: MaybeUninit<Tx<USART1>> = MaybeUninit::uninit();
//USART1标准输入
pub static mut STDIN: MaybeUninit<Rx<USART1>> = MaybeUninit::uninit();
//延时器
pub(crate) static mut DELAY: MaybeUninit<Delay> = MaybeUninit::uninit();
//定时器
pub(crate) static mut TIMER1: MaybeUninit<CountDownTimer<TIM1>> = MaybeUninit::uninit();
pub(crate) static mut TIMER2: MaybeUninit<CountDownTimer<TIM2>> = MaybeUninit::uninit();
pub(crate) static mut TIMER3: MaybeUninit<CountDownTimer<TIM3>> = MaybeUninit::uninit();
pub(crate) static mut TIMER4: MaybeUninit<CountDownTimer<TIM4>> = MaybeUninit::uninit();
//USART
pub(crate) static mut USART2: MaybeUninit<(Tx<USART2>, Rx<USART2>)> = MaybeUninit::uninit();
pub(crate) static mut USART3: MaybeUninit<(Tx<USART3>, Rx<USART3>)> = MaybeUninit::uninit();
//DMA通道
pub(crate) static mut DMA1: MaybeUninit<DMA1> = MaybeUninit::uninit();
// pub(crate) static mut DMA2: MaybeUninit<DMA2> = MaybeUninit::uninit();

pub(crate) static mut PA0: MaybeUninit<PA0<Output<PushPull>>> = MaybeUninit::uninit();
pub(crate) static mut PA4: MaybeUninit<PA4<Input<PullDown>>> = MaybeUninit::uninit();

#[inline]
pub fn init() {
    let (cp, p) = crate::Peripherals::take();
    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();
    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(48.mhz())
        .pclk1(24.mhz())
        .pclk2(24.mhz())
        .freeze(&mut flash.acr);
    assert!(clocks.usbclk_valid());
    let mut afio = p.AFIO.constrain(&mut rcc.apb2);
    let mut gpioa = p.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = p.GPIOB.split(&mut rcc.apb2);
    let mut gpioc = p.GPIOC.split(&mut rcc.apb2);

    //初始化USART1
    let tx = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
    let rx = gpioa.pa10;
    let stdout = Serial::usart1(
        p.USART1,
        (tx, rx),
        &mut afio.mapr,
        Config::default().baudrate(115200.bps()),
        clocks,
        &mut rcc.apb2,
    );

    let (tx, rx) = stdout.split();
    unsafe {
        STDOUT.as_mut_ptr().write(tx);
        STDIN.as_mut_ptr().write(rx);
    }

    sprintln!("初始化延时器");
    //初始化延时器
    unsafe {
        DELAY.as_mut_ptr().write(Delay::new(cp.SYST, clocks));
    }

    sprintln!("初始化定时器");
    //初始化定时器
    unsafe {
        TIMER1
            .as_mut_ptr()
            .write(Timer::tim1(p.TIM1, &clocks, &mut rcc.apb2).start_count_down(1.hz()));
        TIMER2
            .as_mut_ptr()
            .write(Timer::tim2(p.TIM2, &clocks, &mut rcc.apb1).start_count_down(1.hz()));
        TIMER3
            .as_mut_ptr()
            .write(Timer::tim3(p.TIM3, &clocks, &mut rcc.apb1).start_count_down(1.hz()));
        TIMER4
            .as_mut_ptr()
            .write(Timer::tim4(p.TIM4, &clocks, &mut rcc.apb1).start_count_down(1.hz()));
    }

    //初始化DMA通道
    sprintln!("初始化DMA1通道");
    unsafe {
        DMA1.as_mut_ptr().write(p.DMA1.split(&mut rcc.ahb));

        // hprintln!("初始化DMA2通道").unwrap();//打开DMA2会导致设备奔溃
        // DMA2.as_mut_ptr().write(p.DMA2.split(&mut rcc.ahb));
    }

    //初始化LED
    sprintln!("初始化LED");
    unsafe {
        LED.as_mut_ptr()
            .write(gpioc.pc13.into_push_pull_output(&mut gpioc.crh));
    }

    {
        //初始化USART2
        sprintln!("初始化USART2");
        let tx = gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl);
        let rx = gpioa.pa3;

        let serial = Serial::usart2(
            p.USART2,
            (tx, rx),
            &mut afio.mapr,
            Config::default().baudrate(115200.bps()),
            clocks,
            &mut rcc.apb1,
        );

        let txrx = serial.split();
        unsafe {
            USART2.as_mut_ptr().write(txrx);
        }
    }

    {
        //初始化USART3
        sprintln!("初始化USART3");
        // let mut b1 = gpiob.pb1.into_push_pull_output(&mut gpiob.crl);
        // b1.set_high().unwrap();
        let tx = gpiob.pb10.into_alternate_push_pull(&mut gpiob.crh);
        let rx = gpiob.pb11;

        let serial = Serial::usart3(
            p.USART3,
            (tx, rx),
            &mut afio.mapr,
            Config::default().baudrate(115200.bps()),
            clocks,
            &mut rcc.apb1,
        );

        let txrx = serial.split();
        unsafe {
            USART3.as_mut_ptr().write(txrx);
        }
    }

    {
        let mut pa0 = gpioa.pa0.into_push_pull_output(&mut gpioa.crl); //.into_alternate_push_pull(&mut gpioa.crl);
        pa0.set_speed(&mut gpioa.crl, IOPinSpeed::Mhz50);
        let pa4 = gpioa.pa4.into_pull_down_input(&mut gpioa.crl); // 下拉输入
        unsafe {
            PA0.as_mut_ptr().write(pa0);
            PA4.as_mut_ptr().write(pa4);
        }
    }

    sprintln!("SMT32F103C8T6初始化 ...... OK");
}

pub fn enable_interrupt(interrupt: Interrupt) {
    sprintln!("打开{:?}中断", interrupt);
    unsafe {
        cortex_m::peripheral::NVIC::unmask(interrupt);
    }
}

pub fn disable_interrupt(interrupt: Interrupt) {
    sprintln!("关闭{:?}中断", interrupt);
    cortex_m::peripheral::NVIC::mask(interrupt);
}

pub fn stdin<'a>() -> &'a mut Rx<USART1> {
    unsafe { &mut *STDIN.as_mut_ptr() }
}

pub fn stdout<'a>() -> &'a mut Tx<USART1> {
    unsafe { &mut *STDOUT.as_mut_ptr() }
}

pub fn usart2<'a>() -> (&'a mut Tx<USART2>, &'a mut Rx<USART2>) {
    let (tx, rx) = unsafe { &mut *USART2.as_mut_ptr() };
    (tx, rx)
}

pub fn usart3<'a>() -> (&'a mut Tx<USART3>, &'a mut Rx<USART3>) {
    let (tx, rx) = unsafe { &mut *USART3.as_mut_ptr() };
    (tx, rx)
}

pub fn timer1<'a>() -> &'a mut CountDownTimer<TIM1> {
    unsafe { &mut *TIMER1.as_mut_ptr() }
}

pub fn timer2<'a>() -> &'a mut CountDownTimer<TIM2> {
    unsafe { &mut *TIMER2.as_mut_ptr() }
}

pub fn timer3<'a>() -> &'a mut CountDownTimer<TIM3> {
    unsafe { &mut *TIMER3.as_mut_ptr() }
}

pub fn timer4<'a>() -> &'a mut CountDownTimer<TIM4> {
    unsafe { &mut *TIMER4.as_mut_ptr() }
}
