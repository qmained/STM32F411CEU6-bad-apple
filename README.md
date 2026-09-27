# STM32F411 Quickstart Template
A template for working with the STM32F411RETx MCU using either stm32f4xx-hal or Embassy.  
Options for flashing/debugging with openocd/gdb, probe-rs, and cargo embed.  
Includes VS Code JSON files for easy in-IDE debugging with probe-rs.

This template will mostly work with other STM32 models, you will just have to update memory.x
and the chip name in various files.

## How to Use
This template includes files for several different development/debugging strategies, so not all
may be necessary to your workflow. But regardless, the first steps are:

1. Run `cargo generate --git https://github.com/kurtjd/stm32f411-quickstart` to generate a project
1. If not using Embassy, you can safely remove main.rs.embassy and the Embassy dependencies from Cargo.toml
2. If you are using Embassy, uncomment out the Embassy dependencies from Cargo.toml and replace main.rs with main.rs.embassy

Steps for your preferred development enviroment are below.

### OpenOCD + GDB
1. Run `openocd` (uses settings in openocd.cfg)
2. From another terminal, run `gdb -q -x target/thumbv7em-none-eabihf/debug/blackpill` (uses settings in openocd.gdb)

### probe-rs
1. To flash/execute, run `cargo run` (.cargo/config.toml includes runner setting for probe-rs)
2. To debug, run `probe-rs debug --chip STM32F411RETx`

### cargo-embed
1. Run `cargo-embed` (defaults to no GDB, but can change this in Embed.toml)

### VS Code
1. Ensure rust-analyzer and probe-rs VS Code extensions are installed
2. Various tasks provided for building and running (in both debug and release mode)
3. Run and Debug supported via probe-rs extension and the stm32f411 SVD file
