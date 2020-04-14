pub mod uart;
pub mod delays;

const UART_CLOCK: u64 = 48000000;
const UART_BAUD:  u64 = 115200;

pub struct Context {
}

/// Initlize UART0 for serial console with 115200 8n1,
/// and graphics with 1024x768 resolution.
pub fn init() -> Context {
    uart::init(UART_CLOCK, UART_BAUD);

    //rand::init();
    //uart::puts("initialized rand\n");

//    let g = graphics::init();
//    let m = mbox::get_memory();

    init_exceptions();

    Context{}
}

fn init_exceptions() {
    ()
}
