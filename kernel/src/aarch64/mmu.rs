use core::slice;

use super::el;
use crate::driver;

pub const EL1_ADDR_OFFSET: usize = 0x3FFFFF << 42;

extern "C" {
    static mut __data_start: u64;
    static mut __data_end: u64;
    static mut __bss_start: u64;
    static mut __bss_end: u64;

    static mut __no_cache: u64;

    static mut __stack_start: u64;
    static mut __stack_firm_end: u64;
    static mut __stack_firm_start: u64;
    static mut __stack_el1_end: u64;
    static mut __stack_el1_start: u64;
    static mut __stack_el0_end: u64;
    static mut __stack_el0_start: u64;
    static mut __stack_end: u64;

    static mut __tt_firm_start: u64;
    static mut __tt_firm_end: u64;
    static mut __tt_el1_ttbr0_start: u64;
    static mut __tt_el1_ttbr0_end: u64;
    static mut __tt_el1_ttbr1_start: u64;
    static mut __tt_el1_ttbr1_end: u64;

    static mut __el0_heap_start: u64;
    static mut __el0_heap_end: u64;

    static mut _end: u64;
}

pub struct VMTables {
    el1: &'static mut [u64],
    firm: &'static mut [u64],
}

pub fn enabled() -> Option<bool> {
    let mut sctlr: u32;

    let el = el::get_current_el();
    if el == 1 {
        unsafe { asm!("mrs $0, SCTLR_EL1" : "=r"(sctlr)) };
        Some(sctlr & 1 == 1)
    } else if el == 2 {
        unsafe { asm!("mrs $0, SCTLR_EL2" : "=r"(sctlr)) };
        Some(sctlr & 1 == 1)
    } else if el == 3 {
        unsafe { asm!("mrs $0, SCTLR_EL3" : "=r"(sctlr)) };
        Some((sctlr & 1) == 1)
    } else {
        None
    }
}

pub fn get_no_cache<T>() -> &'static mut T {
    unsafe {
        let addr = &mut __no_cache as *mut u64 as usize;
        (addr as *mut T).as_mut().unwrap()
    }
}

fn get_sctlr() -> u32 {
    let mut sctlr: u32 = 0;
    let el = el::get_current_el();
    if el == 1 {
        unsafe { asm!("mrs $0, SCTLR_EL1" : "=r"(sctlr)) };
    } else if el == 2 {
        unsafe { asm!("mrs $0, SCTLR_EL2" : "=r"(sctlr)) };
    } else if el == 3 {
        unsafe { asm!("mrs $0, SCTLR_EL3" : "=r"(sctlr)) };
    }

    sctlr
}

fn set_sctlr(sctlr: u32) {
    let el = el::get_current_el();
    if el == 1 {
        unsafe { asm!("msr SCTLR_EL1, $0" : : "r"(sctlr)) };
    } else if el == 2 {
        unsafe { asm!("msr SCTLR_EL2, $0" : : "r"(sctlr)) };
    } else if el == 3 {
        unsafe { asm!("msr SCTLR_EL3, $0" : : "r"(sctlr)) };
    }
}

/// disable data cache, then enable it when dropping
pub struct DisableCache;

impl DisableCache {
    pub fn new() -> DisableCache {
        let mut sctlr = get_sctlr();
        sctlr &= !(1 << 2);
        set_sctlr(sctlr);
        DisableCache{}
    }
}

impl Drop for DisableCache {
    fn drop(&mut self) {
        let mut sctlr = get_sctlr();
        sctlr |= 1 << 2;
        set_sctlr(sctlr);
    }
}

// 64KB page
// level 2 and 3 translation tables

const PAGESIZE: u64 = 64 * 1024;

// NSTable (63bit)
const FLAG_L2_NS: u64 = 1 << 63; // non secure table


