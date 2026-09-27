#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::Config;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};


#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Config::default());
    let mut led = Output::new(p.PC13, Level::High, Speed::Low);

    loop {
        led.toggle();
        Timer::after_millis(1u64).await;
    }
}

// #[entry]
// fn main() -> ! {
//     // Safety Delay for USB flashing bootloader
//     cortex_m::asm::delay(2_000_000);
//
//
//     let dp = Peripherals::take().unwrap();
//     let mut rcc = dp.RCC.freeze(Config::default());
//
//     // Split peripheral blocks
//     let gpioa = dp.GPIOA.split(&mut rcc);
//     let gpioc = dp.GPIOC.split(&mut rcc);
//
//     // Configure the onboard LED and the KEY button (PA0)
//     let mut led = gpioc.pc13.into_push_pull_output();
//     let button = gpioa.pa0.into_pull_up_input();
//
//     // Set up a hardware timer for the debounce delay step
//     let mut delay = dp.TIM1.delay_ms(&mut rcc);
//
//     // State variable to track if the button was previously registered as pressed
//     let mut button_was_pressed = false;
//
//     let mut state = State {
//         speed: 200,
//         min: 200,
//         max: 1000,
//     };
//
//
//     loop {
//         if button.is_low() {
//             if !button_was_pressed {
//                 state.next();
//                 button_was_pressed = true;
//
//                 delay.delay_ms(50u32);
//             }
//         } else {
//             button_was_pressed = false;
//         }
//         delay.delay_ms(state.speed);
//         led.toggle();
//     }
// }

struct State {
    speed: u32,
    min: u32,
    max: u32,
}

impl State {
    fn next(&mut self) {
        let (new_speed, overflow) = self.speed.overflowing_add(200);
        if overflow || (new_speed > self.max) {
            self.speed = self.min;
        } else {
            self.speed = new_speed;
        }
    }
}
