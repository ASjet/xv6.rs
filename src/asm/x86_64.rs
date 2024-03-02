use super::QemuExitCode;
use crate::{println, vga, with_color};
use lazy_static::lazy_static;
use x86_64::instructions::port::Port;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

const ISA_DEBUG_EXIT_PORT: u16 = 0xf4;
type IsaDebugExitPort = u32;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt
    };
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    unsafe {
        let mut port = Port::new(ISA_DEBUG_EXIT_PORT);
        port.write(exit_code as IsaDebugExitPort);
    }
}

pub fn init_idt() {
    IDT.load();
}

const INTERRUPT_COLOR: vga::ColorCode = vga::ColorCode::new(vga::Color::Yellow, vga::Color::Black);
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    with_color!(INTERRUPT_COLOR, {
        println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
    });
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE_FAULT\n{:#?}", stack_frame);
}

#[test_case]
fn test_breakpoint_interrupt() {
    x86_64::instructions::interrupts::int3();
}
