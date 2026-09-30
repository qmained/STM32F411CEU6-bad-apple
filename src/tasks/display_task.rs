use arrayvec::ArrayString;
use core::fmt::Write;
use embassy_stm32::i2c::I2c;
use embassy_stm32::mode::Async;
use embassy_time::Timer;
use embedded_graphics::geometry::Point;
use embedded_graphics::mono_font::ascii::{FONT_8X13, FONT_9X18_BOLD};
use embedded_graphics::mono_font::{MonoTextStyle, MonoTextStyleBuilder};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::DrawTarget;
use embedded_graphics::text::{Alignment, Baseline, Text, TextStyleBuilder};
use embedded_graphics::Drawable;
use ssd1306::mode::BufferedGraphicsModeAsync;
use ssd1306::prelude::{DisplayConfigAsync, DisplaySize128x64, I2CInterface};
use ssd1306::Ssd1306Async;

#[embassy_executor::task]
pub async fn update_display(
    mut display: Ssd1306Async<
        I2CInterface<I2c<'static, Async>>,
        DisplaySize128x64,
        BufferedGraphicsModeAsync<DisplaySize128x64>,
    >,
) -> ! {
    display.init().await.unwrap();

    display.clear(BinaryColor::Off).unwrap();
    Text::with_text_style(
        "Hello world!",
        Point::new(64, 0),
        MonoTextStyle::new(&FONT_9X18_BOLD, BinaryColor::On),
        TextStyleBuilder::new()
            .alignment(Alignment::Center)
            .baseline(Baseline::Top)
            .build(),
    )
    .draw(&mut display)
    .unwrap();

    display.flush().await.unwrap();

    let mut number = 0u8;
    let mut buf = ArrayString::<16>::new();
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_8X13)
        .text_color(BinaryColor::On)
        .background_color(BinaryColor::Off)
        .build();

    loop {
        buf.clear();
        let _ = write!(buf, "Num: {:03}", number);
        Text::with_alignment(
            buf.as_str(),
            Point::new(64, 40),
            text_style,
            Alignment::Center,
        )
        .draw(&mut display)
        .unwrap();
        display.flush().await.unwrap();
        Timer::after_millis(1000).await;
        number = number.wrapping_add(1);
    }
}
