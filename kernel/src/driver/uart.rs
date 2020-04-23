use core::intrinsics::volatile_store;
use core::intrinsics::volatile_load;

use super::delays;

const SUNXI_UART0_BASE:	*mut u32 = 0x01C28000 as *mut u32;
const UART0_DR:   *mut u32 = 0x01C28000  as *mut u32;
const UART0_LSR:  *mut u32 = 0x01C28014  as *mut u32;

/// Initialiaze UART0 for serial console.
/// Set baud rate and characteristics (8N1) and map to GPIO 14 (Tx) and 15 (Rx).
/// 8N1 stands for "eight data bits, no parity, one stop bit".

extern {
    fn uart0_putc(c : u8);
    fn uart0_init();
}
pub fn init(uart_clock: u64, baudrate: u64) {
    // do nothing
}

/// send a character to serial console
//pub fn send(c :u32){
//    unsafe {uart0_putc(c as u8);}
//}

pub fn send(c : u32) {
    // wait until we can send
    unsafe { asm!("nop;") };
//    for _i in 0..1000 {
//	unsafe { asm!("nop;"::::"volatile") };
//    }
    while (unsafe { volatile_load(UART0_LSR) } & (1 <<5)) == 0 {
        unsafe { asm!("nop;") };
    }

    // write the character to the buffer
    if c == '\n' as u32 {
	unsafe {
            volatile_store(UART0_DR, '\r' as u32);
	}
    }
    unsafe {
        volatile_store(UART0_DR, c);
    }
}

/// print characters to serial console
pub fn puts(s : &str) {
    for c in s.bytes() {
        send(c as u32);
    }
}

/// print a 64-bit value in hexadecimal to serial console
pub fn hex(h : u64) {
    for i in (0..61).step_by(4).rev() {
        let mut n = (h >> i) & 0xF;
        n += if n > 9 { 0x37 } else { 0x30 };
        send(n as u32);
    }
}

/// print a 32-bit value in hexadecimal to serial console
pub fn hex32(h : u32) {
    for i in (0..29).step_by(4).rev() {
        let mut n = (h >> i) & 0xF;
        n += if n > 9 { 0x37 } else { 0x30 };
        send(n as u32);
    }
}

/// print a 64-bit value in decimal to serial console
pub fn decimal(mut h: u64) {
    let mut num = [0; 32];

    if h == 0 {
        send('0' as u32);
        return;
    }

    let mut i = 0;
    while h > 0 {
        let n = h % 10;
        h /= 10;
        num[i] = n + 0x30;
        i += 1;
    }

    while i > 0 {
        send(num[i - 1] as u32);
        i -= 1;
    }
}
