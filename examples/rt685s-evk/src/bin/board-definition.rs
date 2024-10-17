#![no_std]
#![no_main]

use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_imxrt::clocks;
use embassy_imxrt::gpio::PowerManagedIO;
use embassy_imxrt::{self, gpio, iopctl};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_imxrt::init(Default::default());

    info!("Initializing GPIO");
    unsafe { gpio::init() };

    const PIN_COUNT: usize = 2;

    // temp hack for addressing specific pin
    let flex_led = 0;
    let flex_clkout = 1;

    let mut clk_out_config = clocks::ClockOutConfig::default_config();

    // TODO: this array will eventually need to contain different kinds of structs
    // that all implement the PowerManagedIO trait. The contents will NOT be of the same size
    let mut board_io: [gpio::Pmgpio; PIN_COUNT] = [
        gpio::Pmgpio::new(
            gpio::SimplePinState::Active,
            gpio::Flex::new(p.PIO0_26),
            gpio::PinMode::Input(iopctl::Pull::Down, iopctl::Polarity::ActiveHigh),
            gpio::PinMode::Output(
                iopctl::DriveMode::PushPull,
                iopctl::DriveStrength::Normal,
                iopctl::SlewRate::Standard,
            ),
        ),
        gpio::Pmgpio::new(
            gpio::SimplePinState::Idle,
            gpio::Flex::new(p.PIO1_10),
            gpio::PinMode::Input(iopctl::Pull::Down, iopctl::Polarity::ActiveHigh),
            gpio::PinMode::Func(gpio::Function::F7),
        ),
    ];

    for (i, pin) in board_io.iter_mut().enumerate() {
        if let Err(e) = pin.set_state(pin.reset) {
            error!("couldn't set pin {:#} to reset state, result: {:?}", i, e);
        }
    }

    loop {
        if let Err(e) = board_io[flex_led].set_active() {
            error!("couldn't set led to active config, result {:?}", e)
        }
        info!("toggling LED for 10 seconds");
        for _ in 1..10 {
            board_io[flex_led].pin.toggle();
            Timer::after_millis(1000).await;
        }
        info!("turning LED off");
        if let Err(e) = board_io[flex_led].set_idle() {
            error!("couldn't set led to idle config, result {:?}", e)
        }

        if let Err(e) = board_io[flex_clkout].set_active() {
            error!("couldn't set clkout to active config, result {:?}", e)
        }
        if let Err(e) = clk_out_config.enable_and_reset() {
            error!("Couldn't enable clkout clock, result {:?}", e);
        }

        if let Err(e) = clk_out_config.set_clkout_source_and_div(clocks::ClkOutSrc::Lposc, 0) {
            error!("Couldn't configure clkout, result {:?}", e);
        }
        info!("Toggle clkout at 1MHz for 10 seconds");
        Timer::after_millis(10_000).await;

        info!("reconfiguring clkout as an input");
        if let Err(e) = clk_out_config.disable() {
            error!("Couldn't disable clkout, result {:?}", e);
        }
        if let Err(e) = board_io[1].set_idle() {
            error!("couldn't set clkout to idle config, result {:?}", e)
        }
    }

    // TODO: should probably try toggling LED when it's idle to make sure that is rejected
}
