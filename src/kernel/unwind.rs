use crate::arch::{cpu, debug};
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let (file, line) = info.location().map(|l| (l.file(), l.line())).unwrap_or_default();
    if let Some(message) = info.message() {
        log!("file='{}', line={} :: {}", file, line, message);
    } else {
        log!("file='{}', line={} :: panic", file, line);
    }

    let frame = cpu::current_frame();
    unsafe { debug::print_stack_trace(frame) };
    loop {}
}
