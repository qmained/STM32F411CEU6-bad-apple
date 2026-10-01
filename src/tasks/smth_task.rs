use crate::{
    BLINK_CHANGE_SIGNAL, BUTTON_LONG_PRESS_BLINK_SIGNAL, LED_TOGGLE_SIGNAL, MAX_SPEED, MIN_SPEED,
    STEP,
};
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::Output;
use embassy_time::Timer;


#[embassy_executor::task]
pub async fn blink_led(mut led: Output<'static>) -> ! {
    let mut blinking_speed = MIN_SPEED;
    let mut led_enabled = true;
    loop {
        if let Some(speed) = BLINK_CHANGE_SIGNAL.try_take() {
            blinking_speed = speed;
        }

        if let Some(led) = LED_TOGGLE_SIGNAL.try_take() {
            led_enabled = led;
        }

        if BUTTON_LONG_PRESS_BLINK_SIGNAL.try_take().is_some() {
            led.toggle();
            Timer::after_millis(100).await;
            led.toggle();
            Timer::after_millis(100).await;
        }

        if led_enabled {
            led.toggle();
        }
        Timer::after_millis(blinking_speed).await;
    }
}

#[embassy_executor::task]
pub async fn button_check(mut button: ExtiInput<'static, embassy_stm32::mode::Async>) -> ! {
    let mut current_speed = MIN_SPEED;
    let mut led_enabled = true;

    loop {
        match button.wait_for_press().await {
            ButtonEvent::ShortPress => {
                current_speed += STEP;
                if current_speed > MAX_SPEED {
                    current_speed = MIN_SPEED;
                }
                BLINK_CHANGE_SIGNAL.signal(current_speed);
            }
            ButtonEvent::LongPress => {
                led_enabled.toggle();
                LED_TOGGLE_SIGNAL.signal(led_enabled);
            }
        }

        Timer::after_millis(50).await;
    }
}

enum ButtonEvent {
    ShortPress,
    LongPress,
}

trait ExtiButtonExt {
    async fn wait_for_press(&mut self) -> ButtonEvent;
}

impl ExtiButtonExt for ExtiInput<'static, embassy_stm32::mode::Async> {
    async fn wait_for_press(&mut self) -> ButtonEvent {
        self.wait_for_low().await;

        for _ in 0..30 {
            Timer::after_millis(50).await;
            if self.is_high() {
                return ButtonEvent::ShortPress;
            }
        }
        BUTTON_LONG_PRESS_BLINK_SIGNAL.signal(());
        self.wait_for_high().await;
        ButtonEvent::LongPress
    }
}
