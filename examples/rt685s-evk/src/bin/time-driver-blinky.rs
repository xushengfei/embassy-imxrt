#![no_main]
#![no_std]

use core::borrow::Borrow;

use cortex_m_rt::entry;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_imxrt as _;
use embassy_time::{Duration, Timer};
use mimxrt685s_pac as pac;
use panic_probe as _;

use defmt::info;

#[link_section = ".otfad"]
#[used]
static OTFAD: [u8; 256] = [0; 256];

#[link_section = ".fcb"]
#[used]
static FCB: [u8; 512] = [
    0x46, 0x43, 0x46, 0x42, 0x00, 0x00, 0x02, 0x56, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x03, 0x00,
    0x01, 0x02, 0x01, 0x00, 0x01, 0x06, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x50, 0x00, 0x00, 0x00, 0x01, 0x08, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xee, 0x87, 0x11, 0x87, 0x20, 0x8b, 0x29, 0xb3, 0x04, 0xa7, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x05, 0x04, 0x04, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x05, 0x87, 0xfa, 0x87, 0x20, 0x8b, 0x14, 0xb3, 0x04, 0xa7, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x06, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x06, 0x87, 0xf9, 0x87, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x21, 0x87, 0xde, 0x87, 0x20, 0x8b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x72, 0x04, 0x00, 0x04, 0x00, 0x04, 0x00, 0x04, 0x00, 0x04, 0x01, 0x20, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xdc, 0x87, 0x23, 0x87, 0x20, 0x8b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x12, 0x87, 0xed, 0x87, 0x20, 0x8b, 0x04, 0xa3, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x60, 0x87, 0x9f, 0x87, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x01, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x01, 0x00, 0x00, 0x82, 0x00, 0x07, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

#[link_section = ".biv"]
#[used]
static BOOT_IMAGE_VERSION: u32 = 0x01000000;

#[link_section = ".keystore"]
#[used]
static KEYSTORE: [u8; 2048] = [0; 2048];

fn clock_ctrls() -> (
    &'static pac::clkctl1::RegisterBlock,
    &'static pac::rstctl1::RegisterBlock,
) {
    unsafe { (&*pac::Clkctl1::ptr(), &*pac::Rstctl1::ptr()) }
}

fn gpios() -> &'static pac::gpio::RegisterBlock {
    unsafe { (&*pac::Gpio::ptr()) }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let (cc1, rc1) = clock_ctrls();
    // Enable GPIO0 Clock
    info!("Enabling GPIO0 clock");
    cc1.pscctl1_set().write(|w| w.hsgpio0_clk_set().set_clock());

    // Take GPIO0 out of reset
    info!("Clearing GPIO0 reset");
    rc1.prstctl1_clr()
        .write(|w| w.hsgpio0_rst_clr().clr_reset());

    // Set GPIO0_26 (Blue LED) as ouptut
    info!("Configuring GPIO0_26 (Blue LED) as output");
    let g = gpios();
    g.dirset(0).write(|w| unsafe { w.bits(1 << 26) });

    if let embassy_p = embassy_imxrt::init(Default::default()) {
        info!("Initializing pin mux");
        board_init_pins(&embassy_p);

        loop {
            info!("Toggling GPIO0_26 (Blue LED)");
            g.not(0).write(|w| unsafe { w.bits(1 << 26) });
            Timer::after_millis(5000).await;
            //cortex_m::asm::delay(10_000_000);
        }
    } else {
        panic!("Unable to access Peripherals");
    };
}

fn iopctl_reg() -> &'static pac::iopctl::RegisterBlock {
    unsafe { (&*pac::Iopctl::ptr()) }
}

fn board_init_pins(p: &embassy_imxrt::Peripherals) {
    // Configure IO Pad Control 0_26
    //
    // Pin is configured as PIO0_26
    // Disable pull-up / pull-down function
    // Enable pull-down function
    // Disable input buffer function
    // Normal mode
    // Normal drive
    // Analog mux is disabled
    // Pseudo Output Drain is disabled
    // Input function is not inverted
    let iopctrl = iopctl_reg();
    iopctrl.pio0_26().write(|w| {
        w.fsel()
            .function_0()
            .pupdena()
            .disabled()
            .pupdsel()
            .pull_down()
            .ibena()
            .disabled()
            .slewrate()
            .normal()
            .fulldrive()
            .normal_drive()
            .amena()
            .disabled()
            .odena()
            .disabled()
            .iiena()
            .disabled()
    });
}
