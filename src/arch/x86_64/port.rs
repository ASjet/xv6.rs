use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;
use x86_64::instructions::port::Port;

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = Mutex::new(SerialPortIndex::Serial1.new());
}

#[repr(u16)]
pub enum PortIndex {
    ISADebugExit = 0xf4,
    ScanCode = 0x60,
}

impl PortIndex {
    #[inline]
    pub fn read(self) -> u8 {
        unsafe { Port::new(self as u16).read() }
    }

    #[inline]
    pub fn write(self, value: u32) {
        unsafe { Port::new(self as u16).write(value) }
    }
}

#[repr(u16)]
pub enum SerialPortIndex {
    Serial1 = 0x3f8,
}

impl SerialPortIndex {
    pub fn new(self) -> SerialPort {
        let mut serial_port = unsafe { SerialPort::new(self as u16) };
        serial_port.init();
        serial_port
    }
}
