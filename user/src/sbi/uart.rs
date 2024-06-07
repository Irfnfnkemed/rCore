use core::fmt::{self, Write};
use core::sync::atomic::{AtomicPtr, Ordering};
use lazy_static::lazy_static;
use crate::sync::safe_cell_single::SafeCellSingle;

const UART_BASE: usize = 0x10000000;

macro_rules! wait_for {
    ($cond:expr) => {
        while !$cond {
            core::hint::spin_loop();
        }
    };
}

pub struct UartRegs {
    // ports
    regs: [AtomicPtr<u8>; 8],
}

impl Write for UartRegs {
    fn write_str(&mut self, str: &str) -> fmt::Result {
        for byte in str.bytes() {
            self.send(byte);
        }
        Ok(())
    }
}

impl UartRegs {
    // read/write port names
    const RBR: usize = 0;
    const THR: usize = 0;
    const DLL: usize = 0;
    const IER: usize = 1;
    const DLM: usize = 1;
    const IIR: usize = 2;
    const FCR: usize = 2;
    const LCR: usize = 3;
    const MCR: usize = 4;
    const LSR: usize = 5;
    const MSR: usize = 6;
    const SCR: usize = 7;

    // control signals
    const DISABLE_INTERRUPTS: u8 = 0x00;
    const LCR_DLAB_LATCH: u8 = 0x80;
    const LSB_BAUD_RATE: u8 = 0x03;
    const MSB_BAUD_RATE: u8 = 0x00;
    const LCR_EIGHT_BITS: u8 = 0b11;
    const FCR_FIFO_ENABLE: u8 = 0x01;
    const FCR_FIFO_CLEAR: u8 = 0x06;
    const IER_TX_ENABLE: u8 = 0x02;
    const IER_RX_ENABLE: u8 = 0x01;
    const OUTPUT_EMPTY: u8 = 0x20;
    const INPUT_AVAILABLE: u8 = 0x01;

    // other constant
    const BS: u8 = 0x08;
    const DEL: u8 = 0xff;

    pub fn new(base: usize) -> Self {
        let base_ptr = base as *mut u8;
        let mut regs: [AtomicPtr<u8>; 8] = Default::default();
        for i in 0..regs.len() {
            regs[i] = AtomicPtr::new(unsafe { base_ptr.offset(i as isize) });
        }
        let uart = UartRegs { regs };
        uart.init(); // Initialize the UART
        uart
    }

    pub fn init(&self) {
        self.write_reg(Self::IER, Self::DISABLE_INTERRUPTS);// disable interrupts
        self.write_reg(Self::LCR, Self::LCR_DLAB_LATCH);// special mode to set baud rate
        self.write_reg(Self::DLL, Self::LSB_BAUD_RATE);// LSB for baud rate of 38.4K
        self.write_reg(Self::DLM, Self::MSB_BAUD_RATE);// MSB for baud rate of 38.4K
        self.write_reg(Self::LCR, Self::LCR_EIGHT_BITS);// leave set-baud mode, and set word length to 8 bits, no parity
        self.write_reg(Self::FCR, Self::FCR_FIFO_ENABLE | Self::FCR_FIFO_CLEAR);// reset and enable FIFOs
        self.write_reg(Self::IER, Self::IER_TX_ENABLE | Self::IER_RX_ENABLE); // enable transmit and receive interrupts
    }

    fn read_reg(&self, id: usize) -> u8 {
        let ptr = self.regs[id].load(Ordering::Acquire);
        unsafe { *ptr }
    }

     fn write_reg(&self, id: usize, data: u8) {
        let ptr = self.regs[id].load(Ordering::Acquire);
        unsafe { ptr.write(data) };
    }

    // Send a byte on the serial port
    pub fn send(&self, data: u8) {
        match data {
            Self::BS | Self::DEL => {
                wait_for!((self.read_reg(Self::LSR) & Self::OUTPUT_EMPTY) != 0);
                self.write_reg(Self::THR, Self::BS);
                wait_for!((self.read_reg(Self::LSR) & Self::OUTPUT_EMPTY) != 0);
                self.write_reg(Self::THR, b' ');
                wait_for!((self.read_reg(Self::LSR) & Self::OUTPUT_EMPTY) != 0);
                self.write_reg(Self::THR, Self::BS);
            }
            _ => {
                wait_for!((self.read_reg(Self::LSR) & Self::OUTPUT_EMPTY) != 0);
                self.write_reg(Self::THR, data);
            }
        }
    }

    pub unsafe fn recv(&self) -> u8 {
        wait_for!((self.read_reg(Self::LSR) & Self::OUTPUT_EMPTY) != 0);
        self.read_reg(Self::RBR)
    }
}

lazy_static! {
   pub static ref UART: SafeCellSingle<UartRegs> = unsafe { SafeCellSingle::new(UartRegs::new(UART_BASE))};
}


