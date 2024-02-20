const VGA_BUFFER_ADDR: isize = 0xB8000;
#[allow(dead_code)]
const VGA_PIXEL_WIDTH: isize = 2;
#[allow(dead_code)]
const VGA_WIDTH: isize = 80;
#[allow(dead_code)]
const VGA_HEIGHT: isize = 25;

pub fn write_offset_attr(offset: isize, byte: u8, attr: u8) {
    let vga_buffer = VGA_BUFFER_ADDR as *mut u8;
    unsafe {
        *vga_buffer.offset(offset) = byte;
        *vga_buffer.offset(offset+1) = attr;
    }
}

#[allow(dead_code)]
pub fn write_offset(offset: isize, byte: u8) {
    write_offset_attr(offset, byte, 0xb);
}

#[allow(dead_code)]
pub fn write_pos_attr(x: isize, y: isize, byte: u8, attr: u8) {
    write_offset_attr(pos_to_offset(x, y), byte, attr);
}

#[allow(dead_code)]
pub fn write_pos(x: isize, y: isize, byte: u8) {
    write_offset(pos_to_offset(x, y), byte);
}

#[allow(dead_code)]
fn pos_to_offset(x: isize, y: isize) -> isize {
    VGA_PIXEL_WIDTH * (y * VGA_WIDTH + x)
}
