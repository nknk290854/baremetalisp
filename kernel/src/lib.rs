#![feature(core_intrinsics)]
#![feature(lang_items)]
#![feature(start)]
#![feature(asm)]
#![feature(alloc_error_handler)]
#![no_std]
#![allow(dead_code)]

mod parser;
mod boot;
mod driver;
mod aarch64;
mod el0;
mod el1;
mod el2;
mod el3;
mod slab;
mod pager;
mod semantics;

#[macro_use]
extern crate alloc;

use core::panic::PanicInfo;

#[no_mangle]
fn func() {
    ()
}

#[no_mangle]
pub fn entry() -> ! {
    let ctx = driver::init();
//    aarch64::mmu::init();

    boot::run();

//    match aarch64::el::get_current_el() {
//        3 => { el3::el3_to_el1(); }
//        2 => {
//            driver::uart::puts("Warning: execution level is not EL3\n");
//            el2::el2_to_el1();
//        }
//        _ => {
//            driver::uart::puts("Error: execution level is not EL3\n");
//        }
//    }

    loop {}
}

#[lang = "eh_personality"]
#[no_mangle]
extern fn eh_personality() {}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    driver::uart::puts("kernel panic!\n");
    if let Some(location) = info.location() {
        driver::uart::puts(location.file());
        driver::uart::puts(":");
        driver::uart::decimal(location.line() as u64);
        driver::uart::puts("\n");
    }

    if let Some(s) = info.payload().downcast_ref::<&str>() {
        driver::uart::puts(s);
        driver::uart::puts("\n");
    }

    loop {}
}

#[no_mangle]
pub fn abort() -> ! {
    loop {}
}
