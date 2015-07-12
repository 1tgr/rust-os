/*
 * Rust BareBones OS
 * - By John Hodge (Mutabah/thePowersGang) 
 *
 * unwind.rs
 * - Stack unwind (panic) handling
 *
 * == LICENCE ==
 * This code has been put into the public domain, there are no restrictions on
 * its use, and the author takes no liability.
 */
use ::arch::cpu;
use ::arch::debug;
use core::fmt::Arguments;

#[lang="panic_fmt"]
#[no_mangle]
pub extern fn rust_begin_unwind(msg: Arguments, file: &'static str, line: u32) -> ! {
	// 'args' will print to the formatted string passed to panic!
	log!("file='{}', line={} :: {}", file, line, msg);

    let frame = cpu::current_frame();
    unsafe { debug::print_stack_trace(frame) };
	loop {}
}

#[lang="stack_exhausted"]
#[no_mangle]
pub fn __morestack() -> !
{
	loop {}
}


#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Clone,Copy)]
pub enum _Unwind_Reason_Code
{
	_URC_NO_REASON = 0,
	_URC_FOREIGN_EXCEPTION_CAUGHT = 1,
	_URC_FATAL_PHASE2_ERROR = 2,
	_URC_FATAL_PHASE1_ERROR = 3,
	_URC_NORMAL_STOP = 4,
	_URC_END_OF_STACK = 5,
	_URC_HANDLER_FOUND = 6,
	_URC_INSTALL_CONTEXT = 7,
	_URC_CONTINUE_UNWIND = 8,
}

#[allow(non_camel_case_types)]
#[derive(Clone,Copy)]
pub struct _Unwind_Context;

#[allow(non_camel_case_types)]
pub type _Unwind_Action = u32;
static _UA_SEARCH_PHASE: _Unwind_Action = 1;

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Clone,Copy)]
#[allow(raw_pointer_derive)]
pub struct _Unwind_Exception
{
	exception_class: u64,
	exception_cleanup: fn(_Unwind_Reason_Code,*const _Unwind_Exception),
	private: [u64; 2],
}

#[lang="eh_personality"]
#[no_mangle]
pub fn rust_eh_personality(
	_version: isize, _actions: _Unwind_Action, _exception_class: u64,
	_exception_object: &_Unwind_Exception, _context: &_Unwind_Context
	) -> _Unwind_Reason_Code
{
	loop{}
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn _Unwind_Resume()
{
	loop{}
}

