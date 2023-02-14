use super::State;
use crate::Generator;
use core::arch::asm;
use std::panic::UnwindSafe;

#[derive(Debug, Default)]
#[repr(C)]
pub struct Context {
    pub rsp: u64,
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    rbx: u64,
    rbp: u64,
}

// Restore a generator context (which is assumed to have been saved in runtime)
#[naked]
pub unsafe extern "C" fn context_restore(ctx: &Context) {
    // rdi = context
    asm!(
        "mov rsp, [rdi + 0x00]",
        "mov r15, [rdi + 0x08]",
        "mov r14, [rdi + 0x10]",
        "mov r13, [rdi + 0x18]",
        "mov r12, [rdi + 0x20]",
        "mov rbx, [rdi + 0x28]",
        "mov rbp, [rdi + 0x30]",
        "mov rax, {state}",
        "ret",
        state = const State::Ready as u64,
        options(noreturn)
    );
}

// Switch to a new generator context (preserving the old one)
#[naked]
pub unsafe extern "C" fn context_switch(_old_ctx: &mut Context, _new_ctx: &mut Context) -> State {
    // rdi = old context
    // rsi = new context
    asm!(
        // preserve old context
        "mov [rdi + 0x00], rsp",
        "mov [rdi + 0x08], r15",
        "mov [rdi + 0x10], r14",
        "mov [rdi + 0x18], r13",
        "mov [rdi + 0x20], r12",
        "mov [rdi + 0x28], rbx",
        "mov [rdi + 0x30], rbp",
        // switch to a new one
        "mov rsp, [rsi + 0x00]",
        "mov r15, [rsi + 0x08]",
        "mov r14, [rsi + 0x10]",
        "mov r13, [rsi + 0x18]",
        "mov r12, [rsi + 0x20]",
        "mov rbx, [rsi + 0x28]",
        "mov rbp, [rsi + 0x30]",
        "mov rax, {state}",
        "ret",
        state = const State::Pending as u64,
        options(noreturn)
    );
}

// Initialize a generator stack with a closure
pub fn initialize_stack<F: FnOnce() -> R + UnwindSafe, R>(gen: &mut Generator, f: F) {
    unsafe {
        let s_ptr = gen.stack.as_mut_ptr().offset(gen.stack.len() as isize);
        let s_ptr = (s_ptr as usize & !15) as *mut u8; // stack must be aligned to 16 bytes
        let boxed_fn = Box::new(f);
        let f_ptr = Box::into_raw(boxed_fn);
        std::ptr::write(s_ptr.offset(-16) as *mut *mut dyn FnOnce() -> R, f_ptr);
        gen.ctx.rsp = s_ptr.offset(-32) as u64;
        std::ptr::write(gen.ctx.rsp as *mut u64, initialize_code as u64);
    }
}

// A initial function to call the actual closure
// We can abuse using of rsi that was filled in gen_switch_ctx
// This need just for the first time when we switch to a new generator
unsafe extern "C" fn initialize_code(_old_ctx: &Context, _new_ctx: &Context) {
    let fn_addr = (*_new_ctx).rsp + 16; // move to the address of the closure
    let addr = std::ptr::read(fn_addr as *mut *mut dyn FnOnce());
    let f = Box::from_raw(addr);
    f()
}
