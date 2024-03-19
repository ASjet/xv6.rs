use core::fmt::{Arguments, Write};

const DBCN_EID: u64 = 0x4442434E;

pub struct DebugConsole;

impl core::fmt::Write for DebugConsole {
    fn write_str(&mut self, mut s: &str) -> core::fmt::Result {
        while !s.is_empty() {
            match unsafe { sbi_debug_console_write(s.as_bytes()) } {
                Ok(n) => s = &s[n..],
                Err(_) => return Err(core::fmt::Error),
            }
        }

        Ok(())
    }
}

pub fn debug_print(args: Arguments) {
    write!(DebugConsole, "{}", args).unwrap();
}

pub unsafe fn sbi_debug_console_write(s: &[u8]) -> Result<usize, u64> {
    let rc: u64;
    let result: usize;
    core::arch::asm!(
        "ecall",
        in("a7") DBCN_EID,
        in("a6") 0, // 0 for write
        in("a0") s.len(),
        in("a1") s.as_ptr(),
        in("a2") 0,
        lateout("a0") rc,
        lateout("a1") result,
    );
    if rc == 0 {
        Ok(result)
    } else {
        Err(rc)
    }
}
