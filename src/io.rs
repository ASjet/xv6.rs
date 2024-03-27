pub mod console;
pub mod uart;

pub struct BaseIO<T> {
    base: usize,
    data: core::marker::PhantomData<T>,
}

impl<T> BaseIO<T> {
    pub const fn new(base: usize) -> BaseIO<T> {
        BaseIO::<T> {
            base: base,
            data: core::marker::PhantomData,
        }
    }

    pub const fn offset(&self, offset: usize) -> IO<T> {
        IO::<T>::new(self.base + offset)
    }
}

pub struct ScratchIO<T> {
    base: usize,
    scratch_size: usize,
    data: core::marker::PhantomData<T>,
}

impl<T> ScratchIO<T> {
    pub const fn new(base: usize, scratch: usize) -> Self {
        ScratchIO {
            base: base,
            scratch_size: scratch,
            data: core::marker::PhantomData,
        }
    }

    pub const fn index(&self, index: usize) -> IO<T> {
        return IO::new(self.base + (self.scratch_size * index));
    }
}

pub struct IO<T>(*mut T);

impl<T> IO<T> {
    pub const fn new(addr: usize) -> Self {
        IO(addr as *mut T)
    }

    pub fn read(&self) -> T {
        unsafe { self.0.read_volatile() }
    }

    pub fn write(&self, value: T) {
        unsafe { self.0.write_volatile(value) };
    }
}
