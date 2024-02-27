use super::QemuExitCode;
use x86_64::instructions::port::Port;

const ISA_DEBUG_EXIT_PORT: u16 = 0xf4;
type IsaDebugExitPort = u32;

pub fn exit_qemu(exit_code: QemuExitCode) {
    unsafe {
        let mut port = Port::new(ISA_DEBUG_EXIT_PORT);
        port.write(exit_code as IsaDebugExitPort);
    }
}
