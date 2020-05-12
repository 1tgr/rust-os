use core::fmt::Write;
use core::panic::PanicInfo;
use syscall;
use syscall::libc_helpers::StdoutWriter;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let (file, line) = info.location().map(|l| (l.file(), l.line())).unwrap_or_default();
    let _ = if let Some(message) = info.message() {
        writeln!(&mut StdoutWriter, "Panic at {}({}): {}", file, line, message)
    } else {
        writeln!(&mut StdoutWriter, "Panic at {}({})", file, line)
    };

    syscall::exit_thread(-(line as i32))
}
