#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(xv6::test::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(dead_code)]
#![allow(unused_imports)]

extern crate alloc;

use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec;
use bootloader::entry_point;
use bootloader::BootInfo;
use core::panic::PanicInfo;
use xv6::arch;
use xv6::arch::vm;
use xv6::dmesg;
use xv6::println;
use xv6::vga::{self, Color, ColorCode};

const PANIC_INFO_COLOR: ColorCode = ColorCode::new(Color::LightRed, Color::Black);

entry_point!(kernel_main);
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    xv6::init(boot_info);

    dmesg!(
        "physical memory offset: {:#x}",
        boot_info.physical_memory_offset
    );
    for &region in boot_info.memory_map.iter() {
        dmesg!(
            "start: {:#x}, end: {:#x}, type: {:?}",
            region.range.start_addr(),
            region.range.end_addr(),
            region.region_type
        );
    }

    #[cfg(test)]
    test_main();

    let addresses = [
        // the identity-mapped vga buffer page
        0xb8000,
        // some code page
        0x201008,
        // some stack page
        0x0100_0020_1a10,
        // virtual address mapped to physical address 0
        boot_info.physical_memory_offset,
    ];

    for &address in &addresses {
        let virt = address as usize;
        let phys = unsafe { vm::virt_to_phys(virt) };
        println!("VirtualAddr({:#x}) -> PhysAddr({:x?})", virt, phys);
    }

    let mut test_writer = unsafe {
        vga::Writer::new(
            vga::ColorCode::new(Color::Black, Color::White),
            vm::map_virt_to_phys(0, 0xb8000).unwrap(),
        )
    };
    test_writer.set_pos(20, 0);
    test_writer.write_str("test from mapped vga buffer\n");

    // let l4_table = unsafe { vm::load_page_table(vm::cur_pgd_phyaddr()) };
    // for (i, entry) in l4_table.iter().enumerate() {
    //     if !entry.is_unused() {
    //         dmesg!("L4 Entry {}: {:?}", i, entry);

    //         let phys = entry.frame().unwrap().start_address().as_u64() as usize;
    //         let l3_table = unsafe { vm::load_page_table(phys) };
    //         for (i, entry) in l3_table.iter().enumerate() {
    //             if !entry.is_unused() {
    //                 dmesg!("  L3 Entry {}: {:?}", i, entry);
    //             }
    //         }
    //     }
    // }

    // unsafe {
    //     *(0xdeadbeef as *mut u8) = 42;
    // };

    let s = Box::new("str from heap!");
    println!("{} at {:p}", s, s);

    let str = String::from("and String from heap!");
    println!("{} at {:p}", str, str.as_str());

    let v = vec!["vec", "from", "heap", "!"];
    println!("{:?} at {:p}", v, v.as_slice());

    let rc = Rc::new(Box::new("and rc from heap!"));
    println!("{:?} at {:p}", rc.clone(), rc);

    // this function is the entry point, since the linker looks for a function
    // named `_start` by default
    println!("Hello, World!\ninteger: {}, float: {}", 42, 3.14);

    dmesg!("running on cpu {}", arch::getcpu());

    // panic!("Some panic message");

    arch::halt();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    xv6::with_color!(PANIC_INFO_COLOR, {
        println!("{}", info);
    });
    arch::halt();
}

#[cfg(test)]
#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    xv6::test::panic_handler(info)
}
