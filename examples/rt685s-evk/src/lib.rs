#![no_std]

use defmt_rtt as _;
use mimxrt600_fcb::FlexSPIFlashConfigurationBlock;
#[cfg(not(feature = "test-parser"))]
use panic_probe as _;
#[cfg(feature = "test-parser")]
use test_parser_macros as _;
// auto-generated version information from Cargo.toml
include!(concat!(env!("OUT_DIR"), "/biv.rs"));

#[link_section = ".otfad"]
#[used]
static OTFAD: [u8; 256] = [0; 256];

#[rustfmt::skip]
#[link_section = ".fcb"]
#[used]
static FCB: FlexSPIFlashConfigurationBlock = FlexSPIFlashConfigurationBlock::build();

#[link_section = ".keystore"]
#[used]
static KEYSTORE: [u8; 2048] = [0; 2048];
