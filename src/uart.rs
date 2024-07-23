//! UART driver
use mimxrt685s_pac as pac;

use crate::pac::dma0;
use crate::pac::flexcomm0;
use crate::pac::flexcomm1;
use crate::pac::flexcomm2;
use crate::pac::flexcomm3;
use crate::pac::flexcomm4;
use crate::pac::usart0;

//use crate::pac::dma1;
//use crate::{pac};
//use crate::{pac, Peripheral};

/// A GPIO port with up to 32 pins.
//#[derive(Debug, Eq, PartialEq)]
pub enum FlexComm {
    eFLEXCOMM_HOST_UART = 0,
    eFLEXCOMM_DEBUG_UART      = 1,
    eFLEXCOMM_VFG_I2C         = 2,
    eFLEXCOMM_TDM_UART        = 3,

    /// Interface to KIP, Blade and Touchpad
    eFLEXCOMM_TOUCHPAD_UART   = 4, 
    eFLEXCOMM_SURFLINK_UART   = 5,
   // eFLEXCOMM_TEMP_SENSOR_I2C = 6,

     // eFLEXCOMM_IMU_SENSOR_I2C might be redundant. Need investigation
    //eFLEXCOMM_IMU_SENSOR_I2C  = 7, 
    //eFLEXCOMM_UEFI_TP_SPI    = 8,
    //eFLEXCOMM_FUEL_GAUGE_I2C = 9,
}

impl From<u8> for FlexComm {
    fn from(value:u8)->Self{
        use FlexComm::*;

        match value {
            0 => eFLEXCOMM_HOST_UART,
            1 => eFLEXCOMM_DEBUG_UART,
            2 => eFLEXCOMM_VFG_I2C,
            3 => eFLEXCOMM_TDM_UART,
            4 => eFLEXCOMM_TOUCHPAD_UART,
            5 => eFLEXCOMM_SURFLINK_UART,
           // 6 => eFLEXCOMM_TEMP_SENSOR_I2C,
           // 7 => eFLEXCOMM_IMU_SENSOR_I2C,
           // 8 => eFLEXCOMM_UEFI_TP_SPI,
          //  9 => eFLEXCOMM_FUEL_GAUGE_I2C,
            6..=u8::MAX => panic!("Invalid FlexComm Selection!")
        }
    }
}

impl From<FlexComm> for u8 {
    fn from(value: FlexComm) -> Self {
        match value {
            FlexComm ::eFLEXCOMM_HOST_UART => 0,
            FlexComm::eFLEXCOMM_DEBUG_UART => 1,
            FlexComm::eFLEXCOMM_VFG_I2C => 2,
            FlexComm::eFLEXCOMM_TDM_UART => 3,
            FlexComm::eFLEXCOMM_TOUCHPAD_UART => 4,
            FlexComm::eFLEXCOMM_SURFLINK_UART => 5,
            //FlexComm::eFLEXCOMM_TEMP_SENSOR_I2C => 6,
            //FlexComm::eFLEXCOMM_IMU_SENSOR_I2C => 7,
            //FlexComm::eFLEXCOMM_UEFI_TP_SPI => 8,
           // FlexComm::eFLEXCOMM_FUEL_GAUGE_I2C => 9,
        }
    }
}


impl FlexComm{
   // fn regs(&self)->&'static pac::usart0::RegisterBlock{
    fn regs(&self)->&'static pac::flexcomm0::RegisterBlock{
        use FlexComm::*;

        match self{
            eFLEXCOMM_HOST_UART => unsafe { &*(pac::Flexcomm0::ptr() as *const pac::flexcomm0::RegisterBlock)},
            eFLEXCOMM_DEBUG_UART => unsafe { &*(pac::Flexcomm1::ptr() as *const pac::flexcomm0::RegisterBlock)},
            eFLEXCOMM_VFG_I2C => unsafe { &*(pac::Flexcomm2::ptr() as *const pac::flexcomm0::RegisterBlock)},
            eFLEXCOMM_TDM_UART => unsafe { &*(pac::Flexcomm3::ptr() as *const pac::flexcomm0::RegisterBlock)},
            eFLEXCOMM_TOUCHPAD_UART => unsafe { &*(pac::Flexcomm4::ptr() as *const pac::flexcomm0::RegisterBlock)},
            eFLEXCOMM_SURFLINK_UART => unsafe { &*(pac::Flexcomm5::ptr() as *const pac::flexcomm0::RegisterBlock)},
        }
    }

    fn set_peripheral(&self, peripheral:u8, lock: bool) -> bool {
// todo: Ask if I can change peripheral from u8 to "enum Persel"
        if (peripheral != 0x0){
            if (peripheral == 0x3 && self.regs().pselid().read().i2cpresent().is_not_present()){
                return false;
                //todo add a panic here
            } else if (peripheral == 0x2 && self.regs().pselid().read().spipresent().is_not_present()){
                return false;
                //todo add a panic here
            } else if (peripheral == 0x1 && self.regs().pselid().read().usartpresent().is_not_present()){
                return false;
                //todo add a panic here
            }else if ((peripheral == 0x4 ||  peripheral == 0x5) && self.regs().pselid().read().i2spresent().is_not_present()){
                return false;
                //todo add a panic here
            }
        }
        
        if self.regs().pselid().read().lock().is_locked() && self.regs().pselid().read().persel().bits() != peripheral{
            // Flexcomm is locked to different peripheral type than expected
            return false;
        } else {
            // Handle the case when the condition is not met.
            // Add your logic here.
            return true;
        }
        
        if (lock ){
            self.regs().pselid().write(|w| w.lock().locked());
        } else {
            self.regs().pselid().write(|w| w.lock().unlocked());
        }
        unsafe {self.regs().pselid().write(|w| w.persel().bits(peripheral))};

        return true;
    }
}

pub struct Uart{
    baudrate: u32,
    flexcomm: FlexComm,
    //dma: dma0::RegisterBlock,
    //tx_dma: dma0::RegisterBlock,
    //rx_dma: dma0::RegisterBlock,

}