//! UART driver

use core::cell::Cell;
use mimxrt685s_pac as pac;

use crate::pac::dma0;
use crate::pac::flexcomm0;
use crate::pac::flexcomm1;
use crate::pac::flexcomm2;
use crate::pac::flexcomm3;
use crate::pac::flexcomm4;
use crate::pac::usart0;

use flexcomm0::pselid::Persel as PeripheralType;

const UART_CTRL_FLAG_ENABLE_TX: u32 = 1u32 << 0;
const UART_CTRL_FLAG_ENABLE_RX: u32 = 1u32 << 1;
const UART_CTRL_FLAG_ENABLE_FLOWCTRL: u32 = 1u32 << 2;
const UART_CTRL_FLAG_ENABLE_LOOPBACK_TEST: u32 = 1u32 << 3;

const UART_STAT_FLAG_FLAG_OPEN: u32 = 1u32 << 0;
const UART_STAT_FLAG_FLAG_RX_ACTIVE: u32 = 1u32 << 1;
const UART_ACTION_RX_START: u32 = 1u32 << 0;
const UART_ACTION_RX_END: u32 = 1u32 << 1;
const UART_ACTION_NEW_RX_BYTES: u32 = 1u32 << 2;
//use crate::pac::dma1;
//use crate::{pac};
//use crate::{pac, Peripheral};

/// A GPIO port with up to 32 pins.
//#[derive(Debug, Eq, PartialEq)]
pub enum FlexComm {
    FlexcommHostUart = 0,
    FlexcommDebugUart      = 1,
    FlexcommVfgI2c         = 2,
    FlexcommTdmUart        = 3,

    /// Interface to KIP, Blade and Touchpad
    FlexcommTouchpadUart   = 4, 
    FlexcommSurflinkUart   = 5,
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
            0 => FlexcommHostUart,
            1 => FlexcommDebugUart,
            2 => FlexcommVfgI2c,
            3 => FlexcommTdmUart,
            4 => FlexcommTouchpadUart,
            5 => FlexcommSurflinkUart,
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
            FlexComm ::FlexcommHostUart => 0,
            FlexComm::FlexcommDebugUart => 1,
            FlexComm::FlexcommVfgI2c => 2,
            FlexComm::FlexcommTdmUart => 3,
            FlexComm::FlexcommTouchpadUart => 4,
            FlexComm::FlexcommSurflinkUart => 5,
            //FlexComm::eFLEXCOMM_TEMP_SENSOR_I2C => 6,
            //FlexComm::eFLEXCOMM_IMU_SENSOR_I2C => 7,
            //FlexComm::eFLEXCOMM_UEFI_TP_SPI => 8,
           // FlexComm::eFLEXCOMM_FUEL_GAUGE_I2C => 9,
        }
    }
}


impl FlexComm{
    fn regs(&self)->&'static pac::flexcomm0::RegisterBlock{
        use FlexComm::*;

