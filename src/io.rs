pub mod console;
pub mod uart;

pub struct BaseIO<T>(*mut T);

impl<T> BaseIO<T> {
    pub const fn new(base: usize) -> BaseIO<T> {
        return BaseIO(base as *mut T);
    }

    pub const fn offset(&self, offset: usize) -> IO<T> {
        return IO::new(unsafe { self.0.offset(offset as isize) });
    }
}

pub struct IO<T>(*mut T);

impl<T> IO<T> {
    pub const fn new(addr: *mut T) -> Self {
        IO(addr)
    }

    pub fn read(&self) -> T {
        unsafe { self.0.read_volatile() }
    }

    pub fn write(&self, value: T) {
        unsafe { self.0.write_volatile(value) };
    }
}
