use crate::gui::large_point::LargePoint;
use arrayvec::ArrayString;
use core::fmt::Write;
use embassy_stm32::i2c::{I2c, Master};
use embassy_stm32::mode::Async;
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, ThreadModeRawMutex};
use embassy_sync::signal::Signal;
use embedded_graphics::geometry::Point;
use embedded_graphics::mono_font::ascii::{FONT_5X7, FONT_9X18_BOLD};
use embedded_graphics::mono_font::iso_8859_2::FONT_6X10;
use embedded_graphics::mono_font::{MonoTextStyle, MonoTextStyleBuilder};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::DrawTarget;
use embedded_graphics::text::{Alignment, Baseline, Text, TextStyleBuilder};
use embedded_graphics::Drawable;
use ssd1306::mode::BufferedGraphicsModeAsync;
use ssd1306::prelude::{DisplayConfigAsync, DisplaySize128x64, I2CInterface};
use ssd1306::Ssd1306Async;

pub static NUMBER_TIM_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();
pub static NUMBER_UPDATE_SIGNAL: Signal<ThreadModeRawMutex, u32> = Signal::new();

pub type SsdDisplay = Ssd1306Async<
    I2CInterface<I2c<'static, Async, Master>>,
    DisplaySize128x64,
    BufferedGraphicsModeAsync<DisplaySize128x64>,
>;

#[embassy_executor::task]
pub async fn update_display(mut display: SsdDisplay) -> ! {
    display.init().await.unwrap();

    display.clear(BinaryColor::Off).unwrap();
    Text::with_text_style(
        "Hello world!",
        Point::new(64, 5),
        MonoTextStyle::new(&FONT_9X18_BOLD, BinaryColor::On),
        TextStyleBuilder::new()
            .alignment(Alignment::Center)
            .baseline(Baseline::Top)
            .build(),
    )
    .draw(&mut display)
    .unwrap();

    display.flush().await.unwrap();

    let mut buf = ArrayString::<48>::new();
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_5X7)
        .text_color(BinaryColor::On)
        .background_color(BinaryColor::Off)
        .build();

    let pos_text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .background_color(BinaryColor::Off)
        .build();

    let mut large_point = LargePoint::new(0, 0, 4, 4);

    Text::with_alignment(
        "Sec since last start:",
        Point::new(64, 30),
        text_style,
        Alignment::Center,
    )
    .draw(&mut display)
    .unwrap();

    loop {
        buf.clear();
        if let Some(new_num) = NUMBER_UPDATE_SIGNAL.try_take() {
            write!(buf, "{}", new_num).unwrap();
            Text::with_alignment(
                buf.as_str(),
                Point::new(64, 40),
                text_style,
                Alignment::Center,
            )
            .draw(&mut display)
            .unwrap();
        }

        buf.clear();
        write!(buf, "x: {:03} y: {:03}", large_point.x, large_point.y).unwrap();
        Text::with_alignment(
            buf.as_str(),
            Point::new(64, 55),
            pos_text_style,
            Alignment::Center,
        )
        .draw(&mut display)
        .unwrap();

        large_point.draw(&mut display, BinaryColor::Off);
        large_point.next_point();
        large_point.draw(&mut display, BinaryColor::On);

        display.flush().await.unwrap();
    }
}

#[embassy_executor::task]
pub async fn number_counter_task() -> ! {
    let mut number = u32::MAX;
    loop {
        NUMBER_TIM_SIGNAL.wait().await;
        number = number.wrapping_add(1);
        NUMBER_UPDATE_SIGNAL.signal(number);
    }
}
