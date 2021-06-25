use embedded_hal::timer::CountDown;
use embedded_hal::{
    digital::v2::{InputPin, OutputPin},
    prelude::_embedded_hal_timer_CountDown,
    timer::Cancel,
};
use stm32f1xx_hal::{
    gpio::{
        gpioa::{CRL, PA0, PA4},
        Floating, IOPinSpeed, Input, Output, OutputSpeed, PullDown, PushPull,
    },
    pac::TIM1,
    prelude::*,
    timer::{CountDownTimer, Timer},
};

use crate::sprintln;

pub struct UltrasonicWave {
    trigger: PA0<Output<PushPull>>,
    echo: PA4<Input<PullDown>>,
    timer: CountDownTimer<TIM1>,
}

impl UltrasonicWave {
    pub fn configure(
        pins: (PA0<Input<Floating>>, PA4<Input<Floating>>),
        crl: &mut CRL,
        timer: Timer<TIM1>,
    ) -> Self {
        let mut trigger = pins.0.into_push_pull_output(crl); //.into_alternate_push_pull(&mut gpioa.crl);
        trigger.set_speed(crl, IOPinSpeed::Mhz50);
        let echo = pins.1.into_pull_down_input(crl); // 下拉输入

        Self {
            trigger,
            echo,
            timer: timer.start_count_down(1.hz()),
        }
    }
    fn start_measure(&mut self) {
        //sprintln!("拉高Trig电平");
        self.timer.start(20.us());
        self.trigger.set_high().ok(); //拉高电平
        nb::block!(self.timer.wait()).ok(); //持续20us
                                            //crate::delay_us(20); //持续20us
                                            //sprintln!("拉低Trig电平");
        self.trigger.set_low().ok(); //拉低电平
    }
    pub fn measure(&mut self) -> f64 {
        //sprintln!("等待Echo至低电平");
        loop {
            //echo为高电平时,则等待至低电平,才启动超声波
            if self.echo.is_low().unwrap() {
                break;
            }
        }
        //sprintln!("启动超声波");
        self.start_measure(); //启动超声波
                              //sprintln!("等待Echo的高电平到来");
        loop {
            //等待echo的高电平到来
            if self.echo.is_high().unwrap() {
                break;
            }
        }

        //self.timer.cancel().ok();
        //self.timer.reset();
        self.timer.start(5.us());
        //sprintln!("等待Echo的高电平结束");
        loop {
            //等待echo的高电平结束
            if self.echo.is_low().unwrap() {
                break;
            }
        }
        let mms = self.timer.micros_since();
        //sprintln!("Timer Micros {}", mms);
        self.timer.cancel().ok();
        mms as f64 / 1000000.0 * 340.0 / 2.0 * 1000.0
        //counter / 340 / 2 * 100
        // while(GPIO_ReadInputDataBit(GPIOB,GPIO_Pin_10) == 0);//等待echo的高电平到来
        // TIM_SetCounter(TIM2,0); //清零计数器
        // TIM_Cmd(TIM2, ENABLE);  //使能定时器2,开始计数
        // while(GPIO_ReadInputDataBit(GPIOB,GPIO_Pin_10) == 1);//等待echo的高电平结束
        // TIM_Cmd(TIM2, DISABLE);    //失能定时器2,截止计数
        // return (TIM_GetCounter(TIM2))/1000000*340/2 *100;   //此处单位转换为cm
        // 0
    }
}
