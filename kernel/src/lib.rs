#![feature(core_intrinsics)]
#![feature(lang_items)]
#![feature(start)]
#![feature(llvm_asm)]
#![feature(alloc_error_handler)]
#![no_std]
#![allow(dead_code)]

mod parser;
mod boot;
mod driver;
mod aarch64;
mod el0;
mod el1;
//mod el2;
mod el3;
mod slab;
mod pager;
mod semantics;

#[macro_use]
extern crate alloc;

use core::panic::PanicInfo;

extern {
    fn get_device_start() -> u64;
    fn get_device_end() -> u64;
}

extern {
   fn mmu_init();
}

#[no_mangle]
fn func() {
    ()
}

#[no_mangle]
pub fn entry() -> ! {
    /*
    let ctx = driver::init();
    driver::uart::puts("call c function\n");
    let device_start;
    let device_end;
    unsafe{ device_start = get_device_start();}
    unsafe{ device_end   = get_device_end();}
    driver::uart::puts("result = ");
    driver::uart::hex(device_start);
    driver::uart::puts(" , ");
    driver::uart::hex(device_end);
    driver::uart::puts("\n");
*/
//    aarch64::mmu::init();
    unsafe{mmu_init();};
//    boot::run();
/*
    match aarch64::el::get_current_el() {
        3 => { el3::el3_to_el1(); }
        _ => {
            driver::uart::puts("Error: execution level is not EL3\n");
        }
    }
*/
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
