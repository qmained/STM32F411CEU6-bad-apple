#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::i2c::{I2c, Master};
use embassy_stm32::interrupt::typelevel::EXTI0;
use embassy_stm32::mode::Async;
use embassy_stm32::peripherals::{DMA1_CH5, DMA1_CH6, I2C1};
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, dma, exti, i2c, Config};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Instant};
use embedded_graphics::image::{Image, ImageRaw};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::Point;
use embedded_graphics::Drawable;
use heatshrink::decoder::HeatshrinkDecoder;
use heatshrink::{Poll, SinkError};
use ssd1306::mode::{BufferedGraphicsModeAsync, DisplayConfigAsync};
use ssd1306::prelude::I2CInterface;
use ssd1306::rotation::DisplayRotation::Rotate0;
use ssd1306::size::DisplaySize128x64;
use ssd1306::Ssd1306Async;
use {defmt_rtt as _, panic_probe as _};

pub type SsdDisplay = Ssd1306Async<
    I2CInterface<I2c<'static, Async, Master>>,
    DisplaySize128x64,
    BufferedGraphicsModeAsync<DisplaySize128x64>,
>;

static VIDEO_DATA: &[u8] = include_bytes!("../output.bin");

pub static TIM_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<I2C1>;

    DMA1_STREAM6 => dma::InterruptHandler<DMA1_CH6>;
    DMA1_STREAM5 => dma::InterruptHandler<DMA1_CH5>;
    EXTI0 => exti::InterruptHandler<EXTI0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();

    {
        use embassy_stm32::rcc::*;
        use embassy_stm32::time::Hertz;

        config.rcc.hse = Some(Hse {
            freq: Hertz(25_000_000),
            mode: HseMode::Oscillator,
        });
        config.rcc.pll_src = PllSource::HSE;

        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV25,
            mul: PllMul::MUL200,
            divp: Some(PllPDiv::DIV2),
            divq: Some(PllQDiv::DIV4),
            divr: None,
        });

        config.rcc.sys = Sysclk::PLL1_P;
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV2;
        config.rcc.apb2_pre = APBPrescaler::DIV1;
    }

    let p = embassy_stm32::init(config);

    let mut i2c_config = i2c::Config::default();
    i2c_config.frequency = Hertz(400_000);
    let i2c = I2c::new(
        p.I2C1, p.PB8, p.PB9, p.DMA1_CH6, p.DMA1_CH5, Irqs, i2c_config,
    );

    let interface = ssd1306::I2CDisplayInterface::new(i2c);
    let display_base = Ssd1306Async::new(interface, DisplaySize128x64, Rotate0);

    let mut display = display_base.into_buffered_graphics_mode();
    display.init().await.unwrap();

    loop {
        let mut decoder: HeatshrinkDecoder<10, 4, 128, 1024> = HeatshrinkDecoder::new();

        let mut output_buffer = [0u8; 1024];
        let mut buffer_size = 0;

        let mut flash_index = 0;
        let start_time = Instant::now();
        let mut frame_count = 0;
        let frame_duration = Duration::from_micros(106829);

        while flash_index < VIDEO_DATA.len() {
            match decoder.sink(&VIDEO_DATA[flash_index..]) {
                Ok(n) => flash_index += n,
                Err(SinkError::Full) => {}
                Err(SinkError::Misuse) => panic!("Misuse"),
            }

            loop {
                let mut free_space = &mut output_buffer[buffer_size..];

                match decoder.poll(&mut free_space) {
                    Ok(Poll::More(n)) => {
                        buffer_size += n;

                        if buffer_size == 1024 {
                            draw_to_display(
                                &output_buffer,
                                &mut display,
                                start_time,
                                &mut frame_count,
                                frame_duration,
                            )
                            .await;
                            buffer_size = 0;
                        }
                        continue;
                    }
                    Ok(Poll::Empty(n)) => {
                        buffer_size += n;

                        if buffer_size == 1024 {
                            draw_to_display(
                                &output_buffer,
                                &mut display,
                                start_time,
                                &mut frame_count,
                                frame_duration,
                            )
                            .await;
                            buffer_size = 0;
                        }
                        break;
                    }
                    Err(e) => panic!("Err: {e:?}"),
                }
            }
        }

        embassy_time::Timer::after_secs(2).await;
    }
}

async fn draw_to_display(
    buf: &[u8],
    display: &mut SsdDisplay,
    start_time: Instant,
    frame_count: &mut u64,
    frame_duration: Duration,
) {
    let next_frame = start_time + (frame_duration * (*frame_count) as u32);
    *frame_count += 1;
    let raw = ImageRaw::<BinaryColor>::new(buf, 128);
    Image::new(&raw, Point::zero()).draw(display).unwrap();
    display.flush().await.unwrap();

    embassy_time::Timer::at(next_frame).await;
}
