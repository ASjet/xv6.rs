core::arch::global_asm!(
    "
# Context switch
#
#   void switch(struct context *old, struct context *new);
#   switch(save: *mut Context, load: *const Context);
#
# Save current registers in old. Load from new.

.globl switch
switch:
        sd ra, 0(a0)
        sd sp, 8(a0)
        sd s0, 16(a0)
        sd s1, 24(a0)
        sd s2, 32(a0)
        sd s3, 40(a0)
        sd s4, 48(a0)
        sd s5, 56(a0)
        sd s6, 64(a0)
        sd s7, 72(a0)
        sd s8, 80(a0)
        sd s9, 88(a0)
        sd s10, 96(a0)
        sd s11, 104(a0)

        ld ra, 0(a1)
        ld sp, 8(a1)
        ld s0, 16(a1)
        ld s1, 24(a1)
        ld s2, 32(a1)
        ld s3, 40(a1)
        ld s4, 48(a1)
        ld s5, 56(a1)
        ld s6, 64(a1)
        ld s7, 72(a1)
        ld s8, 80(a1)
        ld s9, 88(a1)
        ld s10, 96(a1)
        ld s11, 104(a1)

        ret
"
);

type Register = usize;

/// Saved registers for kernel context switches. Internal mutable.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Context {
    ra: Register,
    sp: Register,

    // callee-saved
    s0: Register,
    s1: Register,
    s2: Register,
    s3: Register,
    s4: Register,
    s5: Register,
    s6: Register,
    s7: Register,
    s8: Register,
    s9: Register,
    s10: Register,
    s11: Register,
}

impl Context {
    pub const fn new() -> Context {
        return Context {
            ra: 0,
            sp: 0,
            s0: 0,
            s1: 0,
            s2: 0,
            s3: 0,
            s4: 0,
            s5: 0,
            s6: 0,
            s7: 0,
            s8: 0,
            s9: 0,
            s10: 0,
            s11: 0,
        };
    }

    pub fn setup(&mut self, ra: Register, sp: Register) {
        self.ra = ra;
        self.sp = sp;
        self.s0 = 0;
        self.s1 = 0;
        self.s2 = 0;
        self.s3 = 0;
        self.s4 = 0;
        self.s5 = 0;
        self.s6 = 0;
        self.s7 = 0;
        self.s8 = 0;
        self.s9 = 0;
        self.s10 = 0;
        self.s11 = 0;
    }
}
