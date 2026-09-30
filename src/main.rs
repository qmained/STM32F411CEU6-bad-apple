#![no_std]
#![no_main]

mod tasks {
    pub mod smth_task;
    pub mod display_task;
}

use crate::tasks::display_task::update_display;
use crate::tasks::smth_task::{blink_led, button_check};
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_stm32::i2c::I2c;
use embassy_stm32::peripherals::I2C1;
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, i2c, Config};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::signal::Signal;
use ssd1306::rotation::DisplayRotation::Rotate0;
use ssd1306::size::DisplaySize128x64;
use ssd1306::Ssd1306Async;
use {defmt_rtt as _, panic_probe as _};

static BLINK_CHANGE_SIGNAL: Signal<ThreadModeRawMutex, u64> = Signal::new();
static LED_TOGGLE_SIGNAL: Signal<ThreadModeRawMutex, bool> = Signal::new();
static BUTTON_LONG_PRESS_BLINK_SIGNAL: Signal<ThreadModeRawMutex, ()> = Signal::new();

static MIN_SPEED: u64 = 500;
static MAX_SPEED: u64 = 1000;
static STEP: u64 = 500;



// Bind the I2C interrupt vector for asynchronous operation
bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<I2C1>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Config::default());

    let button = ExtiInput::new(p.PA0, p.EXTI0, Pull::Up);
    defmt::unwrap!(spawner.spawn(button_check(button)));

    let led = Output::new(p.PC13, Level::High, Speed::Low);
    defmt::unwrap!(spawner.spawn(blink_led(led)));

    let i2c_config = i2c::Config::default();
    let i2c = I2c::new(
        p.I2C1,
        p.PB8,
        p.PB9,
        Irqs,
        p.DMA1_CH6,
        p.DMA1_CH5,
        Hertz(400_000),
        i2c_config,
    );

    let interface = ssd1306::I2CDisplayInterface::new(i2c);
    let display_base = Ssd1306Async::new(interface, DisplaySize128x64, Rotate0);

    let display = display_base.into_buffered_graphics_mode();
    defmt::unwrap!(spawner.spawn(update_display(display)));
}
