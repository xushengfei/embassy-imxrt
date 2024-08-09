#![no_main]
#![no_std]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::Timer;
use {defmt_rtt as _, embassy_imxrt as _, mimxrt685s_pac as pac, panic_probe as _};

fn clock_ctrls() -> (
    &'static pac::clkctl1::RegisterBlock,
    &'static pac::rstctl1::RegisterBlock,
) {
    unsafe { (&*pac::Clkctl1::ptr(), &*pac::Rstctl1::ptr()) }
}

fn gpios() -> &'static pac::gpio::RegisterBlock {
    unsafe { &*pac::Gpio::ptr() }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let (cc1, rc1) = clock_ctrls();
    // Enable GPIO0 Clock
    info!("Enabling GPIO0 clock");
    cc1.pscctl1_set().write(|w| w.hsgpio0_clk_set().set_clock());

    // Take GPIO0 out of reset
    info!("Clearing GPIO0 reset");
    rc1.prstctl1_clr().write(|w| w.hsgpio0_rst_clr().clr_reset());

    // Set GPIO0_26 (Blue LED) as ouptut
    info!("Configuring GPIO0_26 (Blue LED) as output");
    let g = gpios();
    g.dirset(0).write(|w| unsafe { w.bits(1 << 26) });

    let embassy_p = embassy_imxrt::init(Default::default());
    info!("Initializing pin mux");
    board_init_pins(&embassy_p);

    loop {
        info!("Toggling GPIO0_26 (Blue LED)");
        g.not(0).write(|w| unsafe { w.bits(1 << 26) });
        Timer::after_millis(2000).await;
    }
}

fn iopctl_reg() -> &'static pac::iopctl::RegisterBlock {
    unsafe { &*pac::Iopctl::ptr() }
}

fn board_init_pins(_p: &embassy_imxrt::Peripherals) {
    // This should be updated to use the peripherals once the GPIO traits are implemented
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
