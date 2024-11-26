#![no_std]

use defmt_rtt as _;
use mimxrt600_fcb::FlexSPIFlashConfigurationBlock;

#[link_section = ".otfad"]
#[used]
static OTFAD: [u8; 256] = [0; 256];

#[rustfmt::skip]
#[link_section = ".fcb"]
#[used]
static FCB: FlexSPIFlashConfigurationBlock = FlexSPIFlashConfigurationBlock::build();

#[link_section = ".biv"]
#[used]
static BOOT_IMAGE_VERSION: u32 = 0x01000000;

#[link_section = ".keystore"]
#[used]
static KEYSTORE: [u8; 2048] = [0; 2048];

#[panic_handler]
fn panic(p: &core::panic::PanicInfo) -> ! {
    defmt::error!(
        "TEST-FAIL: {} failed on line {} with error {}",
        p.location().unwrap().file(),
        p.location().unwrap().line(),
        p.message().as_str()
    );
    cortex_m::asm::bkpt();
    loop {}
}
