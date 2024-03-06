use super::{gdt, PortIndex};
use crate::{print, println, vga, with_color};
use core::char;
use core::ops::Div;
use int_enum::IntEnum;
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;
use x86_64::instructions;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;
pub static mut TIMER_TICKS: u64 = 0;

pub fn ticks() -> f32 {
    unsafe { TIMER_TICKS as f32 / 10.0 }
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer as usize].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard as usize].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[inline]
pub fn without_interrupts<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    instructions::interrupts::without_interrupts(f)
}

#[derive(Debug, Clone, Copy, IntEnum)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard = PIC_1_OFFSET + 1,
}

impl InterruptIndex {
    fn eoi(self) {
        unsafe {
            PICS.lock().notify_end_of_interrupt(u8::from(self));
        }
    }
}

pub fn init_idt() {
    IDT.load();
}

pub fn init_pic() {
    unsafe {
        PICS.lock().initialize();
    }
    x86_64::instructions::interrupts::enable();
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

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        TIMER_TICKS += 1;
    }
    InterruptIndex::Timer.eoi();
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
            Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore)
        );
    }

    let mut keyboard = KEYBOARD.lock();

    if let Ok(Some(key_event)) = keyboard.add_byte(PortIndex::ScanCode.read()) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(ch) => print!("{}", ch),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    InterruptIndex::Keyboard.eoi();
}

#[test_case]
fn test_breakpoint_interrupt() {
    x86_64::instructions::interrupts::int3();
}