const FLAG_L3_XN:   u64 = 1 << 54; // execute never
const FLAG_L3_PXN:  u64 = 1 << 53; // priviledged execute
const FLAG_L3_CONT: u64 = 1 << 52; // contiguous
const FLAG_L3_DBM:  u64 = 1 << 51; // dirty bit modifier
const FLAG_L3_AF:   u64 = 1 << 10; // access flag
const FLAG_L3_NS:   u64 = 1 <<  5; // non secure

// [9:8]: Shareability attribute, for Normal memory
//    | Shareability
// ---|------------------
// 00 | non sharedable
// 01 | reserved
// 10 | outer sharedable
// 11 | inner sharedable
const FLAG_L3_OSH: u64 = 0b10 << 8;
const FLAG_L3_ISH: u64 = 0b11 << 8;

// [7:6]: access permissions
//    | Access from            |
//    | higher Exception level | Access from EL0
// ---|------------------------|-----------------
// 00 | read/write             | none
// 01 | read/write             | read/write
// 10 | read-only              | none
// 11 | read-only              | read-only
const FLAG_L3_SH_RW_N:  u64 =    0;
const FLAG_L3_SH_RW_RW: u64 =    1 << 6;
const FLAG_L3_SH_R_N:   u64 = 0b10 << 6;
const FLAG_L3_SH_R_R:   u64 = 0b11 << 6;

// [4:2]: AttrIndx
// defined in MAIR register
const FLAG_L3_ATTR_MEM: u64 = 0     ; // normal memory
const FLAG_L3_ATTR_DEV: u64 = 1 << 2; // device MMIO
const FLAG_L3_ATTR_NC:  u64 = 2 << 2; // non-cachable


#[cfg(any(feature = "raspi3", feature = "raspi2"))]
pub const DRIVER_MEM_START: usize =  0x3C000000;

#[cfg(any(feature = "raspi3", feature = "raspi2"))]
pub const DRIVER_MEM_END:   usize =  0x40000000;

#[cfg(feature = "raspi4")]
pub const DRIVER_MEM_START: usize =  0xfd000000; // maybe...

#[cfg(feature = "raspi4")]
pub const DRIVER_MEM_END:   usize = 0x100000000; // maybe...

pub fn print_addr() {
    let addr = unsafe { &mut __data_start as *mut u64 as u64 };
    driver::uart::puts("__data_start         = 0x");
    driver::uart::hex(addr as u64);
    driver::uart::puts("\n");

    let addr = unsafe { &mut __data_end as *mut u64 as u64 };
    driver::uart::puts("__data_end           = 0x");
    driver::uart::hex(addr as u64);
    driver::uart::puts("\n");

    let addr = unsafe { &mut __bss_start as *mut u64 as u64 };
    driver::uart::puts("__bss_start          = 0x");
    driver::uart::hex(addr as u64);
    driver::uart::puts("\n");

    let addr = unsafe { &mut __bss_end as *mut u64 as u64 };
    driver::uart::puts("__bss_end            = 0x");
    driver::uart::hex(addr as u64);
    driver::uart::puts("\n");

    let addr = unsafe { &mut __no_cache as *mut u64 as u64 };
    driver::uart::puts("__no_cache           = 0x");
    driver::uart::hex(addr as u64);
    driver::uart::puts("\n");

    let addr = unsafe { &mut __stack_firm_end as *mut u64 as u64 };
    driver::uart::puts("__stack_firm_end     = 0x");
    driver::uart::hex(addr as u64);
    driver::uart::puts("\n");

    let addr = unsafe { &mut __stack_firm_start as *mut u64 as u64 };
    driver::uart::puts("__stack_firm_start   = 0x");
    driver::uart::hex(addr as u64);
    driver::uart::puts("\n");

    let addr = unsafe { &mut __stack_el1_end as *mut u64 as u64 };
    driver::uart::puts("__stack_el1_end      = 0x");
    driver::uart::hex(addr as u64);
    driver::uart::puts("\n");

    let addr = unsafe { &mut __stack_el1_start as *mut u64 as u64 };
    driver::uart::puts("__stack_el1_start    = 0x");
    driver::uart::hex(addr as u64);
    driver::uart::puts("\n");

    let addr = unsafe { &mut __stack_el0_end as *mut u64 as u64 };
    driver::uart::puts("__stack_el0_end      = 0x");
    driver::uart::hex(addr as u64);
    driver::uart::puts("\n");

    let addr = unsafe { &mut __stack_el0_start as *mut u64 as u64 };
    driver::uart::puts("__stack_el0_start    = 0x");
    driver::uart::hex(addr as u64);
    driver::uart::puts("\n");

    let addr = unsafe { &mut __tt_firm_start as *mut u64 as u64 };
    driver::uart::puts("__tt_firm_start      = 0x");
    driver::uart::hex(addr as u64);
    driver::uart::puts("\n");

    let addr = unsafe { &mut __tt_el1_ttbr0_start as *mut u64 as u64 };
    driver::uart::puts("__tt_el1_ttbr0_start = 0x");
    driver::uart::hex(addr as u64);
    driver::uart::puts("\n");

    let addr = unsafe { &mut __tt_el1_ttbr1_start as *mut u64 as u64 };
    driver::uart::puts("__tt_el1_ttbr1_start = 0x");
    driver::uart::hex(addr as u64);
    driver::uart::puts("\n");

    let addr = unsafe { &mut __el0_heap_start as *mut u64 as u64 };
    driver::uart::puts("__el0_heap_start     = 0x");
    driver::uart::hex(addr as u64);
    driver::uart::puts("\n");

    let addr = unsafe { &mut __el0_heap_end as *mut u64 as u64 };
    driver::uart::puts("__el0_heap_end       = 0x");
    driver::uart::hex(addr as u64);
    driver::uart::puts("\n");

    let addr = unsafe { &mut _end as *mut u64 as u64 };
    driver::uart::puts("_end                 = 0x");
    driver::uart::hex(addr as u64);
    driver::uart::puts("\n");
}

