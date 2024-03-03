use x86_64::instructions::port::Port;

const ISA_DEBUG_EXIT_PORT: u16 = 0xf4;
const SCAN_CODE_PORT: u16 = 0x60;
type IsaDebugExitPort = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failure = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    unsafe {
        let mut port = Port::new(ISA_DEBUG_EXIT_PORT);
        port.write(exit_code as IsaDebugExitPort);
    }
}

pub fn scan_code() -> u8 {
    unsafe { Port::new(SCAN_CODE_PORT).read() }
}
