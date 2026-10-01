#![no_std]
#![no_main]

mod gui;
mod tasks;
mod java_logo;

use crate::tasks::display_task::{number_counter_task, update_display, NUMBER_TIM_SIGNAL};
use crate::tasks::smth_task::{blink_led, button_check};
use core::mem::forget;
use defmt::export::display;
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_stm32::i2c::I2c;
use embassy_stm32::interrupt::typelevel::EXTI0;
use embassy_stm32::peripherals::{DMA1_CH5, DMA1_CH6, I2C1};
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::low_level::{RoundTo, Timer};
use embassy_stm32::{bind_interrupts, dma, exti, i2c, interrupt, pac, Config};
use embassy_stm32::interrupt::{InterruptExt, Priority};
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, ThreadModeRawMutex};
use embassy_sync::signal::Signal;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::Drawable;
use embedded_graphics::geometry::Point;
use embedded_graphics::image::{Image, ImageRaw};
use embedded_graphics::pixelcolor::BinaryColor;
use ssd1306::mode::DisplayConfigAsync;
use ssd1306::rotation::DisplayRotation::Rotate0;
use ssd1306::size::DisplaySize128x64;
use ssd1306::Ssd1306Async;
use {defmt_rtt as _, panic_probe as _};
use crate::java_logo::{JAVA_LOGO_128X64, LAIN_128X64, SYSTEMD_128X64};

static BLINK_CHANGE_SIGNAL: Signal<ThreadModeRawMutex, u64> = Signal::new();
static LED_TOGGLE_SIGNAL: Signal<ThreadModeRawMutex, bool> = Signal::new();
static BUTTON_LONG_PRESS_BLINK_SIGNAL: Signal<ThreadModeRawMutex, ()> = Signal::new();

pub static TIM_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

static MIN_SPEED: u64 = 500;
static MAX_SPEED: u64 = 1000;
static STEP: u64 = 500;

bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<I2C1>;

    DMA1_STREAM6 => dma::InterruptHandler<DMA1_CH6>;
    DMA1_STREAM5 => dma::InterruptHandler<DMA1_CH5>;
    EXTI0 => exti::InterruptHandler<EXTI0>;
});

#[interrupt]
unsafe fn TIM2() {
    let reqs = pac::TIM2;

    if reqs.sr().read().uif() {
        reqs.sr().modify(|w| w.set_uif(false));
        // TIM_SIGNAL.signal(());
        NUMBER_TIM_SIGNAL.signal(());
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Config::default());

    let timer = Timer::new(p.TIM2);
    timer.set_frequency(Hertz::hz(1), RoundTo::Faster);
    timer.enable_update_interrupt(true);
    timer.clear_update_interrupt();

    unsafe {
        pac::TIM2.dier().modify(|w| w.set_uie(true));
        interrupt::TIM2.set_priority(Priority::P1);
        interrupt::TIM2.enable();
    }

    timer.start();
    forget(timer);

    let button = ExtiInput::new(p.PA0, p.EXTI0, Pull::Up, Irqs);
    spawner.spawn(button_check(button).unwrap());
    let led = Output::new(p.PC13, Level::High, Speed::Low);
    spawner.spawn(blink_led(led).unwrap());

    let mut i2c_config = i2c::Config::default();
    i2c_config.frequency = Hertz(400_000);
    let i2c = I2c::new(
        p.I2C1, p.PB8, p.PB9, p.DMA1_CH6, p.DMA1_CH5, Irqs, i2c_config,
    );

    let interface = ssd1306::I2CDisplayInterface::new(i2c);
    let display_base = Ssd1306Async::new(interface, DisplaySize128x64, Rotate0);

    let mut display = display_base.into_buffered_graphics_mode();
    let image_raw: ImageRaw<BinaryColor> = ImageRaw::new(&LAIN_128X64, 128);
    display.init().await.unwrap();

    display.clear(BinaryColor::Off).unwrap();
    Image::new(&image_raw, Point::zero())
        .draw(&mut display)
        .unwrap();

    display.flush().await.unwrap();

    // spawner.spawn(update_display(display).unwrap());
    // spawner.spawn(number_counter_task().unwrap());
}
