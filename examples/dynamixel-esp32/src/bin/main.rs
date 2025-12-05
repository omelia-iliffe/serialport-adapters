#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use defmt::info;
use esp_hal::clock::CpuClock;
use esp_hal::main;
use esp_hal::time::{Duration, Instant};
use {esp_backtrace as _, esp_println as _};

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    // generator version: 1.0.1

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let dynamixel_uart = peripherals.UART0;
    let dynamixel_tx = peripherals.GPIO9;
    let dynamixel_rx = peripherals.GPIO11;
    let dynamixel_dir = peripherals.GPIO10;
    let dynamixel_baudrate = 1_000_000;
    let dynamixel_uart = esp_hal::uart::Uart::new(
        dynamixel_uart,
        esp_hal::uart::Config::default().with_baudrate(dynamixel_baudrate),
    )
    .unwrap()
    .with_rx(dynamixel_rx)
    .with_tx(dynamixel_tx);
    let dynamixel_dir =
        esp_hal::gpio::Output::new(dynamixel_dir, esp_hal::gpio::Level::Low, Default::default());
    let dynamixel_uart = serialport_adapters::esp_hal::Rs485Uart::new(
        dynamixel_uart,
        dynamixel_dir,
        dynamixel_baudrate,
    )
    .unwrap();

    let mut client =
        dynamixel2::client::Client::with_buffers(dynamixel_uart, [0; 128], [0; 128]).unwrap();

    loop {
        info!("Starting scan at {}", dynamixel_baudrate);
        for response in client.scan().unwrap() {
            if let Ok(r) = response {
                esp_println::println!("{:?}", r);
            }
        }
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(500) {}
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0/examples/src/bin
}
