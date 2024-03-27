use super::{BaseIO, IO};
use crate::spinlock::Mutex;
use core::fmt::{Arguments, Write};

/// low-level driver routines for 16550a UART.
///
/// the UART control registers are memory-mapped
/// at address UART0. this macro returns the
/// address of one of the registers.
use crate::arch::def::UART0;

static UART: Mutex<SyncUartWriter> = Mutex::new(SyncUartWriter, "uart0");

// the UART control registers.
// some have different meanings for
// read vs write.
// see http://byterunner.com/16550.html

const UART0_BASE: BaseIO<u8> = BaseIO::new(UART0 as usize);
const RHR: IO<u8> = UART0_BASE.offset(0); // receive holding register (for input bytes)
const THR: IO<u8> = UART0_BASE.offset(0); // transmit holding register (for output bytes)
const IER: IO<u8> = UART0_BASE.offset(1); // interrupt enable register
const IER_RX_ENABLE: u8 = 1 << 0;
const IER_TX_ENABLE: u8 = 1 << 1;
const FCR: IO<u8> = UART0_BASE.offset(2); // FIFO control register
const FCR_FIFO_ENABLE: u8 = 1 << 0;
const FCR_FIFO_CLEAR: u8 = 3 << 1; // clear the content of the two FIFOs
const ISR: IO<u8> = UART0_BASE.offset(2); // interrupt status register
const LCR: IO<u8> = UART0_BASE.offset(3); // line control register
const LCR_EIGHT_BITS: u8 = 3 << 0;
const LCR_BAUD_LATCH: u8 = 1 << 7; // special mode to set baud rate
const LSR: IO<u8> = UART0_BASE.offset(5); // line status register
const LSR_RX_READY: u8 = 1 << 0; // input is waiting to be read from RHR
const LSR_TX_IDLE: u8 = 1 << 5; // THR can accept another character to send

pub fn init() {
    // disable interrupts.
    IER.write(0x00);

    // special mode to set baud rate.
    LCR.write(LCR_BAUD_LATCH);

    // LSB for baud rate of 38.4K.
    THR.write(0x03);

    // MSB for baud rate of 38.4K.
    IER.write(0x00);

    // leave set-baud mode,
    // and set word length to 8 bits, no parity.
    LCR.write(LCR_EIGHT_BITS);

    // reset and enable FIFOs.
    FCR.write(FCR_FIFO_ENABLE | FCR_FIFO_CLEAR);

    // enable transmit and receive interrupts.
    IER.write(IER_TX_ENABLE | IER_RX_ENABLE);
}

struct SyncUartWriter;

impl SyncUartWriter {
    fn write_byte(&self, c: u8) {
        // let _guard = unsafe { CPU::this_mut().push_off() };
        while (LSR.read() & LSR_TX_IDLE) == 0 {}
        THR.write(c);
    }
}

impl core::fmt::Write for SyncUartWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        s.as_bytes().iter().for_each(|c| self.write_byte(*c));
        Ok(())
    }
}

pub fn uart_print_sync(args: Arguments) {
    UART.lock().write_fmt(args).unwrap();
}

pub fn uart_print(args: Arguments) {
    SyncUartWriter.write_fmt(args).unwrap();
}
