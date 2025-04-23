#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts, init,
    peripherals::UART0,
    uart,
    uart::{BufferedInterruptHandler, Config},
};
use embedded_io_async::Read;
use static_cell::ConstStaticCell;

use defmt::info;
use {defmt_rtt as _, panic_probe as _};

static TX_BUF_CELL: ConstStaticCell<[u8; 128]> = ConstStaticCell::new([0; 128]);
static RX_BUF_CELL: ConstStaticCell<[u8; 256]> = ConstStaticCell::new([0; 256]);

// Bind the UART0 interrupt to the buffered‑UART handler
bind_interrupts!(struct Irqs {
    UART0_IRQ => BufferedInterruptHandler<UART0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize the RP2350 HAL
    let p = init(Default::default());

    // Standard 115200 baud; tweak as needed
    let mut config = Config::default();
    config.baudrate = 115200;

    // safely "take" two &'static mut buffers
    let tx_buf: &'static mut [u8; 128] = TX_BUF_CELL.take();
    let rx_buf: &'static mut [u8; 256] = RX_BUF_CELL.take();

    let uart = uart::BufferedUart::new(
        p.UART0, Irqs,    // our bound interrupt struct
        p.PIN_0, // TX pin
        p.PIN_1, // RX pin
        tx_buf,  // TX backing buffer
        rx_buf,  // RX backing buffer
        config,
    );

    // Split into TX and RX halves
    let (_tx, mut rx) = uart.split();

    // Local read buffer
    let mut buf = [0u8; 256];

    loop {
        // read up to 256 bytes; returns actual count in `n`
        let n = rx.read(&mut buf).await.unwrap();
        info!("Got {} bytes: {:?}", n, &buf[..n]);
    }
}