pub fn init() -> Option<VMTables> {
    print_addr();

    // check for 64KiB granule and at least 36 bits physical address bus
    let mut mmfr: u64;
    unsafe { asm!("mrs $0, id_aa64mmfr0_el1" : "=r" (mmfr)) };
    let b = mmfr & 0xF;
    if b < 1 /* 36 bits */ {
        driver::uart::puts("ERROR: 36 bit address space not supported\n");
        return None;
    }

    if mmfr & (0xF << 24) != 0 /* 64KiB */ {
        driver::uart::puts("ERROR: 64KiB granule not supported\n");
        return None;
    }

//#[cfg(any(feature = "raspi3", feature = "raspi2"))]
//    let ret = Some(VMTables{el1: init_el1(), firm: init_el2()} );

//#[cfg(feature = "raspi4")]
//    let ret = Some(VMTables{el1: init_el1(), firm: init_el3()} );
    let ret = Some(VMTables{el1: init_el1(), firm: init_el3()} );
        driver::uart::puts("Initialize mmu done!\n");
    ret
}

fn init_table_flat(tt: &'static mut [u64], addr: u64) -> &'static mut [u64] {
    let data_start = unsafe { &mut __data_start as *mut u64 as usize } >> 16;
    let stack_start = unsafe { &mut __stack_start as *mut u64 as usize } >> 16;
    let stack_end = unsafe { &mut __stack_end as *mut u64 as usize } >> 16;
    let no_cache = unsafe { &mut __no_cache as *mut u64 as usize } >> 16;

    for t in tt.iter_mut() {
        *t = 0;
    }

    // L2 table, 4GiB space
    for i in 0..8 {
//        tt[i] = addr + (i as u64 + 1) * 8192 * 8 | 0b11;
	tt[i] = addr + 8192 * 8 | 0b11;
    }

    // L3 table, instructions and read only data
    for i in 0..data_start {
        tt[i % 8192 + 8192] = (i * 64 * 1024) as u64 | 0b11 |
            FLAG_L3_AF | FLAG_L3_ISH | FLAG_L3_SH_R_R | FLAG_L3_ATTR_MEM;
    }

    // L3 table, data and bss
    for i in data_start..no_cache {
        tt[i % 8192 + 8192] = (i * 64 * 1024) as u64 | 0b11 |
            FLAG_L3_AF | FLAG_L3_XN | FLAG_L3_PXN | FLAG_L3_ISH | FLAG_L3_SH_RW_RW | FLAG_L3_ATTR_MEM;
    }

    tt[no_cache % 8192 + 8192] = no_cache as u64 * 64 * 1024 | 0b11 |
        FLAG_L3_AF | FLAG_L3_XN | FLAG_L3_PXN | FLAG_L3_ISH | FLAG_L3_SH_RW_RW | FLAG_L3_ATTR_NC;

    // L3 table, stack
    for i in stack_start..stack_end {
        tt[i %8192+ 8192] = (i * 64 * 1024) as u64 | 0b11 |
            FLAG_L3_AF | FLAG_L3_XN | FLAG_L3_PXN | FLAG_L3_ISH | FLAG_L3_SH_RW_N | FLAG_L3_ATTR_MEM;
    }

    // L3 table
    for i in stack_end..(8192 * 8) {
        tt[i %8192 + 8192] = (i * 64 * 1024) as u64 | 0b11 |
            FLAG_L3_NS | FLAG_L3_AF | FLAG_L3_ISH | FLAG_L3_SH_RW_RW | FLAG_L3_ATTR_NC;
    }

    let start = DRIVER_MEM_START >> 16; // div by 64 * 1024
    let end   = start + ((DRIVER_MEM_END - DRIVER_MEM_START) >> 16); // div by 64 * 1024

    // L3 table, device
    for i in start..end {
        tt[i %8192 + 8192] = (i * 64 * 1024) as u64 | 0b11 |
            FLAG_L3_NS | FLAG_L3_XN | FLAG_L3_PXN | FLAG_L3_AF | FLAG_L3_OSH | FLAG_L3_SH_RW_RW | FLAG_L3_ATTR_DEV;
    }

    tt
}

fn get_mair() -> u64 {
    (0xFF <<  0) | // AttrIdx=0: normal, IWBWA, OWBWA, NTR
    (0x04 <<  8) | // AttrIdx=1: device, nGnRE (must be OSH too)
    (0x44 << 16)   // AttrIdx=2: non cacheable
}

/// for TCR_EL2 and TCR_EL2
fn get_tcr() -> u64 {
    let mut mmfr: u64;
    unsafe { asm!("mrs $0, id_aa64mmfr0_el1" : "=r" (mmfr)) };
    let b = mmfr & 0xF;

    1 << 31 | // Res1
    1 << 23 | // Res1
    b << 16 |
    1 << 14 | // 64KiB granule
    3 << 12 | // inner shadable
    1 << 10 | // Normal memory, Outer Write-Back Read-Allocate Write-Allocate Cacheable.
    1 <<  8 | // Normal memory, Inner Write-Back Read-Allocate Write-Allocate Cacheable.
    22        // T0SZ = 22, 2 levels (level 2 and 3 translation tables), 2^42B (4TiB) space
}

fn update_sctlr(sctlr: u64) -> u64 {
    let sctlr =
        sctlr   |
        1 << 44 | // set DSSBS, enable speculative load and store
        1 << 12 | // set I, instruction cache
        1 <<  2 | // set C, data cache
        1;        // set M, enable MMU
    sctlr & !(
        1 << 25 | // clear EE
        1 << 19 | // clear WXN
        1 <<  3 | // clear SA
        1 <<  1   // clear A
    )
}

/// mask firmware's stack and transition table
fn mask_firm(tt: &'static mut [u64]) -> &'static mut [u64] {
    // mask EL3's transition table
    let end = unsafe { &mut __tt_firm_end as *mut u64 as usize } >> 16; // div by 64KiB
    let start = unsafe { &mut __tt_firm_start as *mut u64 as usize } >> 16; // div by 64KiB
    for i in start..end {
        tt[i %8192+ 8192] = 0;
    }

    tt
}

/// mask EL1's stack and transition table
fn mask_el1(tt: &'static mut [u64]) -> &'static mut [u64] {
    // mask stack of EL1 and EL0
    let end = unsafe { &mut __stack_el1_end as *mut u64 as usize } >> 16; // div by 64KiB
    let start = unsafe { &mut __stack_el0_start as *mut u64 as usize } >> 16; // div by 64KiB
    for i in end..start {
        tt[i%8192 + 8192] = 0;
    }

    // mask EL1's transition table
    let start = unsafe { &mut __tt_el1_ttbr0_start as *mut u64 as usize } >> 16; // div by 64KiB
    let end = unsafe { &mut __tt_el1_ttbr1_end as *mut u64 as usize } >> 16; // div by 64KiB
    for i in start..end {
        tt[i%8192 + 8192] = 0;
    }

    tt
}

/// set up EL3's page table, 64KB page, level 2 and 3 translation tables,
/// assume 2MiB stack space per CPU
fn init_el3() -> &'static mut [u64] {
    driver::uart::puts("MMU::init_el3!\n");
    let addr = unsafe { &mut __tt_firm_start as *mut u64 as u64 };
    let ptr  = addr as *mut u64;
    let tt   = unsafe { slice::from_raw_parts_mut(ptr, 8192 * 10) };
    let tt   = init_table_flat(tt, addr);
    driver::uart::puts("MMU::init_el3:2!\n");
    // detect stack over flow
    let end = unsafe { &mut __stack_firm_end as *mut u64 as usize };
    let start = unsafe { &mut __stack_firm_start as *mut u64 as usize };
    driver::uart::puts("MMU::init_el3:3!\n");
    // #CPU
    let nc = (start - end) >> 21; // div by 2MiB (32 pages)
    for i in 0..(nc - 1) {
        tt[((end >> 16) + i * 32)%8192 + 8192] = 0;
    }

    driver::uart::puts("MMU::init_el3:4!\n");
    let tt = mask_el1(tt);
    driver::uart::puts("MMU::init_el3:5!\n");
    // first, set Memory Attributes array, indexed by PT_MEM, PT_DEV, PT_NC in our example
    unsafe { asm!("msr mair_el3, $0" : : "r" (get_mair())) };

    // next, specify mapping characteristics in translate control register
    unsafe { asm!("msr tcr_el3, $0" : : "r" (get_tcr())) };

    // tell the MMU where our translation tables are.
    unsafe { asm!("msr ttbr0_el3, $0" : : "r" (addr + 1)) };
    driver::uart::puts("MMU::init_el3:6!\n");

    // finally, toggle some bits in system control register to enable page translation
    let mut sctlr: u64;
    driver::uart::puts("MMU::init_el3:7!\n");
    unsafe { asm!("dsb ish; isb; mrs $0, sctlr_el3" : "=r" (sctlr)) };
    driver::uart::puts("MMU::init_el3:8!\n");
    sctlr = update_sctlr(sctlr);
    driver::uart::puts("MMU::init_el3:9!\n");
    unsafe { asm!("msr sctlr_el3, $0; dsb sy; isb" : : "r" (sctlr)) };
    driver::uart::puts("MMU::init_el3: end\n");
    tt
}

/// set up EL2's page table, 64KB page, level 2 and 3 translation tables,
/// assume 2MiB stack space per CPU
fn init_el2() -> &'static mut [u64] {
    let addr = unsafe { &mut __tt_firm_start as *mut u64 as u64 };
    let ptr  = addr as *mut u64;
    let tt   = unsafe { slice::from_raw_parts_mut(ptr, 8192 * 10) };
    let tt   = init_table_flat(tt, addr);

    // detect stack over flow
    let end = unsafe { &mut __stack_firm_end as *mut u64 as usize };
    let start = unsafe { &mut __stack_firm_start as *mut u64 as usize };

    // #CPU
    let nc = (start - end) >> 21; // div by 2MiB (32 pages)
    for i in 0..(nc - 1) {
        tt[(end >> 16) + i * 32 + 8192] = 0;
    }

    let tt = mask_el1(tt);

    // first, set Memory Attributes array, indexed by PT_MEM, PT_DEV, PT_NC in our example
    unsafe { asm!("msr mair_el2, $0" : : "r" (get_mair())) };

    // next, specify mapping characteristics in translate control register
    unsafe { asm!("msr tcr_el2, $0" : : "r" (get_tcr())) };

    // tell the MMU where our translation tables are.
    unsafe { asm!("msr ttbr0_el2, $0" : : "r" (addr + 1)) };

    // finally, toggle some bits in system control register to enable page translation
    let mut sctlr: u64;
    unsafe { asm!("dsb ish; isb; mrs $0, sctlr_el2" : "=r" (sctlr)) };
    sctlr = update_sctlr(sctlr);
    unsafe { asm!("msr sctlr_el2, $0; dsb sy; isb" : : "r" (sctlr)) };

    tt
}

/// set up EL1's page table, 64KB page, level 2 and 3 translation tables,
/// assume 2MiB stack space per CPU
fn init_el1() -> &'static mut [u64] {
    // TTBR0: user space
    let ttbr0 = unsafe { &mut __tt_el1_ttbr0_start as *mut u64 as u64 };
    let ptr  = ttbr0 as *mut u64;
    let tt   = unsafe { slice::from_raw_parts_mut(ptr, 8192 * 10) };
    let tt   = init_table_flat(tt, ttbr0);

    // detect stack over flow
    let end = unsafe { &mut __stack_el0_end as *mut u64 as usize };
    let start = unsafe { &mut __stack_el0_start as *mut u64 as usize };

    // #CPU
    let nc = (start - end) >> 21; // div by 2MiB (32 pages)
    for i in 0..(nc - 1) {
        tt[(end >> 16) + i * 32 + 8192] = 0;
    }

    let tt = mask_firm(tt);

    // user space stack
    let end = unsafe { &mut __stack_el0_end as *mut u64 as usize } >> 16;
    let start = unsafe { &mut __stack_el0_start as *mut u64 as usize } >> 16;
    for i in end..start {
        tt[i%8192 + 8192] = (i << 16) as u64 | 0b11 |
            FLAG_L3_AF | FLAG_L3_XN | FLAG_L3_PXN | FLAG_L3_ISH | FLAG_L3_SH_RW_RW | FLAG_L3_ATTR_MEM;
    }

    // mask EL1's stack
    let end = unsafe { &mut __stack_el1_end as *mut u64 as usize } >> 16; // div by 64KiB
    let start = unsafe { &mut __stack_el1_start as *mut u64 as usize } >> 16; // div by 64KiB
    for i in end..start {
        tt[i%8192 + 8192] = 0;
    }

    // EL0, heap
    let start = unsafe { &mut __el0_heap_start as *mut u64 as usize } >> 16;
    let end = unsafe { &mut __el0_heap_end as *mut u64 as usize } >> 16;
    for i in start..end {
        tt[i%8192 + 8192] = (i * 64 * 1024) as u64 | 0b11 |
            FLAG_L3_AF | FLAG_L3_XN | FLAG_L3_PXN | FLAG_L3_ISH | FLAG_L3_SH_RW_RW | FLAG_L3_ATTR_MEM;
    }

    //-------------------------------------------------------------------------
    // TTBR1: kernel space

    // kernel space memory
    let ttbr1 = unsafe { &mut __tt_el1_ttbr1_start as *mut u64 as u64 };
    let ptr  = ttbr1 as *mut u64;
//    let tt   = unsafe { slice::from_raw_parts_mut(ptr, 8192 * 2) };
    let tt   = unsafe { slice::from_raw_parts_mut(ptr, 8192 * 2) };

    // zero clear
    for v in tt.iter_mut() {
	//        *v = 0;
	*v = (ttbr1 + 65536) | 0b11;
    }

    tt[0] = (ttbr1 + 65536) | 0b11;

    // kernel stack
    let end = unsafe { &mut __stack_el1_end as *mut u64 as usize } >> 16;
    let start = unsafe { &mut __stack_el1_start as *mut u64 as usize } >> 16;
    driver::uart::puts("TTBR1 initialize start!\n");
    for i in end..start {
        tt[i % 8192 + 8192] = (i << 16) as u64 | 0b11 |
            FLAG_L3_AF | FLAG_L3_XN | FLAG_L3_PXN | FLAG_L3_ISH | FLAG_L3_SH_RW_N | FLAG_L3_ATTR_MEM;
    }
    driver::uart::puts("TTBR1 initialize end!\n");
    // detect stack over flow
    let end = unsafe { &mut __stack_el1_end as *mut u64 as usize };
    for i in 0..(nc - 1) {
        tt[(end >> 16) % 8129 + i * 32 + 8192] = 0;
    }

    // user space transition table
    let start = unsafe { &mut __tt_el1_ttbr0_start as *mut u64 as usize } >> 16;
    let end = unsafe { &mut __tt_el1_ttbr0_end as *mut u64 as usize } >> 16;
    for i in start..end {
        tt[i % 8192 + 8192] = (i * 64 * 1024) as u64 | 0b11 |
            FLAG_L3_NS | FLAG_L3_AF | FLAG_L3_ISH | FLAG_L3_SH_RW_N | FLAG_L3_ATTR_NC;
    }
    driver::uart::puts("TTBR1 initialize end2!\n");
    //-------------------------------------------------------------------------

    // first, set Memory Attributes array, indexed by PT_MEM, PT_DEV, PT_NC in our example
    unsafe { asm!("msr mair_el1, $0" : : "r" (get_mair())) };

    let mut mmfr: u64;
    unsafe { asm!("mrs $0, id_aa64mmfr0_el1" : "=r" (mmfr)) };
    let b = mmfr & 0xF;
    driver::uart::puts("TTBR1 initialize end3!\n");
    let tcr: u64 =
         b << 32 |
         3 << 30 | // 64KiB granule, TTBR1_EL1
         3 << 28 | // inner shadable, TTBR1_EL1
         1 << 26 | // Normal memory, Outer Write-Back Read-Allocate Write-Allocate Cacheable, TTBR1_EL1
         1 << 24 | // Normal memory, Inner Write-Back Read-Allocate Write-Allocate Cacheable, TTBR1_EL1
        22 << 16 | // T1SZ = 22, 2 levels (level 2 and 3 translation tables), 2^42B (4TiB) space
         1 << 14 | // 64KiB granule
         3 << 12 | // inner shadable, TTBR0_EL1
         1 << 10 | // Normal memory, Outer Write-Back Read-Allocate Write-Allocate Cacheable, TTBR0_EL1
         1 <<  8 | // Normal memory, Inner Write-Back Read-Allocate Write-Allocate Cacheable, TTBR0_EL1
        22;        // T0SZ = 22, 2 levels (level 2 and 3 translation tables), 2^42B (4TiB) space

    // next, specify mapping characteristics in translate control register
    unsafe { asm!("msr tcr_el1, $0" : : "r" (tcr)) };
    driver::uart::puts("TTBR1 initialize end4!\n");
    // tell the MMU where our translation tables are.
    unsafe { asm!("msr ttbr0_el1, $0" : : "r" (ttbr0 + 1)) };
    unsafe { asm!("msr ttbr1_el1, $0" : : "r" (ttbr1 + 1)) };
    driver::uart::puts("TTBR1 initialize end5!\n");
    // finally, toggle some bits in system control register to enable page translation
    let mut sctlr: u64;
    unsafe { asm!("dsb ish; isb; mrs $0, sctlr_el1" : "=r" (sctlr)) };
    sctlr = update_sctlr(sctlr);
    sctlr &= !(
        1 << 4 // clear SA0
    );
    unsafe { asm!("msr sctlr_el1, $0; dsb sy; isb" : : "r" (sctlr)) };
    driver::uart::puts("TTBR1 initialize end!\n");
    tt
}
