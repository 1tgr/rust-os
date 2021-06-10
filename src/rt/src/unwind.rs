use core::fmt::Write;
use core::hint::unreachable_unchecked;
use core::panic::PanicInfo;
use os::libc_helpers::StdoutWriter;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let (file, line) = info.location().map(|l| (l.file(), l.line())).unwrap_or_default();
    let _ = if let Some(message) = info.message() {
        writeln!(&mut StdoutWriter, "Panic at {}({}): {}", file, line, message)
    } else {
        writeln!(&mut StdoutWriter, "Panic at {}({})", file, line)
    };

    unsafe { unreachable_unchecked() }
}
