use crate::aarch64;
use crate::driver;

extern "C" {
    static mut __stack_el1_end: u64;
    static mut __stack_el1_start: u64;
    fn el3_to_el1_asm();
}

pub fn el3_to_el1() {
    driver::uart::puts("el3_to_el1\n");
    unsafe{el3_to_el1_asm();}
}

pub fn el3_to_el1_org() {
    driver::uart::puts("el3_to_el1\n");

    let end = unsafe { &mut __stack_el1_end as *mut u64 as usize };
    let start = unsafe { &mut __stack_el1_start as *mut u64 as usize };

    let nc = (start - end) >> 21; // div by 2MiB (32 pages), #CPU
    let size = (start - end) / nc;

    let aff = aarch64::cpu::get_affinity_lv0();
//    let addr = start - size * aff as usize + aarch64::mmu::EL1_ADDR_OFFSET;
    let addr = start;

    unsafe {
        llvm_asm!(
            "mrs x0, hcr_el2
             orr x0, x0, #(1 << 31) // AArch64
             orr x0, x0, #(1 << 1)  // SWIO hardwired on Pi3
             msr hcr_el2, x0

             // enable CNTP for EL1
             mrs x0, cnthctl_el2
             orr x0, x0, #3
             msr cnthctl_el2, x0
             msr cntvoff_el2, xzr

             mov x0, $0
             msr sp_el1, x0    // set stack pointer
             mov x0, #0b101    // EL1h
             msr spsr_el3, x0
//             adr x0, el1_entry // entry point
             adr x0, led_blink
             msr elr_el3, x0
             eret"
            :
            : "r"(addr)
            : "x0"
        );
    }
    driver::uart::puts("el3_to_el1 done\n");
}
