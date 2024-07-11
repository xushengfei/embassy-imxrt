#![no_std]

// List of the hal modules for export
pub mod adc;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}