        match self{
            FlexcommHostUart => unsafe { &*(pac::Flexcomm0::ptr() as *const pac::flexcomm0::RegisterBlock)},
            FlexcommDebugUart => unsafe { &*(pac::Flexcomm1::ptr() as *const pac::flexcomm0::RegisterBlock)},
            FlexcommVfgI2c => unsafe { &*(pac::Flexcomm2::ptr() as *const pac::flexcomm0::RegisterBlock)},
            FlexcommTdmUart => unsafe { &*(pac::Flexcomm3::ptr() as *const pac::flexcomm0::RegisterBlock)},
            FlexcommTouchpadUart => unsafe { &*(pac::Flexcomm4::ptr() as *const pac::flexcomm0::RegisterBlock)},
            FlexcommSurflinkUart => unsafe { &*(pac::Flexcomm5::ptr() as *const pac::flexcomm0::RegisterBlock)},
        }
    }
    
    pub fn set_peripheral(&self, peripheral:PeripheralType, lock: bool) -> bool {
        if peripheral != PeripheralType::NoPeriphSelected{
            if peripheral == PeripheralType::I2c && self.regs().pselid().read().i2cpresent().is_not_present(){
                return false;
                //todo add a panic here
            } else if peripheral == PeripheralType::Spi && self.regs().pselid().read().spipresent().is_not_present(){
                return false;
                //todo add a panic here
            } else if peripheral == PeripheralType::Usart && self.regs().pselid().read().usartpresent().is_not_present(){
                return false;
                //todo add a panic here
            }else if (peripheral == PeripheralType::I2sReceive ||  peripheral == PeripheralType::I2sTransmit) && self.regs().pselid().read().i2spresent().is_not_present(){
                return false;
                //todo add a panic here
            }
        }
        
        if self.regs().pselid().read().lock().is_locked() && self.regs().pselid().read().persel().ne(&peripheral){
            // Flexcomm is locked to different peripheral type than expected
            return false;
        }

        //self.regs().pselid().read().persel().into().
        //self.regs().pselid().read().persel().ne(&peripheral);
        
        if lock {
            self.regs().pselid().write(|w| w.lock().locked());
        } else {
            self.regs().pselid().write(|w| w.lock().unlocked());
        }
        unsafe {self.regs().pselid().write(|w| w.persel().bits(peripheral as u8))};

        return true;
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum UartParity{
    None,
    Even,
    Odd,
}

#[derive(Debug, PartialEq, Eq)]
pub enum UartBitsPerCharacter{
    Seven,
    Eight,
}

#[derive(Debug, PartialEq, Eq)]
pub enum UartStopBits{
    One,
    Two,
}

pub struct Uart{
    flexcomm: FlexComm,
    baudrate: u32,
    bitsPerCharacter: UartBitsPerCharacter,
    parity: UartParity,
    stopBits: UartStopBits,
    controlFlags: u32,
    statusFlags: Cell<u32>,
    interruptPri:u16
    //dma: dma0::RegisterBlock,
    //tx_dma: dma0::RegisterBlock,
    //rx_dma: dma0::RegisterBlock,

}

impl Uart{

    fn reg(&self)->&'static pac::usart0::RegisterBlock{
        unsafe { &*(pac::Usart0::ptr() as *const pac::usart0::RegisterBlock)}
    }

    pub fn new(flexcomm: FlexComm, baudrate: u32, bitsPerCharacter: UartBitsPerCharacter, parity: UartParity, stopBits: UartStopBits, controlFlags:u32,_statusFlags:u32,interruptPri:u16 )->Self{
        Uart{
            flexcomm,
            baudrate,
            bitsPerCharacter,
            parity,
            stopBits,
            controlFlags,
            statusFlags:Cell::new(_statusFlags),
            interruptPri
        }
    }

    pub fn init(&self){
        //self.flexcomm.set_peripheral(PeripheralType::Usart, true);
        //self.flexcomm.regs().ctrl().write(|w| w.enable().enabled());
        //self.flexcomm.regs().ctrl().write(|w| w.enable().enabled());
        //self.flexcomm.regs().ctrl().write(|w| w.enable().enabled());
        //self.flexcomm.regs().ctrl().write(|w| w.enable().enabled());
        //self.flexcomm.regs().ctrl().write(|w| w.enable().enabled());
        //self.flexcomm.regs().ctrl().write(|w| w.enable().enabled());

        // Todo: Add code to abort DMA transfer of Tx and Rx
        // todo: Add code to save the interrupt ID, prio and target ID

        if self.bitsPerCharacter == UartBitsPerCharacter::Seven{
            self.reg().cfg().write(|w| w.datalen().bit_7());
        }
        else{
            //self.bitsPerCharacter == UartBitsPerCharacter::Eight
            self.reg().cfg().write(|w| w.datalen().bit_8());
        }

        if self.parity == UartParity::None{
            self.reg().cfg().write(|w| w.paritysel().no_parity());
        } else if self.parity == UartParity::Even{
            self.reg().cfg().write(|w| w.paritysel().even_parity());
        } else if self.parity == UartParity::Odd{
            self.reg().cfg().write(|w| w.paritysel().odd_parity());
        }

        if self.stopBits == UartStopBits::One{
            self.reg().cfg().write(|w| w.stoplen().bit_1());
        } else {
            //UartStopBits::Two
            self.reg().cfg().write(|w| w.stoplen().bits_2());
        }

        // TODO : Set Nvic priority : NVIC_SetPriority((IRQn_Type)pTgtPrivCtxt->interruptId, pTgtPrivCtxt->interruptPri);



    }

    pub fn uart_open(&self, clkFreq_Hz :u32){
        //self.flexcomm.set_peripheral(PeripheralType::Usart, true);
        //self.flexcomm.regs().ctrl().write(|w| w.enable().enabled());
        self.uart_calculate_baud_rate(clkFreq_Hz);

        // TODO: Add code to abort the DMA's on Rx and Tx lines
        // TODO: Add code to enable Flexcomm Uart DMA.

        if self.controlFlags & UART_CTRL_FLAG_ENABLE_TX != 0{
            self.reg().fifocfg().write(|w| w.enabletx().set_bit());
            self.reg().fifocfg().write(|w| w.emptytx().set_bit());
        }

        if self.controlFlags & UART_CTRL_FLAG_ENABLE_RX != 0{
            self.reg().fifocfg().write(|w| w.emptyrx().set_bit());
            self.reg().fifocfg().write(|w| w.enablerx().set_bit());
            self.reg().fifocfg().write(|w| w.dmarx().set_bit());

            //TODO: Start the DMA transfer
        }

        //Clear the FIFO error
        self.reg().fifostat().write(|w| w.txerr().set_bit());
        self.reg().fifostat().write(|w| w.rxerr().set_bit());

        self.reg().intenset().write(|w| w.starten().set_bit());
        self.reg().intenset().write(|w| w.deltarxbrken().set_bit());
        self.reg().intenset().write(|w| w.framerren().set_bit());
        self.reg().intenset().write(|w| w.parityerren().set_bit());

        // TODO: Enable NVIC interrupt : NVIC_EnableIRQ((IRQn_Type)pTgtPrivCtxt->interruptId);

        self.statusFlags.set(self.statusFlags.get() | UART_STAT_FLAG_FLAG_OPEN);

    }

    pub fn uart_close(&self){
        //self.flexcomm.regs().ctrl().write(|w| w.enable().disabled());
    }

    pub fn uart_wakeup(&self){
        //self.flexcomm.regs().ctrl().write(|w| w.enable().enabled());
    }

    pub fn uart_sleep(&self){
       // self.flexcomm.regs().ctrl().write(|w| w.enable().disabled());
    }

    fn uart_calculate_baud_rate(&self,_clkFreq_Hz :u32){
        let mut best_osrval:u8 = 0xf;
        let mut best_brgval:u16 = 0xffff;
       // TODO: add code to update values of OSR and BRG reg.

       best_osrval = 0xf;
       best_brgval = 0xffff;
       let _baudrate = self.baudrate;

       unsafe{
           self.reg().osr().write(|w| w.osrval().bits(best_osrval));
           self.reg().brg().write(|w| w.brgval().bits(best_brgval));
       }
    }
}